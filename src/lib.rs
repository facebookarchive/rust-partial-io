/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![deny(warnings)]

//! Helpers for testing I/O behavior with partial, interrupted and blocking reads and writes.
//!
//! This library provides:
//!
//! * `PartialRead` and `PartialWrite`, which wrap existing `Read` and
//!   `Write` implementations and allow specifying arbitrary behavior on the
//!   next `read`, `write` or `flush` call.
//! * With the optional `futures03` and `tokio1` features, `PartialAsyncRead` and
//!   `PartialAsyncWrite` to wrap existing `AsyncRead` and `AsyncWrite`
//!   implementations. These implementations are task-aware, so they will know
//!   how to pause and unpause tasks if they return a `WouldBlock` error.
//! * With the optional `quickcheck1` feature, generation of random sequences of
//!   operations which can be provided to one of the wrappers. See the
//!   `quickcheck_types` documentation for more.
//!
//! # Motivation
//!
//! A `Read` or `Write` wrapper is conceptually simple but can be difficult to
//! get right, especially if the wrapper has an internal buffer. Common
//! issues include:
//!
//! * A partial read or write, even without an error, might leave the wrapper
//!   in an invalid state ([example fix][1]).
//!
//! With the `AsyncRead` and `AsyncWrite` provided by `futures03` and `tokio1`:
//!
//! * A call to `read_to_end` or `write_all` within the wrapper might be partly
//!   successful but then error out. These functions will return the error
//!   without informing the caller of how much was read or written. Wrappers
//!   with an internal buffer will want to advance their state corresponding
//!   to the partial success, so they can't use `read_to_end` or `write_all`
//!   ([example fix][2]).
//! * Instances must propagate `Poll::Pending` up, but that shouldn't leave
//!   them in an invalid state.
//!
//! These situations can be hard to think about and hard to test.
//!
//! `partial-io` can help in two ways:
//!
//! 1. For a known bug involving any of these situations, `partial-io` can help
//!    you write a test.
//! 2. With the `quickcheck1` feature enabled, `partial-io` can also help shake
//!    out bugs in your wrapper. See `quickcheck_types` for more.
//!
//! # Examples
//!
//! ```rust
//! use std::io::{self, Cursor, Read};
//!
//! use partial_io::{PartialOp, PartialRead};
//!
//! let data = b"Hello, world!".to_vec();
//! let cursor = Cursor::new(data);  // Cursor<Vec<u8>> implements io::Read
//! let ops = vec![PartialOp::Limited(7), PartialOp::Err(io::ErrorKind::Interrupted)];
//! let mut partial_read = PartialRead::new(cursor, ops);
//!
//! let mut out = vec![0; 256];
//!
//! // The first read will read 7 bytes.
//! assert_eq!(partial_read.read(&mut out).unwrap(), 7);
//! assert_eq!(&out[..7], b"Hello, ");
//! // The second read will fail with ErrorKind::Interrupted.
//! assert_eq!(partial_read.read(&mut out[7..]).unwrap_err().kind(), io::ErrorKind::Interrupted);
//! // The iterator has run out of operations, so it no longer truncates reads.
//! assert_eq!(partial_read.read(&mut out[7..]).unwrap(), 6);
//! assert_eq!(&out[..13], b"Hello, world!");
//! ```
//!
//! For a real-world example, see the [tests in `zstd-rs`].
//!
//! [1]: https://github.com/gyscos/zstd-rs/commit/3123e418595f6badd5b06db2a14c4ff4555e7705
//! [2]: https://github.com/gyscos/zstd-rs/commit/02dc9d9a3419618fc729542b45c96c32b0f178bb
//! [tests in `zstd-rs`]: https://github.com/gyscos/zstd-rs/blob/master/src/stream/mod.rs

#[cfg(feature = "futures03")]
mod async_read;
#[cfg(feature = "futures03")]
mod async_write;
#[cfg(feature = "futures03")]
mod futures_util;
#[cfg(feature = "quickcheck1")]
pub mod quickcheck_types;
mod read;
mod write;

use std::io;

#[cfg(feature = "tokio1")]
pub use crate::async_read::tokio_impl::ReadBufExt;
#[cfg(feature = "futures03")]
pub use crate::async_read::PartialAsyncRead;
#[cfg(feature = "futures03")]
pub use crate::async_write::PartialAsyncWrite;
pub use crate::{read::PartialRead, write::PartialWrite};

/// What to do the next time an IO operation is performed.
///
/// This is not the same as `io::Result<Option<usize>>` because it contains
/// `io::ErrorKind` instances, not `io::Error` instances. This allows it to be
/// clonable.
#[derive(Clone, Debug)]
pub enum PartialOp {
    /// Limit the next IO operation to a certain number of bytes.
    ///
    /// The wrapper will call into the inner `Read` or `Write`
    /// instance. Depending on what the underlying operation does, this may
    /// return an error or a fewer number of bytes.
    ///
    /// Some methods like `Write::flush` and `AsyncWrite::poll_flush` don't
    /// have a limit. For these methods, `Limited(n)` behaves the same as
    /// `Unlimited`.
    Limited(usize),

    /// Do not limit the next IO operation.
    ///
    /// The wrapper will call into the inner `Read` or `Write`
    /// instance. Depending on what the underlying operation does, this may
    /// return an error or a limited number of bytes.
    Unlimited,

    /// Return an error instead of calling into the underlying operation.
    ///
    /// For methods on `Async` traits:
    /// * `ErrorKind::WouldBlock` is translated to `Poll::Pending` and the task
    ///   is scheduled to be woken up in the future.
    /// * `ErrorKind::Interrupted` causes a retry.
    Err(io::ErrorKind),
}

#[inline]
fn make_ops<I>(iter: I) -> Box<dyn Iterator<Item = PartialOp> + Send>
where
    I: IntoIterator<Item = PartialOp> + 'static,
    I::IntoIter: Send,
{
    Box::new(iter.into_iter().fuse())
}

#[cfg(test)]
mod tests {
    pub fn assert_send<S: Send>() {}
}
