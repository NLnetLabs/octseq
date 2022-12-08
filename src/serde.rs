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


//------------ SerializeOctets -----------------------------------------------

pub trait SerializeOctets {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error>;
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


//------------ DeserializeOctets ---------------------------------------------

pub trait DeserializeOctets<'de>: Sized {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error>;
}

impl<'de> DeserializeOctets<'de> for &'de [u8] {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = &'de [u8];

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an octet sequence")
            }

            fn visit_borrowed_bytes<E: serde::de::Error>(
                self, value: &'de [u8]
            ) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        deserializer.deserialize_bytes(Visitor)
    }
}

#[cfg(feature = "std")]
impl<'de> DeserializeOctets<'de> for std::borrow::Cow<'de, [u8]> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        Ok(std::borrow::Cow::Borrowed(
            DeserializeOctets::deserialize_octets(deserializer)?
        ))
    }
}

#[cfg(feature = "std")]
impl<'de> DeserializeOctets<'de> for std::vec::Vec<u8> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = std::vec::Vec<u8>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an octet sequence")
            }

            fn visit_byte_buf<E: serde::de::Error>(
                self, value: std::vec::Vec<u8>
            ) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        deserializer.deserialize_byte_buf(Visitor)
    }
}

#[cfg(feature = "bytes")]
impl<'de> DeserializeOctets<'de> for bytes::Bytes {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        std::vec::Vec::deserialize_octets(deserializer).map(Into::into)
    }
}

#[cfg(feature = "smallvec")]
impl<'de, A> DeserializeOctets<'de> for smallvec::SmallVec<A>
where A: smallvec::Array<Item = u8> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        std::vec::Vec::deserialize_octets(deserializer).map(Into::into)
    }
}

#[cfg(feature = "heapless")]
impl<'de, const N: usize> DeserializeOctets<'de> for heapless::Vec<u8, N> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        HeaplessVecVisitor::new().deserialize(deserializer)
    }
}

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

