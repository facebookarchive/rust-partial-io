# partial-io [![Build Status](https://travis-ci.org/facebookincubator/rust-partial-io.svg?branch=master)](https://travis-ci.org/facebookincubator/rust-partial-io)

A Rust utility library to test resilience of `Read` or `Write` wrappers.

If you'd like to help out, see [CONTRIBUTING.md](CONTRIBUTING.md).

[Documentation](https://facebookincubator.github.io/rust-partial-io)

## Quick start

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
partial-io = "0.1"
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
[dev-dependencies.partial-io]
version = "0.1"
features = ["tokio"]
```

## QuickCheck integration

`partial-io` can optionally integrate with the `quickcheck` library to generate
random test cases. Enable the `quickcheck` feature to use this:

```toml
[dev-dependencies.partial-io]
version = "0.1"
features = ["quickcheck"]
```

See the
[documentation](https://facebookincubator.github.io/rust-partial-io/partial_io/quickcheck_types/index.html)
for how to use `quickcheck` to generate tests.

## License

`partial-io` is BSD-licensed. We also provide an additional patent grant.
