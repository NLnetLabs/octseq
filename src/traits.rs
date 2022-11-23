//! Variable length octet sequences.
//!
//! This crate provides a set of basic traits that allow defining types that
//! are generic over a variable length sequence of octets (or, vulgo: bytes).
//! It implements these traits for most commonly used types of such sequences
//! and provides a array-backed type for use in a no-std environment.
//!
//! # Types of Octet Sequences
//!
//! There are two fundamental types of octet sequences. If a sequence
//! contains content of a constant size, we call it simply ‘octets.’ If the
//! sequence is actually a buffer into which octets can be placed, it is
//! called an `octets builder.`
//!
//! # Octets
//!
//! In their most simple form, any type that implements `AsRef<[u8]>` can
//! serve as octets. However, in some cases additional functionality is
//! required.
//!
//! The trait [`Octets`] allows taking a sub-sequence, called a ‘range’, out
//! of the octets in the cheapest way possible. For most types, ranges will
//! be octet slices `&[u8]` but some shareable types (most notably
//! `bytes::Bytes`) allow ranges to be owned values, thus avoiding the
//! lifetime limitations a slice would bring. Therefore, `Octets` allows
//! defining the type of a range as an associated type.
//!
//! # Octets Builders
//!
//! Octets builders and their [`OctetsBuilder`] trait are comparatively
//! straightforward. They represent a buffer to which octets can be appended.
//! Whether the buffer can grow to accommodate appended data depends on the
//! underlying type. Because it may not, all such operations may fail with an
//! error defined by the trait implementation via
//! `OctetsBuilder::AppendError`. Types where appending never fails (other
//! than possibly panicking when running out of memory) should use the core
//! library’s `Infallible` type here. This unlocks additional methods that
//! don’t return a result and thus avoid an otherwise necessary unwrap.
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
//!
//!
//! # Conversion Traits
//!
//! A series of special traits allows converting octets into octets builder
//! and vice versa. They pair octets with their natural builders via
//! associated types. These conversions are always cyclic, i.e., if an
//! octets value is converted into a builder and then that builder is
//! converted back into an octets value, the initial and final octets value
//! have the same type.
//!
//!
//! # Using Trait Bounds
//!
//! When using these traits as bounds for generic types, always limit yourself
//! to the most loose bounds you can get away with. Not all types holding
//! octet sequences can actually implement all these traits, so by being too
//! eager you may paint yourself into a corner.
//!
//! In many cases you can get away with a simple `AsRef<[u8]>` bound. Only use
//! an explicit `Octets` bound when you need to return a range that may be
//! kept around.
//!
//! Similarly, only demand of an octets builder what you actually need. Even
//! something as seemingly trivial as `AsMut<[u8]>` isn’t a given. For
//! instance, `Cow<[u8]>` doesn’t implement it but otherwise is a perfectly
//! fine octets builder.


use core::fmt;
use core::convert::Infallible;
use core::ops::{Index, RangeBounds};
#[cfg(feature = "bytes")] use bytes::{Bytes, BytesMut};
#[cfg(feature = "std")] use std::borrow::Cow;
#[cfg(feature = "std")] use std::vec::Vec;


//============ Octets and Octet Builders =====================================

//------------ Octets --------------------------------------------------------

/// A type representing an octets sequence.
///
/// The primary purpose of the trait is to allow access to a sub-sequence,
/// called a ‘range.’ The type of this range is given via the `Range`
/// associated type. For most types it will be a `&[u8]` with a lifetime
/// equal to that of a reference. Only if an owned range can be created
/// cheaply, it should be that type.
pub trait Octets: AsRef<[u8]> {
    type Range<'a>: Octets where Self: 'a;

    /// Returns a sub-sequence or ‘range’ of the sequence.
    ///
    /// # Panics
    ///
    /// The method should panic if `start` or `end` are greater than the
    /// length of the octets sequence or if `start` is greater than `end`.
    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_>;
}

