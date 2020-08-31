/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{make_ops, PartialOp};
use std::{
    cmp, io,
    task::{Context, Poll},
};

pub(crate) struct FuturesOps {
    ops: Box<dyn Iterator<Item = PartialOp> + Send>,
}

impl FuturesOps {
    /// Creates a new instance of `TokioOps`.
    pub(crate) fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = PartialOp> + 'static,
        I::IntoIter: Send,
    {
        Self {
            ops: make_ops(iter),
        }
    }

    /// Replaces ops with a new iterator.
    pub(crate) fn replace<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PartialOp> + 'static,
        I::IntoIter: Send,
    {
        self.ops = make_ops(iter)
    }

    /// Helper for poll methods.
    ///
    /// `cb` is the callback that implements the actual logic. The second argument is `Some(n)` to
    /// limit the number of bytes being written, or `None` for unlimited.
    pub(crate) fn poll_impl<T>(
        &mut self,
        cx: &mut Context,
        cb: impl FnOnce(&mut Context, Option<usize>) -> Poll<io::Result<T>>,
        remaining: usize,
        err_str: &'static str,
    ) -> Poll<io::Result<T>> {
        loop {
            match self.ops.next() {
                Some(PartialOp::Limited(n)) => {
                    let len = cmp::min(n, remaining);
                    break cb(cx, Some(len));
                }
                Some(PartialOp::Err(kind)) => {
                    if kind == io::ErrorKind::WouldBlock {
                        // Async* instances must convert WouldBlock errors to Poll::Pending and
                        // reschedule the task.
                        cx.waker().wake_by_ref();
                        break Poll::Pending;
                    } else if kind == io::ErrorKind::Interrupted {
                        // Async* instances must retry on Interrupted errors.
                        continue;
                    } else {
                        break Poll::Ready(Err(io::Error::new(kind, err_str)));
                    }
                }
                Some(PartialOp::Unlimited) | None => break cb(cx, None),
            }
        }
    }

    /// Helper for poll methods that ignore the length specified in `PartialOp::Limited`.
    pub(crate) fn poll_impl_no_limit<T>(
        &mut self,
        cx: &mut Context,
        cb: impl FnOnce(&mut Context) -> Poll<io::Result<T>>,
        err_str: &'static str,
    ) -> Poll<io::Result<T>> {
        loop {
            match self.ops.next() {
                Some(PartialOp::Err(kind)) => {
                    if kind == io::ErrorKind::WouldBlock {
                        // Async* instances must convert WouldBlock errors to Poll::Pending and
                        // reschedule the task.
                        cx.waker().wake_by_ref();
                        break Poll::Pending;
                    } else if kind == io::ErrorKind::Interrupted {
                        // Async* instances must retry on interrupted errors.
                        continue;
                    } else {
                        break Poll::Ready(Err(io::Error::new(kind, err_str)));
                    }
                }
                _ => break cb(cx),
            }
        }
    }
}
