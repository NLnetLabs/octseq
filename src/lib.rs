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
//! In addition, the crate provides helper types to extract data from and
//! construct such octet sequences in a sequential fashion. Extracting data
//! has been termed _parsing_ and is provided via the [_parse_] module.
//! For constructing sequences, the term _composing_ was chosen to clearly
//! distinguish the process from formatting into human-readable test. Types
//! for this purpose live in the [_compose_] module.

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
pub use self::compose::*;

pub mod traits;
pub mod parse;
pub mod compose;
pub mod serde;
pub mod types;

