# partial-io [![Build Status](https://travis-ci.org/facebookincubator/rust-partial-io.svg?branch=master)](https://travis-ci.org/facebookincubator/rust-partial-io) [![crates.io](https://img.shields.io/crates/v/partial-io.svg)](https://crates.io/crates/partial-io)

A Rust utility library to test resilience of `Read` or `Write` wrappers.

If you'd like to help out, see [CONTRIBUTING.md](CONTRIBUTING.md).

[Documentation (latest release)](https://docs.rs/partial-io)

[Documentation (master)](https://facebookincubator.github.io/rust-partial-io)

## Example

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

## Quick start

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
partial-io = "0.3"
```

Next, add this to your crate:

```rust
#[cfg(test)]
extern crate partial_io;
```

Now you can use `partial-io` in your tests.

## Tokio integration

`partial-io` can optionally integrate with the `tokio-io` library to provide
wrappers for `AsyncRead` and `AsyncWrite` instances. Enable the `tokio` feature
to use this:

```toml
[dev-dependencies]
partial-io = { version = "0.3", features = ["tokio"] }
```

## QuickCheck integration

`partial-io` can optionally integrate with the `quickcheck` library to generate
random test cases. Enable the `quickcheck` feature to use this:

```toml
[dev-dependencies]
partial-io = { version = "0.3", features = ["quickcheck"] }
```

See the
[documentation](https://facebookincubator.github.io/rust-partial-io/partial_io/quickcheck_types/index.html)
for how to use `quickcheck` to generate tests.

## License

`partial-io` is BSD-licensed. We also provide an additional patent grant.
