//! Octets Builders
//!
//! Octets builders, i.e., anything that implements the [`OctetsBuilder`]
//! trait, represent a buffer to which octets can be appended.
//! Whether the buffer can grow to accommodate appended data depends on the
//! underlying type.
//!
//! The [`OctetsBuilder`] trait only provides methods to append data to the
//! builder. Implementations may, however, provide more functionality. They
//! do so by implementing additional traits. Conversely, if additional
//! functionality is needed from a builder, this can be expressed by
//! adding trait bounds.
//!
//! Some examples are:
//!
//! * creating an empty octets builder from scratch is provided by the
//!   [`EmptyBuilder`] trait,
//! * looking at already assembled octets is done via `AsRef<[u8]>`,
//! * manipulation of already assembled octets requires `AsMut<[u8]>`, and
//! * truncating the sequence of assembled octets happens through
//!   [`Truncate`].

use core::fmt;
use core::convert::Infallible;
#[cfg(feature = "bytes")] use bytes::{Bytes, BytesMut};
#[cfg(feature = "std")] use std::borrow::Cow;
#[cfg(feature = "std")] use std::vec::Vec;


//------------ OctetsBuilder -------------------------------------------------

/// A buffer to construct an octet sequence.
///
/// Octet builders represent a buffer of space available for building an
/// octets sequence by appending the contents of octet slices. The buffers
/// may consist of a predefined amount of space or grow as needed.
///
/// Octet builders provide access to the already assembled data through
/// octet slices via their implementations of `AsRef<[u8]>` and
/// `AsMut<[u8]>`.
pub trait OctetsBuilder {
    /// The type of the octets the builder can be converted into.
    ///
    /// If `Octets` implements [`IntoBuilder`], the `Builder` associated
    /// type of that trait must be `Self`.
    ///
    /// [`IntoBuilder`]: trait.IntoBuilder.html
    type Octets: AsRef<[u8]>;

    /// The error type when appending data fails.
    ///
    /// There are exactly two options for this type: Builders where appending
    /// never fails use `Infallible`. Builders with a limited buffer which 
    /// may have insufficient space for appending use [`ShortBuf`].
    ///
    /// The trait bound on the type allows upgrading the error to [`ShortBuf`]
    /// even for builders with unlimited space. This way, an error type for
    /// operations that use a builder doesn’t need to be generic over the
    /// append error type and can simply use a variant for anything
    /// `Into<ShortBuf>` instead.
    type AppendError: Into<ShortBuf>;

    /// Appends the content of a slice to the builder.
    ///
    /// If there isn’t enough space available for appending the slice,
    /// returns an error and leaves the builder alone.
    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError>;

    /// Converts the builder into immutable octets.
    fn freeze(self) -> Self::Octets
    where Self: Sized;
}

#[cfg(feature = "std")]
impl OctetsBuilder for Vec<u8> {
    type Octets = Self;
    type AppendError = Infallible;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}

#[cfg(feature = "std")]
impl<'a> OctetsBuilder for Cow<'a, [u8]> {
    type Octets = Self;
    type AppendError = Infallible;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        if let Cow::Owned(ref mut vec) = *self {
            vec.extend_from_slice(slice);
        } else {
            let mut vec = std::mem::replace(
                self, Cow::Borrowed(b"")
            ).into_owned();
            vec.extend_from_slice(slice);
            *self = Cow::Owned(vec);
        }
        Ok(())
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}