impl<'t, T: Octets + ?Sized> Octets for &'t T {
    type Range<'a> = <T as Octets>::Range<'a> where Self: 'a;

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        (*self).range(range)
    }
}

impl Octets for [u8] {
    type Range<'a> = &'a [u8];

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.index(
            (range.start_bound().cloned(), range.end_bound().cloned())
        )
    }
}

#[cfg(feature = "std")]
impl<'c> Octets for Cow<'c, [u8]> {
    type Range<'a> = &'a [u8] where Self: 'a;

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.as_ref().range(range)
    }
}

#[cfg(feature = "std")]
impl Octets for Vec<u8> {
    type Range<'a> = &'a [u8];

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.as_slice().range(range)
    }
}

#[cfg(feature = "bytes")]
impl Octets for Bytes {
    type Range<'a> = Bytes;

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.slice(range)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> Octets for smallvec::SmallVec<A> {
    type Range<'a> = &'a [u8] where A: 'a;

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.as_slice().range(range)
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

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> Truncate for smallvec::SmallVec<A> {
    fn truncate(&mut self, len: usize) {
        self.truncate(len)
    }
}

//------------ OctetsFrom ----------------------------------------------------

/// Convert a type from one octets type to another.
///
/// This trait allows creating a value of a type that is generic over an
/// octets sequence from an identical value using a different type of octets
/// sequence.
///
/// This is different from just `From` in that the conversion may fail if the
/// source sequence is longer than the space available for the target type.
pub trait OctetsFrom<Source>: Sized {
    type Error;

    /// Performs the conversion.
    fn try_octets_from(source: Source) -> Result<Self, Self::Error>;

    /// Performs an infallible conversion.
    fn octets_from(source: Source) -> Self
    where Self::Error: Into<Infallible> {
        // XXX Use .into_ok() once that is stable.
        match Self::try_octets_from(source) {
            Ok(ok) => ok,
            Err(_) => unreachable!()
        }
    }
}

impl<'a, Source: AsRef<[u8]> + 'a> OctetsFrom<&'a Source> for &'a [u8] {
    type Error = Infallible;

    fn try_octets_from(source: &'a Source) -> Result<Self, Self::Error> {
        Ok(source.as_ref())
    }
}

#[cfg(feature = "std")]
impl<Source> OctetsFrom<Source> for Vec<u8>
where
    Self: From<Source>,
{
    type Error = Infallible;

    fn try_octets_from(source: Source) -> Result<Self, Self::Error> {
        Ok(From::from(source))
    }
}

#[cfg(feature = "bytes")]
impl<Source> OctetsFrom<Source> for Bytes
where
    Self: From<Source>,
{
    type Error = Infallible;

    fn try_octets_from(source: Source) -> Result<Self, Self::Error> {
        Ok(From::from(source))
    }
}

#[cfg(feature = "bytes")]
impl<Source> OctetsFrom<Source> for BytesMut
where
    Self: From<Source>,
{
    type Error = Infallible;

    fn try_octets_from(source: Source) -> Result<Self, Self::Error> {
        Ok(From::from(source))
    }
}

#[cfg(features = "smallvec")]
impl<Source, A> OctetsFrom<Source> for smallvec::SmallVec<A>
where
    Source: AsRef<u8>,
    A: Array<Item = u8>,
{
    type Error = Infallible;

    fn try_octets_from(source: Source) -> Result<Self, Self::Infallible> {
        Ok(smallvec::ToSmallVec::to_smallvec(source.as_ref()))
    }
}


//------------ OctetsInto ----------------------------------------------------

/// Convert a type from one octets type to another.
///
/// This trait allows trading in a value of a type that is generic over an
/// octets sequence for an identical value using a different type of octets
/// sequence.
///
/// This is different from just `Into` in that the conversion may fail if the
/// source sequence is longer than the space available for the target type.
///
/// This trait has a blanket implementation for all pairs of types where
/// `OctetsFrom` has been implemented.
pub trait OctetsInto<Target>: Sized {
    type Error;

    /// Performs the conversion.
    fn try_octets_into(self) -> Result<Target, Self::Error>;

    /// Performs an infallible conversion.
    fn octets_into(self) -> Target
    where Self::Error: Into<Infallible> {
        match self.try_octets_into() {
            Ok(ok) => ok,
            Err(_) => unreachable!()
        }
    }
}

impl<Source, Target: OctetsFrom<Source>> OctetsInto<Target> for Source {
    type Error = <Target as OctetsFrom<Source>>::Error;

    fn try_octets_into(self) -> Result<Target, Self::Error> {
        Target::try_octets_from(self)
    }
}


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

    /// The type of the error that happens when appending data fails.
    ///
    /// The only error that is allowed to happen is that the builder does
    /// not have enough space available to append the data. This case is
    /// covered by [`ShortBuf`] which should be used by octets builders that
    /// have limited space.
    ///
    /// For types, such as `Vec<u8>` or `BytesMut` where appending never
    /// fails (other than with an out-of-memory panic), this should be
    /// `Infallible` (or `!` when that becomes stable).
    ///
    /// `ShortBuf` has an `impl From<Infallible>`, so requiring
    /// `Into<ShortBuf>` as a trait bound covers those two types. Doing so
    /// avoids complicated trait bounds in types being generic over octets
    /// builders, even if it might seem a bit strange at first.
    type AppendError: Into<ShortBuf>;

    /// Appends the content of a slice to the builder.
    ///
    /// If there isn’t enough space available for appending the slice,
    /// returns an error and leaves the builder alone.
    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError>;

    /// Converts the builder into immutable octets.
    fn freeze(self) -> Self::Octets
    where Self: Sized;

    /// Returns the length of the already assembled data.
    fn len(&self) -> usize;

    /// Returns whether the builder is currently empty.
    fn is_empty(&self) -> bool;

    fn append_slice(&mut self, slice: &[u8])
    where Self::AppendError: Into<Infallible> {
        // XXX Use .into_ok() once that is stable.
        let _ = self.try_append_slice(slice);
    }

    /// Appends all data or nothing.
    ///
    /// The method executes the provided closure that presumably will try to
    /// append data to the builder and propagates an error from the builder.
    /// If the closure returns with an error, the builder is truncated back
    /// to the length from before the closure was executed.
    ///
    /// Note that upon an error the builder is _only_ truncated. If the
    /// closure modified any already present data via `AsMut<[u8]>`, these
    /// modification will survive.
    fn try_append_all<F>(&mut self, op: F) -> Result<(), Self::AppendError>
    where
        Self: Truncate,
        F: FnOnce(&mut Self) -> Result<(), Self::AppendError>,
    {
        let pos = self.len();
        match op(self) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.truncate(pos);
                Err(err)
            }
        }
    }
}

#[cfg(feature = "std")]
impl OctetsBuilder for Vec<u8> {
    type Octets = Self;
    type AppendError = Infallible;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}

#[cfg(feature = "std")]
impl<'a> OctetsBuilder for Cow<'a, [u8]> {
    type Octets = Self;
    type AppendError = Infallible;

    fn try_append_slice(
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

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    fn freeze(self) -> Self::Octets {
        self
    }
}

#[cfg(feature = "bytes")]
impl OctetsBuilder for BytesMut {
    type Octets = Bytes;
    type AppendError = Infallible;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn freeze(self) -> Self::Octets {
        self.freeze()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array<Item = u8>> OctetsBuilder for smallvec::SmallVec<A> {
    type Octets = Self;
    type AppendError = Infallible;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        self.extend_from_slice(slice);
        Ok(())
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn freeze(self) -> Self::Octets {
        self
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


//============ Error Types ===================================================

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

//--- From

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


//============ Testing =======================================================

#[cfg(test)]
mod test {
}

