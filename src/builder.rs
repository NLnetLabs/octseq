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

    /// An error returned when building fails.
    ///
    /// The type argument `E` is a user supplied type covering errors
    /// outside the provenance of the octets builder itself. If nothing can
    /// happen outside, use `Infallible` should be used, i.e., in this case
    /// the error type should be `Self::BuildError<Infallible>`.
    ///
    /// If appending octets to the buffer cannot result in any errors, this
    /// should just be `E`.
    type BuildError<E>: From<E>;

    /// The result of appending data.
    ///
    /// This is the result type for cases where no outside error can happen,
    /// i.e., where the error type would indeed be
    /// `Self::BuildError<Infallible>`.
    ///
    /// For implementations where appending data can fail, this should be a
    /// `Result<T, Error>` with `Error` being the error type for appending
    /// data. Not that this is different from `Self::BuildError<E>` in that
    /// it is _not_ wrapping some user supplied type `E`.
    ///
    /// For unlimited buffers, this should be simply `T`.
    type AppendResult<T>: CollapseResult<T, Self::BuildError<Infallible>>;

    /// Appends the content of a slice to the builder.
    ///
    /// If there isn’t enough space available for appending the slice,
    /// returns an error and leaves the builder alone.
    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>>;

    /// Appends the content of a slice to the builder.
    ///
    /// If there isn’t enough space available for appending the slice,
    /// returns an error and leaves the builder alone.
    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Self::AppendResult<()> {
        self.try_append_slice(slice).collapse()
    }

    /// Converts the builder into immutable octets.
    fn freeze(self) -> Self::Octets
    where Self: Sized;
}

#[cfg(feature = "std")]
impl OctetsBuilder for Vec<u8> {
    type Octets = Self;
    type BuildError<E> = E;
    type AppendResult<T> = T;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>> {
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
    type BuildError<E> = E;
    type AppendResult<T> = T;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>> {
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
    type BuildError<E> = E;
    type AppendResult<T> = T;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>> {
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
    type BuildError<E> = E;
    type AppendResult<T> = T;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>> {
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
    type BuildError<E> = ShortBuild<E>;
    type AppendResult<T> = Result<T, ShortBuf>;

    fn try_append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::BuildError<Infallible>> {
        self.extend_from_slice(slice).map_err(|_| ShortBuild::ShortBuf)
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

/// A helper trait to allow removing the error case of infallible results.
///
/// This could technically also be achieved using `From`, however, the
/// blanket impl below will not be possible for that. So we just have to
/// cobble together a helper trait.
pub trait CollapseResult<T, E> {
    fn collapse_result(src: Result<T, E>) -> Self;
}

// This blanket impl will cover the cases for unlimited buffers, i.e., it
// will allow turning a `Result<T, Infallible>` into just a `T`.
//
// Octets builders that can error will have to impl the trait for their own
// error type.
impl<T> CollapseResult<T, Infallible> for T {
    fn collapse_result(src: Result<T, Infallible>) -> Self {
        match src {
            Ok(t) => t,
            Err(_) => unreachable!()
        }
    }
}

/// Another helper trait that acts as the reverse of `CollapseResult`.
///
/// This trait acts for `CollapseResult` as `Into` acts for `From`. It only
/// exists to make it possible to write out the trait bounds of
/// `OctetsBuilder::AppendResult<T>` while at the same time allow users to
/// call a method on whatever `OctetsBuilder::try_append_slice` returns to
/// collapse that result. Thus, it is only implemented via the below blanket
/// impl.
pub trait Collapse<U> {
    fn collapse(self) -> U;
}

impl<T, E, U: CollapseResult<T, E>> Collapse<U> for Result<T, E> {
    fn collapse(self) -> U {
        U::collapse_result(self)
    }
}


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

impl<T> CollapseResult<T, ShortBuild<Infallible>> for Result<T, ShortBuf> {
    fn collapse_result(src: Result<T, ShortBuild<Infallible>>) -> Self {
        src.map_err(|err| match err {
            ShortBuild::ShortBuf => ShortBuf,
            ShortBuild::Build(_) => unreachable!()
        })
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


//------------ ShortBuild ----------------------------------------------------

#[derive(Clone, Debug)]
pub enum ShortBuild<T> {
    Build(T),
    ShortBuf,
}

impl<T> From<T> for ShortBuild<T> {
    fn from(t: T) -> Self {
        ShortBuild::Build(t)
    }
}

//--- Display and Error

impl<T: fmt::Display> fmt::Display for ShortBuild<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShortBuild::Build(t) => t.fmt(f),
            ShortBuild::ShortBuf => ShortBuf.fmt(f)
        }
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug + fmt::Display> std::error::Error for ShortBuild<T> {}

