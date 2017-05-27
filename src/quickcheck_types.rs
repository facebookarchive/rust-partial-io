/*
 *  Copyright (c) 2017-present, Facebook, Inc.
 *  All rights reserved.
 *
 *  This source code is licensed under the BSD-style license found in the
 *  LICENSE file in the root directory of this source tree. An additional grant
 *  of patent rights can be found in the PATENTS file in the same directory.
 *
 */
//! `QuickCheck` support for partial IO operations.
//!
//! This module allows sequences of [`PartialOp`]s to be randomly generated. These
//! sequences can then be fed into a [`PartialRead`], [`PartialWrite`],
//! [`PartialAsyncRead`] or [`PartialAsyncWrite`].
//!
//! Once `quickcheck` has identified a failing test case, it will shrink the
//! sequence of `PartialOp`s and find a minimal test case. This minimal case can
//! then be used to reproduce the issue.
//!
//! To generate random sequences of operations, write a `quickcheck` test with a
//! `PartialWithErrors<GE>` input, where `GE` implements [`GenError`]. Then pass
//! the sequence in as the second argument to the partial wrapper.
//!
//! Several implementations of `GenError` are provided. These can be used to
//! customize the sorts of errors generated. For even more customization, you
//! can write your own `GenError` implementation.
//!
//! # Examples
//!
//! ```rust,ignore
//! extern crate quickcheck;
//! use partial_io::{GenInterrupted, PartialWithErrors};
//!
//! quickcheck! {
//!     fn test_something(seq: PartialWithErrors<GenInterrupted>) {
//!         let reader = ...;
//!         let partial_reader = PartialRead::new(reader, seq);
//!         // ...
//!     }
//! }
//! ```
//!
//! For a detailed example, see `examples/buggy_write.rs` in this repository.
//!
//! [`PartialOp`]: ../struct.PartialOp.html
//! [`PartialRead`]: ../struct.PartialRead.html
//! [`PartialWrite`]: ../struct.PartialWrite.html
//! [`PartialAsyncRead`]: ../struct.PartialAsyncRead.html
//! [`PartialAsyncWrite`]: ../struct.PartialAsyncWrite.html
//! [`GenError`]: trait.GenError.html

use std::io;
use std::marker::PhantomData;
use std::ops::Deref;

use quickcheck::{Arbitrary, Gen, empty_shrinker};

use PartialOp;

/// Given a custom error generator, randomly generate a list of `PartialOp`s.
#[derive(Clone, Debug)]
pub struct PartialWithErrors<GE> {
    items: Vec<PartialOp>,
    _marker: PhantomData<GE>,
}

impl<GE> IntoIterator for PartialWithErrors<GE> {
    type Item = PartialOp;
    type IntoIter = ::std::vec::IntoIter<PartialOp>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<GE> Deref for PartialWithErrors<GE> {
    type Target = [PartialOp];
    fn deref(&self) -> &Self::Target {
        self.items.deref()
    }
}

/// Represents a way to generate `io::ErrorKind` instances.
///
/// See [the module level documentation](index.html) for more.
pub trait GenError: Clone + Default + Send {
    /// Optionally generate an `io::ErrorKind` instance.
    fn gen_error<G: Gen>(&mut self, g: &mut G) -> Option<io::ErrorKind>;
}

/// Generate an `ErrorKind::Interrupted` error 20% of the time.
///
/// See [the module level documentation](index.html) for more.
#[derive(Clone, Debug, Default)]
pub struct GenInterrupted;

/// Generate an `ErrorKind::WouldBlock` error 20% of the time.
///
/// See [the module level documentation](index.html) for more.
#[derive(Clone, Debug, Default)]
pub struct GenWouldBlock;

/// Generate `Interrupted` and `WouldBlock` errors 10% of the time each.
///
/// See [the module level documentation](index.html) for more.
#[derive(Clone, Debug, Default)]
pub struct GenInterruptedWouldBlock;

macro_rules! impl_gen_error {
    ($id: ident, [$($errors:expr),+]) => {
        impl GenError for $id {
            fn gen_error<G: Gen>(&mut self, g: &mut G) -> Option<io::ErrorKind> {
                // 20% chance to generate an error.
                if g.gen_weighted_bool(5) {
                    Some(g.choose(&[$($errors,)*]).unwrap().clone())
                } else {
                    None
                }
            }
        }
    }
}

impl_gen_error!(GenInterrupted, [io::ErrorKind::Interrupted]);
impl_gen_error!(GenWouldBlock, [io::ErrorKind::WouldBlock]);
impl_gen_error!(GenInterruptedWouldBlock,
                [io::ErrorKind::Interrupted, io::ErrorKind::WouldBlock]);

/// Do not generate any errors. The only operations generated will be
/// `PartialOp::Limited` instances.
///
/// See [the module level documentation](index.html) for more.
#[derive(Clone, Debug, Default)]
pub struct GenNoErrors;

impl GenError for GenNoErrors {
    fn gen_error<G: Gen>(&mut self, _g: &mut G) -> Option<io::ErrorKind> {
        None
    }
}

impl<GE> Arbitrary for PartialWithErrors<GE>
    where GE: GenError + 'static
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let size = g.size();
        // Generate a sequence of operations. A uniform distribution for this is
        // fine because the goal is to shake bugs out relatively effectively.
        let mut gen_error = GE::default();
        let items: Vec<_> = (0..size)
            .map(|_| {
                     match gen_error.gen_error(g) {
                         Some(err) => PartialOp::Err(err),
                         // Don't generate 0 because for writers it can mean that
                         // writes are no longer accepted.
                         None => PartialOp::Limited(g.gen_range(1, size)),
                     }
                 })
            .collect();
        PartialWithErrors {
            items: items,
            _marker: PhantomData,
        }
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        Box::new(self.items
                     .clone()
                     .shrink()
                     .map(|items| {
                              PartialWithErrors {
                                  items: items,
                                  _marker: PhantomData,
                              }
                          }))
    }
}

impl Arbitrary for PartialOp {
    fn arbitrary<G: Gen>(_g: &mut G) -> Self {
        // We only use this for shrink, so we don't need to implement this.
        unimplemented!();
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        match *self {
            // Skip 0 because for writers it can mean that writes are no longer
            // accepted.
            PartialOp::Limited(n) => {
                Box::new(n.shrink()
                             .filter(|k| k != &0)
                             .map(PartialOp::Limited))
            }
            _ => empty_shrinker(),
        }
    }
}
