//! Constructing octets sequences from data.
//!
//! Composing encoded data always happens directly into an octets builder.
//! Therefore, no `Composer` type is necessary. This module only defines a
//! trait [`Compose`] which is used as an extension trait to provide
//! `compose` methods for built-in types.

// XXX Add the Compose trait or remove the module.
