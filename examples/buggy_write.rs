/*
 *  Copyright (c) 2017-present, Facebook, Inc.
 *  All rights reserved.
 *
 *  This source code is licensed under the BSD-style license found in the
 *  LICENSE file in the root directory of this source tree. An additional grant
 *  of patent rights can be found in the PATENTS file in the same directory.
 *
 */
//! An example of a buggy buffered writer that does not handle
//! `io::ErrorKind::Interrupted` properly.

#![deny(warnings)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate partial_io;
extern crate quickcheck;

use std::io::{self, Write};

/// A buffered writer whose `write` method is faulty.
pub struct BuggyWrite<W> {
    inner: W,
    buf: Vec<u8>,
    offset: usize,
}

impl<W: Write> BuggyWrite<W> {
    pub fn new(inner: W) -> Self {
        BuggyWrite {
            inner: inner,
            buf: Vec::with_capacity(256),
            offset: 0,
        }
    }

    fn write_from_offset(&mut self) -> io::Result<()> {
        while self.offset < self.buf.len() {
            self.offset += self.inner.write(&self.buf[self.offset..])?;
        }
        Ok(())
    }

    fn reset_buffer(&mut self) {
        unsafe {
            self.buf.set_len(0);
        }
        self.offset = 0;
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for BuggyWrite<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write out anything that is currently in the internal buffer.
        if self.offset < self.buf.len() {
            self.write_from_offset()?;
        }

        // Reset the internal buffer.
        self.reset_buffer();

        // Read from the provided buffer.
        self.buf.extend_from_slice(buf);

        // BUG: it is incorrect to call write immediately because if it fails,
        // we'd have read some bytes from the buffer without telling the caller
        // how many.
        // XXX: To fix the bug, comment out the next line.
        self.write_from_offset()?;
        Ok(self.buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Flush out any data that can be flushed out.
        while self.offset < self.buf.len() {
            self.write_from_offset()?;
        }

        // If that succeeded, reset the internal buffer.
        self.reset_buffer();

        // Flush the inner writer
        self.inner.flush()
    }
}

fn main() {
    test::buggy_write();
    // To run this test and see it fail, uncomment the next line. To fix the
    // bug, see `examples/buggy_write.rs`.
    //test::quickcheck_buggy_write();
}

mod test {
    //! Tests to demonstrate how to use partial-io to catch bugs in `buggy_write`.

    // * 'cargo test' doesn't support running tests inside examples.
    // * We'd like to make it possible to run this example to test it out.
    // * There's no way to programmatically collect and run tests in Rust.
    //
    // As a compromise, have tests marked 'pub' and private wrappers marked
    // 'test'.

    use std::io::{self, Write};

    use quickcheck::{TestResult, quickcheck};

    use partial_io::{GenInterrupted, PartialOp, PartialWithErrors, PartialWrite};

    use super::*;

    lazy_static! {
        // These strings have been chosen to be around the default size for
        // quickcheck (100). With significantly smaller or larger inputs, the
        // results might not be as good.
        static ref HELLO_STR: Vec<u8> = "Hello".repeat(50).into_bytes();
        static ref WORLD_STR: Vec<u8> = "World".repeat(40).into_bytes();
    }

    #[test]
    fn test_buggy_write() {
        buggy_write();
    }

    pub fn buggy_write() {
        let partial = vec![PartialOp::Err(io::ErrorKind::Interrupted),
                           PartialOp::Unlimited];
        let (hello_res, world_res, flush_res, inner) = buggy_write_internal(partial);
        assert_eq!(hello_res.unwrap_err().kind(), io::ErrorKind::Interrupted);
        assert_eq!(world_res.unwrap(), 5 * 40);
        assert_eq!(flush_res.unwrap(), ());

        // Note that inner has both "Hello" and "World" in it, even though according
        // to what the API returned it should only have had "World" in it.
        let mut expected = Vec::new();
        expected.extend_from_slice(&HELLO_STR);
        expected.extend_from_slice(&WORLD_STR);
        assert_eq!(inner, expected);
    }

    /// Test that quickcheck catches buggy writes.
    ///
    /// To run this test and see it fail, remove the #[ignore] annotation. To
    /// fix the bug, see `examples/buggy_write.rs`.
    #[test]
    #[ignore]
    fn test_quickcheck_buggy_write() {
        quickcheck_buggy_write();
    }

    pub fn quickcheck_buggy_write() {
        quickcheck(quickcheck_buggy_write2 as fn(PartialWithErrors<GenInterrupted>) -> TestResult);
    }

    fn quickcheck_buggy_write2(partial: PartialWithErrors<GenInterrupted>) -> TestResult {
        let (hello_res, world_res, flush_res, inner) = buggy_write_internal(partial);
        // If flush_res failed then we can't really do anything since we don't know
        // how much was written internally. Otherwise hello_res and world_res should
        // work.
        if flush_res.is_err() {
            return TestResult::discard();
        }

        let mut expected = Vec::new();
        if hello_res.is_ok() {
            expected.extend_from_slice(&HELLO_STR);
        }
        if world_res.is_ok() {
            expected.extend_from_slice(&WORLD_STR);
        }
        assert_eq!(inner, expected);
        TestResult::passed()
    }

    fn buggy_write_internal<I>(partial_iter: I)
                               -> (io::Result<usize>, io::Result<usize>, io::Result<()>, Vec<u8>)
        where I: IntoIterator<Item = PartialOp> + 'static
    {
        let inner = Vec::new();
        let partial_writer = PartialWrite::new(inner, partial_iter);

        let mut buggy_write = BuggyWrite::new(partial_writer);

        // Try writing a couple of things into it.
        let hello_res = buggy_write.write(&HELLO_STR);
        let world_res = buggy_write.write(&WORLD_STR);

        // Flush the contents to make sure nothing remains in the internal buffer.
        let flush_res = buggy_write.flush();

        let inner = buggy_write.into_inner().into_inner();

        (hello_res, world_res, flush_res, inner)
    }

}