#[cfg(feature = "bytes")]
impl OctetsBuilder for BytesMut {
    type Octets = Bytes;
    type AppendError = Infallible;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn freeze(self) -> Self::Octets {
        self.freeze()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> OctetsBuilder for smallvec::SmallVec<A> {
    type Octets = Self;
    type AppendError = Infallible;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> OctetsBuilder for heapless::Vec<u8, N> {
    type Octets = Self;
    type AppendError = ShortBuf;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice).map_err(|_| ShortBuf)
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}


//------------ Truncate ------------------------------------------------------

/// An octet sequence that can be shortened.
pub trait Truncate {
    /// Truncate the sequence to `len` octets.
    ///
    /// If `len` is larger than the length of the sequence, nothing happens.
    fn truncate(&mut self, len: usize);
}

impl<'a> Truncate for &'a [u8] {
    fn truncate(&mut self, len: usize) {
        if len < self.len() {
            *self = &self[..len]
        }
    }
}

#[cfg(feature = "std")]
impl<'a> Truncate for Cow<'a, [u8]> {
    fn truncate(&mut self, len: usize) {
        match *self {
            Cow::Borrowed(ref mut slice) => *slice = &slice[..len],
            Cow::Owned(ref mut vec) => vec.truncate(len),
        }
    }
}

#[cfg(feature = "std")]
impl Truncate for Vec<u8> {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

#[cfg(feature = "bytes")]
impl Truncate for Bytes {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

#[cfg(feature = "bytes")]
impl Truncate for BytesMut {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> Truncate for smallvec::SmallVec<A> {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> Truncate for heapless::Vec<u8, N> {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}


//------------ EmptyBuilder --------------------------------------------------

/// An octets builder that can be newly created empty.
pub trait EmptyBuilder {
    /// Creates a new empty octets builder with a default size.
    fn empty() -> Self;

    /// Creates a new empty octets builder with a suggested initial size.
    ///
    /// The builder may or may not use the size provided by `capacity` as the
    /// initial size of the buffer. It may very well be possibly that the
    /// builder is never able to grow to this capacity at all. Therefore,
    /// even if you create a builder for your data size via this function,
    /// appending may still fail.
    fn with_capacity(capacity: usize) -> Self;
}

#[cfg(feature = "std")]
impl EmptyBuilder for Vec<u8> {
    fn empty() -> Self {
        Vec::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

#[cfg(feature = "bytes")]
impl EmptyBuilder for BytesMut {
    fn empty() -> Self {
        BytesMut::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        BytesMut::with_capacity(capacity)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> EmptyBuilder for smallvec::SmallVec<A> {
    fn empty() -> Self {
        smallvec::SmallVec::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        smallvec::SmallVec::with_capacity(capacity)
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> EmptyBuilder for heapless::Vec<u8, N> {
    fn empty() -> Self {
        Self::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        debug_assert!(capacity <= N);
        Self::with_capacity(capacity)
    }
}

//------------ IntoBuilder ---------------------------------------------------

/// An octets type that can be converted into an octets builder.
pub trait IntoBuilder {
    /// The type of octets builder this octets type can be converted into.
    type Builder: OctetsBuilder;

    /// Converts an octets value into an octets builder.
    fn into_builder(self) -> Self::Builder;
}

#[cfg(feature = "std")]
impl IntoBuilder for Vec<u8> {
    type Builder = Self;

    fn into_builder(self) -> Self::Builder {
        self
    }
}

#[cfg(feature = "std")]
impl<'a> IntoBuilder for &'a [u8] {
    type Builder = Vec<u8>;

    fn into_builder(self) -> Self::Builder {
        self.into()
    }
}

#[cfg(feature = "std")]
impl<'a> IntoBuilder for Cow<'a, [u8]> {
    type Builder = Self;

    fn into_builder(self) -> Self::Builder {
        self
    }
}

#[cfg(feature = "bytes")]
impl IntoBuilder for Bytes {
    type Builder = BytesMut;

    fn into_builder(self) -> Self::Builder {
        // XXX Currently, we need to copy to do this. If bytes gains a way
        //     to convert from Bytes to BytesMut for non-shared data without
        //     copying, we should change this.
        BytesMut::from(self.as_ref())
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> IntoBuilder for smallvec::SmallVec<A> {
    type Builder = Self;

    fn into_builder(self) -> Self::Builder {
        self
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> IntoBuilder for heapless::Vec<u8, N> {
    type Builder = Self;

    fn into_builder(self) -> Self::Builder {
        self
    }
}


//------------ FromBuilder ---------------------------------------------------

/// An octets type that can be created from an octets builder.
pub trait FromBuilder: AsRef<[u8]> + Sized {
    /// The type of builder this octets type can be created from.
    type Builder: OctetsBuilder<Octets = Self>;

    /// Creates an octets value from an octets builder.
    fn from_builder(builder: Self::Builder) -> Self;
}

#[cfg(feature = "std")]
impl FromBuilder for Vec<u8> {
    type Builder = Self;

    fn from_builder(builder: Self::Builder) -> Self {
        builder
    }
}

#[cfg(feature = "std")]
impl<'a> FromBuilder for Cow<'a, [u8]> {
    type Builder = Self;

    fn from_builder(builder: Self::Builder) -> Self {
        builder
    }
}

#[cfg(feature = "bytes")]
impl FromBuilder for Bytes {
    type Builder = BytesMut;

    fn from_builder(builder: Self::Builder) -> Self {
        builder.freeze()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> FromBuilder for smallvec::SmallVec<A> {
    type Builder = Self;

    fn from_builder(builder: Self::Builder) -> Self {
        builder
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> FromBuilder for heapless::Vec<u8, N> {
    type Builder = Self;

    fn from_builder(builder: Self::Builder) -> Self {
        builder
    }
}


//============ Error Handling ================================================

//------------ ShortBuf ------------------------------------------------------

/// An attempt was made to write beyond the end of a buffer.
///
/// This type is returned as an error by all functions and methods that append
/// data to an [octets builder] when the buffer size of the builder is not
/// sufficient to append the data.
///
/// [octets builder]: trait.OctetsBuilder.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortBuf;


//--- From and CollapseResult

impl From<Infallible> for ShortBuf {
    fn from(_: Infallible) -> ShortBuf {
        unreachable!()
    }
}


//--- Display and Error

impl fmt::Display for ShortBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("buffer size exceeded")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ShortBuf {}


//------------ Functions for Infallible --------------------------------------

/// Erases an error for infallible results.
///
/// This function can be used in place of the still unstable
/// `Result::into_ok` for operations on infallible octets builders.
///
/// If you perform multiple operations, [`with_infallible`] allows you to
/// use the question mark operator on them before erasing the error.
pub fn infallible<T, E: Into<Infallible>>(src: Result<T, E>) -> T {
    match src {
        Ok(ok) => ok,
        Err(_) => unreachable!(),
    }
}

/// Erases an error for a closure returninb an infallible results.
///
/// This function can be used for a sequence of operations on an infallible
/// octets builder. By wrapping these operations in a closure, you can still
/// use the question mark operator rather than having to wrap each individual
/// operation in [`infallible`].
pub fn with_infallible<F, T, E>(op: F) -> T
where
    F: FnOnce() -> Result<T, E>,
    E: Into<Infallible>,
{
    infallible(op())
}

