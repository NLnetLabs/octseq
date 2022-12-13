//! Serde support.
//!
//! [Serde](https://serde.rs/) supports native serialization of octets
//! sequences. However, because of missing specialization, it has to
//! serialize the octets slices and vec as literal sequences of `u8`s. In
//! order to allow octets sequences their own native serialization, the crate
//! defines two traits [`SerializeOctets`] and [`DeserializeOctets`] if
//! built with the `serde` feature enabled.
#![cfg(feature = "serde")]

use core::fmt;
use core::marker::PhantomData;
use serde::de::Visitor;


//------------ SerializeOctets -----------------------------------------------

pub trait SerializeOctets {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error>;

    fn as_serialized_octets(&self) -> AsSerializedOctets<Self> {
        AsSerializedOctets(self)
    }
}

impl<'a> SerializeOctets for &'a [u8] {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self)
    }
}

#[cfg(feature = "std")]
impl<'a> SerializeOctets for std::borrow::Cow<'a, [u8]> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "std")]
impl SerializeOctets for std::vec::Vec<u8> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "bytes")]
impl SerializeOctets for bytes::Bytes {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "smallvec")]
impl<A> SerializeOctets for smallvec::SmallVec<A>
where A: smallvec::Array<Item = u8> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "heapless")]
impl<const N: usize> SerializeOctets for heapless::Vec<u8, N> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}


//------------ AsSerializedOctets --------------------------------------------

/// A wrapper forcing a value to serialize through its octets.
///
/// This type can be used where a `Serialize` value is required.
pub struct AsSerializedOctets<'a, T: ?Sized>(&'a T);

impl<'a, T: SerializeOctets> serde::Serialize for AsSerializedOctets<'a, T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_octets(serializer)
    }
}


//------------ DeserializeOctets ---------------------------------------------

pub trait DeserializeOctets<'de>: Sized {
    type Visitor: Visitor<'de, Value = Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error>;

    fn deserialize_with_visitor<
        D: serde::Deserializer<'de>,
        V: serde::de::Visitor<'de>,
    >(
        deserializer: D,
        visitor: V,
    ) -> Result<V::Value, D::Error>;

    fn visitor() -> Self::Visitor;
}

impl<'de> DeserializeOctets<'de> for &'de [u8] {
    type Visitor = BorrowedVisitor<Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        deserializer.deserialize_bytes(visitor)
    }

    fn visitor() -> Self::Visitor {
        BorrowedVisitor::new()
    }
}

#[cfg(feature = "std")]
impl<'de> DeserializeOctets<'de> for std::borrow::Cow<'de, [u8]> {
    type Visitor = BorrowedVisitor<Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        deserializer.deserialize_bytes(visitor)
    }

    fn visitor() -> Self::Visitor {
        BorrowedVisitor::new()
    }
}

#[cfg(feature = "std")]
impl<'de> DeserializeOctets<'de> for std::vec::Vec<u8> {
    type Visitor = BufVisitor<Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        BufVisitor::new()
    }
}

#[cfg(feature = "bytes")]
impl<'de> DeserializeOctets<'de> for bytes::Bytes {
    type Visitor = BufVisitor<Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        BufVisitor::new()
    }
}

#[cfg(feature = "smallvec")]
impl<'de, A> DeserializeOctets<'de> for smallvec::SmallVec<A>
where
    A: smallvec::Array<Item = u8>,
{
    type Visitor = BufVisitor<Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        BufVisitor::new()
    }
}

#[cfg(feature = "heapless")]
impl<'de, const N: usize> DeserializeOctets<'de> for heapless::Vec<u8, N> {
    type Visitor = HeaplessVecVisitor<N>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
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
        HeaplessVecVisitor::new()
    }
}

//------------ BorrowedVisitor -------------------------------------------

pub struct BorrowedVisitor<T>(PhantomData<T>);

impl<T> BorrowedVisitor<T> {
    fn new() -> Self {
        BorrowedVisitor(PhantomData)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<T, D::Error>
    where
        T: From<&'de [u8]>,
    {
        deserializer.deserialize_bytes(self)
    }
}

impl<'de, T> serde::de::Visitor<'de> for BorrowedVisitor<T>
where
    T: From<&'de [u8]>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an octet sequence")
    }

    fn visit_borrowed_bytes<E: serde::de::Error>(
        self,
        value: &'de [u8],
    ) -> Result<Self::Value, E> {
        Ok(value.into())
    }
}

//------------ BufVisitor ------------------------------------------------

#[cfg(feature = "std")]
pub struct BufVisitor<T>(PhantomData<T>);

#[cfg(feature = "std")]
impl<T> BufVisitor<T> {
    fn new() -> Self {
        BufVisitor(PhantomData)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<T, D::Error>
    where
        T: From<std::vec::Vec<u8>>,
    {
        deserializer.deserialize_byte_buf(self)
    }
}

#[cfg(feature = "std")]
impl<'de, T> serde::de::Visitor<'de> for BufVisitor<T>
where
    T: From<std::vec::Vec<u8>>,
{
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an octet sequence")
    }

    fn visit_borrowed_bytes<E: serde::de::Error>(
        self,
        value: &'de [u8],
    ) -> Result<Self::Value, E> {
        Ok(std::vec::Vec::from(value).into())
    }

    fn visit_byte_buf<E: serde::de::Error>(
        self,
        value: std::vec::Vec<u8>,
    ) -> Result<Self::Value, E> {
        Ok(value.into())
    }
}


//------------ HeaplessVisitor -----------------------------------------------

#[cfg(feature = "heapless")]
pub struct HeaplessVecVisitor<const N: usize>;

#[cfg(feature = "heapless")]
impl<const N: usize> HeaplessVecVisitor<N> {
    fn new() -> Self {
        Self
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<heapless::Vec<u8, N>, D::Error> {
        deserializer.deserialize_byte_buf(self)
    }
}

#[cfg(feature = "heapless")]
impl<'de, const N: usize> serde::de::Visitor<'de> for HeaplessVecVisitor<N> {
    type Value = heapless::Vec<u8, N>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "an octet sequence of length {} of shorter",
            N
        ))
    }

    fn visit_bytes<E: serde::de::Error>(
        self,
        value: &[u8],
    ) -> Result<Self::Value, E> {
        if value.len() > N {
            return Err(E::invalid_length(value.len(), &self));
        }

        Ok(heapless::Vec::from_iter(value.iter().copied()))
    }
}

