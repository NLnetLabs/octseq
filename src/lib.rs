//! Variable length octet sequences.
//!
//! This crate provides a set of basic traits that allow defining types that
//! are generic over a variable length sequence of octets (or, vulgo: bytes).
//! It implements these traits for most commonly used types of such sequences
//! and provides a array-backed type for use in a no-std environment.
//!
//! These traits are all defined – and explained en detail – in the [_traits_]
//! module. They are, however, all re-exported here at the crate root.
//!
//! In addition, the crate provides a helper type to extract data that has
//! been encoded into an octet sequences. This has been dubbed _parsing_ and
//! is provided via the [_parse_] module.

#![no_std]
#![allow(renamed_and_removed_lints)]
#![allow(clippy::unknown_clippy_lints)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "std"))]
#[allow(unused_imports)] // Import macros even if unused.
#[macro_use]
extern crate std;

pub use self::traits::*;
pub use self::parse::*;

pub mod traits;
pub mod parse;
pub mod serde;
pub mod str;
pub mod types;

