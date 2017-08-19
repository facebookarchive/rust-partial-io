/*
 *  Copyright (c) 2017-present, Facebook, Inc.
 *  All rights reserved.
 *
 *  This source code is licensed under the BSD-style license found in the
 *  LICENSE file in the root directory of this source tree. An additional grant
 *  of patent rights can be found in the PATENTS file in the same directory.
 *
 */
#![deny(warnings)]
// futures-rs 0.1.14 changed task::park() and unpark() to current() and
// notify(), respectively. Keep using the old versions for compat.
#![allow(deprecated)]

//! Helpers for testing I/O behavior with partial, interrupted and blocking reads and writes.
//!
//! This library provides:
//!
//! * [`PartialRead`] and [`PartialWrite`], which wrap existing `Read` and
//!   `Write` implementations and allow specifying arbitrary behavior on the
//!   next `read`, `write` or `flush` call.
//! * With the optional `tokio` feature, [`PartialAsyncRead`] and
//!   [`PartialAsyncWrite`] to wrap existing `AsyncRead` and `AsyncWrite`
//!   implementations. These implementations are task-aware, so they will know
//!   how to pause and unpause tasks if they return a `WouldBlock` error.
//! * With the optional `quickcheck` feature, generation of random sequences of
//!   operations which can be provided to one of the wrappers. See the
//!   [`quickcheck_types` documentation](quickcheck_types/index.html) for more.
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
//! With `tokio`'s `AsyncRead` and `AsyncWrite`:
//!
//! * `read_to_end` or `write_all` within the wrapper might be partly
//!   successful but then error out. These functions will return the error
//!   without informing the caller of how much was read or written. Wrappers
//!   with an internal buffer will want to advance their state corresponding
//!   to the partial success, so they can't use `read_to_end` or `write_all`
//!   ([example fix][2]).
//! * Instances cannot propagate `ErrorKind::Interrupted` failures up. Wrappers
//!   must always retry.
//! * Instances must propagate `ErrorKind::WouldBlock` failures up, but that
//!   shouldn't leave them in an invalid state.
//!
//! These situations can be hard to think about and hard to test.
//!
//! `partial-io` can help in two ways:
//!
//! 1. For a known bug involving any of these situations, `partial-io` can help
//!    you write a test.
//! 2. With the `quickcheck` feature enabled, `partial-io` can also help shake
//!    out bugs in your wrapper. See [`quickcheck_types`] for more.
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
//! [`PartialRead`]: struct.PartialRead.html
//! [`PartialWrite`]: struct.PartialWrite.html
//! [`PartialAsyncRead`]: struct.PartialAsyncRead.html
//! [`PartialAsyncWrite`]: struct.PartialAsyncWrite.html
//! [`quickcheck_types`]: quickcheck_types/index.html
//! [1]: https://github.com/gyscos/zstd-rs/commit/3123e418595f6badd5b06db2a14c4ff4555e7705
//! [2]: https://github.com/gyscos/zstd-rs/commit/02dc9d9a3419618fc729542b45c96c32b0f178bb
//! [tests in `zstd-rs`]: https://github.com/gyscos/zstd-rs/blob/master/src/stream/mod.rs

#[cfg(feature = "tokio")]
extern crate futures;
#[cfg(feature = "quickcheck")]
extern crate quickcheck;
#[cfg(feature = "tokio")]
extern crate tokio_io;

#[cfg(feature = "tokio")]
mod async_read;
#[cfg(feature = "tokio")]
mod async_write;
#[cfg(feature = "quickcheck")]
pub mod quickcheck_types;
mod read;
mod write;

use std::io;

#[cfg(feature = "tokio")]
pub use async_read::PartialAsyncRead;
#[cfg(feature = "tokio")]
pub use async_write::PartialAsyncWrite;
#[cfg(feature = "quickcheck")]
pub use quickcheck_types::{GenError, GenInterrupted, GenInterruptedWouldBlock, GenNoErrors,
                           GenWouldBlock, PartialWithErrors};
pub use read::PartialRead;
pub use write::PartialWrite;

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
    Limited(usize),

    /// Do not limit the next IO operation.
    ///
    /// The wrapper will call into the inner `Read` or `Write`
    /// instance. Depending on what the underlying operation does, this may
    /// return an error or a limited number of bytes.
    Unlimited,

    /// Return an error instead of calling into the underlying operation.
    Err(io::ErrorKind),
}

#[inline]
fn make_ops<I>(iter: I) -> Box<Iterator<Item = PartialOp>>
where
    I: IntoIterator<Item = PartialOp> + 'static,
{
    Box::new(iter.into_iter().fuse())
}
