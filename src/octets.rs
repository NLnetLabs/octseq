//! Octets Sequences
//!
//! An octets sequence – or just octets for short – is a variable length
//! sequence of bytes. In their most simple form, any type that implements
//! `AsRef<[u8]>` can serve as octets. However, in some cases additional
//! functionality is required.
//!
//! The trait [`Octets`] allows taking a sub-sequence, called a ‘range’, out
//! of the octets in the cheapest way possible. For most types, ranges will
//! be octet slices `&[u8]` but some shareable types (most notably
//! `bytes::Bytes`) allow ranges to be owned values, thus avoiding the
//! lifetime limitations a slice would bring. Therefore, `Octets` allows
//! defining the type of a range as an associated type.


use core::convert::Infallible;
use core::ops::{Index, RangeBounds};
#[cfg(feature = "bytes")] use bytes::{Bytes, BytesMut};
#[cfg(feature = "std")] use std::borrow::Cow;
#[cfg(feature = "std")] use std::vec::Vec;
#[cfg(feature = "heapless")] use crate::builder::ShortBuf;


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

#[cfg(feature = "heapless")]
impl<const N: usize> Octets for heapless::Vec<u8, N> {
    type Range<'a> = &'a [u8] where Self: 'a;

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.index(
            (range.start_bound().cloned(), range.end_bound().cloned())
        )
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

#[cfg(feature = "heapless")]
impl<Source, const N: usize> OctetsFrom<Source> for heapless::Vec<u8, N>
where
    Source: AsRef<[u8]>,
{
    type Error = ShortBuf;

    fn try_octets_from(source: Source) -> Result<Self, ShortBuf> {
        heapless::Vec::from_slice(source.as_ref()).map_err(|_| ShortBuf)
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


//------------ SmallOctets ---------------------------------------------------

/// A octets vector that doesn’t allocate for small sizes.
#[cfg(feature = "smallvec")]
pub type SmallOctets = smallvec::SmallVec<[u8; 24]>;

