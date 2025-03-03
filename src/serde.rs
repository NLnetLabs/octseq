//! Serde support.
//!
//! [Serde](https://serde.rs/) supports native serialization of octets
//! sequences. However, because of missing specialization, it has to
//! serialize the octets slices and vec as literal sequences of `u8`s. In
//! order to allow octets sequences their own native deserialization, the
//! crate defines the trait [`DeserializeOctets`] if built with the `serde`
//! feature enabled.
#![cfg(feature = "serde")]

use core::fmt;
use core::marker::PhantomData;
use serde::de::Visitor;

pub fn serialize<Octs, S>(
    octs: &Octs,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    Octs: AsRef<[u8]> + ?Sized,
{
    serializer.serialize_bytes(octs.as_ref())
}

pub fn deserialize<'de, Octs, D>(deserializer: D) -> Result<Octs, D::Error>
where
    D: serde::Deserializer<'de>,
    Octs: DeserializeOctets<'de>,
{
    Octs::deserialize_octets(deserializer)
}

//------------ AsSerializedOctets --------------------------------------------

/// A wrapper forcing a value to serialize through its octets.
///
/// This type can be used where a `Serialize` value is required.
pub struct AsSerializedOctets<'a>(&'a [u8]);

impl<'a> serde::Serialize for AsSerializedOctets<'a> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serialize(&self.0, serializer)
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> From<&'a T> for AsSerializedOctets<'a> {
    fn from(value: &'a T) -> Self {
        AsSerializedOctets(value.as_ref())
    }
}

//------------ DeserializeOctets ---------------------------------------------

pub trait DeserializeOctets<'de>: Sized {
    type Visitor: Visitor<'de, Value = Self>;

    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        Self::deserialize_with_visitor(deserializer, Self::visitor())
    }

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
        Self::deserialize_with_visitor(deserializer, Self::visitor())
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
