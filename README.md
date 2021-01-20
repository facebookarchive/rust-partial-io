# partial-io

[![partial-io on crates.io](https://img.shields.io/crates/v/partial-io)](https://crates.io/crates/partial-io)
[![Documentation (latest release)](https://docs.rs/partial-io/badge.svg)](https://docs.rs/partial-io/)
[![Documentation (main)](https://img.shields.io/badge/docs-main-brightgreen)](https://facebookincubator.github.io/rust-partial-io/rustdoc/partial_io/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Helpers for testing I/O behavior with partial, interrupted and blocking reads and writes.

This library provides:

* `PartialRead` and `PartialWrite`, which wrap existing `Read` and
  `Write` implementations and allow specifying arbitrary behavior on the
  next `read`, `write` or `flush` call.
* With the optional `futures03` and `tokio02` features, `PartialAsyncRead` and
  `PartialAsyncWrite` to wrap existing `AsyncRead` and `AsyncWrite`
  implementations. These implementations are task-aware, so they will know
  how to pause and unpause tasks if they return a `WouldBlock` error.
* With the optional `quickcheck09` feature, generation of random sequences of
  operations which can be provided to one of the wrappers. See the
  `quickcheck_types` documentation for more.

## Motivation

A `Read` or `Write` wrapper is conceptually simple but can be difficult to
get right, especially if the wrapper has an internal buffer. Common
issues include:

* A partial read or write, even without an error, might leave the wrapper
  in an invalid state ([example fix][1]).

With the `AsyncRead` and `AsyncWrite` provided by `futures03` and `tokio02`:

* A call to `read_to_end` or `write_all` within the wrapper might be partly
  successful but then error out. These functions will return the error
  without informing the caller of how much was read or written. Wrappers
  with an internal buffer will want to advance their state corresponding
  to the partial success, so they can't use `read_to_end` or `write_all`
  ([example fix][2]).
* Instances must propagate `Poll::Pending` up, but that shouldn't leave
  them in an invalid state.

These situations can be hard to think about and hard to test.

`partial-io` can help in two ways:

1. For a known bug involving any of these situations, `partial-io` can help
   you write a test.
2. With the `quickcheck09` feature enabled, `partial-io` can also help shake
   out bugs in your wrapper. See `quickcheck_types` for more.

## Examples

```rust
use std::io::{self, Cursor, Read};

use partial_io::{PartialOp, PartialRead};

let data = b"Hello, world!".to_vec();
let cursor = Cursor::new(data);  // Cursor<Vec<u8>> implements io::Read
let ops = vec![PartialOp::Limited(7), PartialOp::Err(io::ErrorKind::Interrupted)];
let mut partial_read = PartialRead::new(cursor, ops);

let mut out = vec![0; 256];

// The first read will read 7 bytes.
assert_eq!(partial_read.read(&mut out).unwrap(), 7);
assert_eq!(&out[..7], b"Hello, ");
// The second read will fail with ErrorKind::Interrupted.
assert_eq!(partial_read.read(&mut out[7..]).unwrap_err().kind(), io::ErrorKind::Interrupted);
// The iterator has run out of operations, so it no longer truncates reads.
assert_eq!(partial_read.read(&mut out[7..]).unwrap(), 6);
assert_eq!(&out[..13], b"Hello, world!");
```

For a real-world example, see the [tests in `zstd-rs`].

[1]: https://github.com/gyscos/zstd-rs/commit/3123e418595f6badd5b06db2a14c4ff4555e7705
[2]: https://github.com/gyscos/zstd-rs/commit/02dc9d9a3419618fc729542b45c96c32b0f178bb
[tests in `zstd-rs`]: https://github.com/gyscos/zstd-rs/blob/master/src/stream/mod.rs

## Contributing

See the [CONTRIBUTING](CONTRIBUTING.md) file for how to help out.

## License

This project is available under the [MIT license](LICENSE).

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:

cargo install cargo-readme
cargo readme > README.md
-->
