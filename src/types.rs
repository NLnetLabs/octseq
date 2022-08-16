
use core::{cmp, fmt};
use crate::traits::{
    EmptyBuilder, FromBuilder, IntoBuilder, OctetsBuilder, OctetsFrom,
    ShortBuf, Truncate,
};


//------------ SmallOctets ---------------------------------------------------

/// A octets vector that doesn’t allocate for small sizes.
#[cfg(feature = "smallvec")]
pub type SmallOctets = smallvec::SmallVec<[u8; 24]>;


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

impl<const N: usize> Truncate for Array<N> {
    fn truncate(&mut self, len: usize) {
        self.len = cmp::min(self.len, len)
    }
}


//--- OctetsBuilder and EmptyBuilder

impl<const N: usize> OctetsBuilder for Array<N> {
    type Octets = Self;
    type AppendError = ShortBuf;

    fn try_append_slice(&mut self, slice: &[u8]) -> Result<(), ShortBuf> {
        let end = self.len + slice.len();
        if end > N {
            return Err(ShortBuf)
        }
        self.octets[self.len..end].copy_from_slice(slice);
        self.len = end;
        Ok(())
    }

    fn freeze(self) -> Self::Octets {
        self
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

impl<const N: usize, Source: AsRef<[u8]>> OctetsFrom<Source> for Array<N> {
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
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        struct Visitor<const N: usize>;

        impl<'de, const N: usize> serde::de::Visitor<'de> for Visitor<N> {
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

        deserializer.deserialize_bytes(Visitor)
    }
}

