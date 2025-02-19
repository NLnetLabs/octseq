//! A fixed-capacity octets sequence.

use core::{cmp, fmt};
use core::ops::RangeBounds;
use crate::builder::{
    EmptyBuilder, FreezeBuilder, FromBuilder, IntoBuilder, OctetsBuilder,
    ShortBuf, Truncate,
};
use crate::octets::{Octets, OctetsFrom};


//------------ Array ---------------------------------------------------------

#[derive(Clone)]
pub struct Array<const N: usize> {
    octets: [u8; N],
    len: usize
}

impl<const N: usize> Array<N> {
    /// Creates a new empty value.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns an octets slice with the content of the array.
    pub fn as_slice(&self) -> &[u8] {
        &self.octets[..self.len]
    }

    /// Returns a mutable octets slice with the content of the array.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.octets[..self.len]
    }

    /// Resizes the array in place updating additional octets.
    ///
    /// The method only changes the length of the contained octets
    /// sequence. If `new_len` is greater than the current length, the
    /// content of the additional octets will be left at whatever they
    /// were.
    ///
    /// Returns an error if `new_len` is larger than the array size.
    pub fn resize_raw(&mut self, new_len: usize) -> Result<(), ShortBuf> {
        if new_len > N {
            Err(ShortBuf)
        }
        else {
            self.len = new_len;
            Ok(())
        }
    }
}


//--- Default

impl<const N: usize> Default for Array<N> {
    fn default() -> Self {
        Array {
            octets: [0; N],
            len: 0
        }
    }
}


//--- TryFrom

impl<'a, const N: usize> TryFrom<&'a [u8]> for Array<N> {
    type Error = ShortBuf;

    fn try_from(src: &'a [u8]) -> Result<Self, ShortBuf> {
        let len = src.len();
        if len > N {
            Err(ShortBuf)
        }
        else {
            let mut res = Self::default();
            res.octets[..len].copy_from_slice(src);
            res.len = len;
            Ok(res)
        }
    }
}


//--- Deref, AsRef, Borrow, and Mut versions

impl<const N: usize> core::ops::Deref for Array<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const N: usize> core::ops::DerefMut for Array<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<const N: usize> AsRef<[u8]> for Array<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> AsMut<[u8]> for Array<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl<const N: usize> core::borrow::Borrow<[u8]> for Array<N> {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> core::borrow::BorrowMut<[u8]> for Array<N> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

//--- Truncate

impl<const N: usize> Octets for Array<N> {
    type Range<'a> = &'a [u8];

    fn range(&self, range: impl RangeBounds<usize>) -> Self::Range<'_> {
        self.as_slice().range(range)
    }
}


//--- Truncate

impl<const N: usize> Truncate for Array<N> {
    fn truncate(&mut self, len: usize) {
        self.len = cmp::min(self.len, len)
    }
}


//--- OctetsBuilder, EmptyBuilder, and FreezeBuilder

impl<const N: usize> OctetsBuilder for Array<N> {
    type AppendError = ShortBuf;

    fn append_slice(
        &mut self, slice: &[u8]
    ) -> Result<(), Self::AppendError> {
        let end = self.len + slice.len();
        if end > N {
            return Err(ShortBuf)
        }
        self.octets[self.len..end].copy_from_slice(slice);
        self.len = end;
        Ok(())
    }
}

impl<const N: usize> EmptyBuilder for Array<N> {
    fn empty() -> Self {
        Default::default()
    }

    fn with_capacity(_capacity: usize) -> Self {
        Self::empty()
    }
}

impl<const N: usize> FreezeBuilder for Array<N> {
    type Octets = Self;

    fn freeze(self) -> Self::Octets {
        self
    }
}


//--- IntoBuilder, FromBuilder

impl<const N: usize> IntoBuilder for Array<N> {
    type Builder = Self;

    fn into_builder(self) -> Self::Builder {
        self
    }
}

impl<const N: usize> FromBuilder for Array<N> {
    type Builder = Self;

    fn from_builder(builder: Self::Builder) -> Self {
        builder
    }
}


//--- OctetsFrom

impl<Source: AsRef<[u8]>, const N: usize> OctetsFrom<Source> for Array<N> {
    type Error = ShortBuf;

    fn try_octets_from(source: Source) -> Result<Self, Self::Error> {
        Self::try_from(source.as_ref())
    }
}


//--- PartialEq and Eq

impl<T: AsRef<[u8]>, const N: usize> PartialEq<T> for Array<N> {
    fn eq(&self, other: &T) -> bool {
        self.as_slice().eq(other.as_ref())
    }
}

impl<const N: usize> Eq for Array<N> { }


//--- PartialOrd and Ord

impl<T: AsRef<[u8]>, const N: usize> PartialOrd<T> for Array<N> {
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}

impl<const N: usize> Ord for Array<N> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_slice().cmp(other)
    }
}


//--- Hash

impl<const N: usize> core::hash::Hash for Array<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}


//--- Debug

impl<const N: usize> fmt::Debug for Array<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("octets::Array")
            .field(&self.as_slice())
            .finish()
    }
}


//--- SerializeOctets and DeserializeOctets

#[cfg(feature = "serde")]
impl<const N: usize> crate::serde::SerializeOctets for Array<N> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> crate::serde::DeserializeOctets<'de> for Array<N> {
    type Visitor = ArrayVisitor<N>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        Self::visitor().deserialize(deserializer)
    }

    fn deserialize_with_visitor<D, V>(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
        V: serde::de::Visitor<'de>,
    {
        deserializer.deserialize_byte_buf(visitor)
    }

    fn visitor() -> Self::Visitor {
        ArrayVisitor
    }
}


//------------ ArrayVisitor ----------------------------------------------

#[cfg(feature = "serde")]
pub struct ArrayVisitor<const N: usize>;

#[cfg(feature = "serde")]
impl<const N: usize> ArrayVisitor<N> {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Array<N>, D::Error> {
        deserializer.deserialize_byte_buf(self)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::de::Visitor<'de> for ArrayVisitor<N> {
    type Value = Array<N>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an octet sequence")
    }

    fn visit_bytes<E: serde::de::Error>(
        self, value: &[u8]
    ) -> Result<Self::Value, E> {
        Array::try_from(value).map_err(E::custom)
    }
}

