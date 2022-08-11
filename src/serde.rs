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

#[cfg(feature = "serde")]
pub trait SerializeOctets {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error>;
}

#[cfg(feature = "serde")]
impl<'a> SerializeOctets for &'a [u8] {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self)
    }
}

#[cfg(all(feature = "std", feature = "serde"))]
impl<'a> SerializeOctets for std::borrow::Cow<'a, [u8]> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(all(feature = "std", feature = "serde"))]
impl SerializeOctets for std::vec::Vec<u8> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(all(feature = "bytes", feature = "serde"))]
impl SerializeOctets for bytes::Bytes {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(all(feature = "smallvec", feature = "serde"))]
impl<A> SerializeOctets for smallvec::SmallVec<A>
where A: smallvec::Array<Item = u8> {
    fn serialize_octets<S: serde::Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_ref())
    }
}


//------------ DeserializeOctets ---------------------------------------------

#[cfg(feature = "serde")]
pub trait DeserializeOctets<'de>: Sized {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error>;
}

#[cfg(feature = "serde")]
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

#[cfg(all(feature = "std", feature = "serde"))]
impl<'de> DeserializeOctets<'de> for std::borrow::Cow<'de, [u8]> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        Ok(std::borrow::Cow::Borrowed(
            DeserializeOctets::deserialize_octets(deserializer)?
        ))
    }
}

#[cfg(all(feature = "std", feature = "serde"))]
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

#[cfg(all(feature = "bytes", feature = "serde"))]
impl<'de> DeserializeOctets<'de> for bytes::Bytes {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        std::vec::Vec::deserialize_octets(deserializer).map(Into::into)
    }
}

#[cfg(all(feature = "smallvec", feature = "serde"))]
impl<'de, A> DeserializeOctets<'de> for smallvec::SmallVec<A>
where A: smallvec::Array<Item = u8> {
    fn deserialize_octets<D: serde::Deserializer<'de>>(
        deserializer: D
    ) -> Result<Self, D::Error> {
        std::vec::Vec::deserialize_octets(deserializer).map(Into::into)
    }
}


