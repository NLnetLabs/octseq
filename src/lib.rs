//! Variable length octet sequences.
//!
//! This crate provides a set of basic traits that allow defining types that
//! are generic over a variable length sequence of octets (or, vulgo: bytes).
//!
//! There are two groups of traits: those that represent behavior of imutable
//! octets sequences are collected in the module _[octets]_ and those for
//! building such sequences are in _[builder]_. Most traits from these modules
//! are also re-exported at the crate root for convenience.
//!
//! These traits are implemented for a number of types. Apart from `[u8]`,
//! the implementations are opt-in via features. These are:
//!
//! * `std` for `Vec<u8>`, `Cow<[u8]>`, and `Arc<[u8]>`,
//! * `bytes` for the `Bytes` and `BytesMut` types from the
//!   [bytes](https://crates.io/crates/bytes) crate,
//! * `heapless` for the `Vec<u8, N>` type from the
//!   [heapless](https://crates.io/crates/heapless) crate, and
//! * `smallvec` for a smallvec for item type `u8` from the
//!   [smallvec](https://crates.io/crates/smallvec) crate.
//!
//! A number of additional modules exist that provide a few helpful things:
//!
//! * The _[mod@array]_ module provides an octets builder backed by an octets
//!   array.
//! * The _[mod@str]_ module provides both imutable and buildable string types
//!   that are generic over the octets sequence they wrap.
//! * The
#![cfg_attr(feature = "serde", doc = "  _[serde]_")]
#![cfg_attr(not(feature = "serde"), doc = "  _serde_")]
//!   module, which needs to be enabled via the `serde`
//!   feature, provides traits and functions to more efficiently serialize
//!   octets sequences.

#![no_std]
#![allow(renamed_and_removed_lints)]
#![allow(clippy::unknown_clippy_lints)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(feature = "std"))]
#[allow(unused_imports)] // Import macros even if unused.
#[macro_use]
extern crate std;

pub use self::array::*;
pub use self::builder::*;
pub use self::octets::*;
pub use self::parse::*;

pub mod array;
pub mod builder;
pub mod octets;
pub mod parse;
pub mod serde;
pub mod str;
