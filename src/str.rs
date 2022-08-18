//! Strings atop octet sequences.
//!
//! This module provides the type `Str<Octets>` that guarantees the same
//! invariants – namely that the content is an UTF-8 encoded string – as
//! the standard library’s `str` and `String` types but atop a generic
//! octet sequence.

use core::{borrow, cmp, fmt, hash, ops, str};
use core::convert::Infallible;
use crate::traits::{EmptyBuilder, OctetsBuilder, Truncate};


//------------ Str -----------------------------------------------------------

/// A fixed length UTF-8 encoded string atop an octet sequence.
#[derive(Clone, Default)]
pub struct Str<Octets>(Octets);

impl<Octets> Str<Octets> {
    /// Converts a sequence of octets into a string.
    pub fn from_utf8(octets: Octets) -> Result<Self, FromUtf8Error<Octets>>
    where Octets: AsRef<[u8]> {
        if let Err(error) = str::from_utf8(octets.as_ref()) {
            Err(FromUtf8Error { octets, error })
        }
        else {
            Ok(Self(octets))
        }
    }

    /// Converts a sequence of octets into a string without checking.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the contents of `octets` is a
    /// correctly encoded UTF-8 string.
    pub unsafe fn from_utf8_unchecked(octets: Octets) -> Self {
        Self(octets)
    }

    /// Converts the string into its raw octets.
    pub fn into_octets(self) -> Octets {
        self.0
    }

    /// Returns the string as a string slice.
    pub fn as_str(&self) -> &str
    where Octets: AsRef<[u8]> {
        unsafe { str::from_utf8_unchecked(self.0.as_ref()) }
    }

    /// Returns the string as a mutable string slice.
    pub fn as_str_mut(&mut self) -> &mut str
    where Octets: AsMut<[u8]> {
        unsafe { str::from_utf8_unchecked_mut(self.0.as_mut()) }
    }

    /// Returns a reference to the underlying octets sequence.
    pub fn as_octets(&self) -> &Octets {
        &self.0
    }

    /// Returns a mutable reference to the underlying octets sequence.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the content of the octets sequence is
    /// valid UTF-8 before the borrow ends.
    pub unsafe fn as_octets_mut(&mut self) -> &mut Octets {
        &mut self.0
    }

    /// Returns the string’s octets as a slice.
    pub fn as_slice(&self) -> &[u8]
    where Octets: AsRef<[u8]> {
        self.0.as_ref()
    }

    /// Returns a mutable slice of the string’s octets.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the content of the slice is
    /// valid UTF-8 before the borrow ends.
    pub unsafe fn as_slice_mut(&mut self) -> &mut [u8]
    where Octets: AsMut<[u8]> {
        self.0.as_mut()
    }

    /// Returns the length of the string in octets.
    pub fn len(&self) -> usize
    where Octets: AsRef<[u8]> {
        self.0.as_ref().len()
    }

    /// Returns whether the string is empty.
    pub fn is_empty(&self) -> bool
    where Octets: AsRef<[u8]> {
        self.0.as_ref().is_empty()
    }
}


//--- Deref, DerefMut, AsRef, AsMut, Borrow, BorrowMut

impl<Octets: AsRef<[u8]>> ops::Deref for Str<Octets> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]> + AsMut<[u8]>> ops::DerefMut for Str<Octets> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl<Octets: AsRef<[u8]>> AsRef<str> for Str<Octets>{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]>> AsRef<[u8]> for Str<Octets>{
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<Octets: AsMut<[u8]>> AsMut<str> for Str<Octets> {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

impl<Octets: AsRef<[u8]>> borrow::Borrow<str> for Str<Octets>{
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]>> borrow::Borrow<[u8]> for Str<Octets>{
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<Octets> borrow::BorrowMut<str> for Str<Octets> 
where Octets: AsRef<[u8]> +  AsMut<[u8]> {
    fn borrow_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

//--- Debug and Display

impl<Octets: AsRef<[u8]>> fmt::Debug for Str<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<Octets: AsRef<[u8]>> fmt::Display for Str<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

//--- PartialEq and Eq

impl<Octets, Other> PartialEq<Other> for Str<Octets>
where
    Octets: AsRef<[u8]>,
    Other: AsRef<str>,
{
    fn eq(&self, other: &Other) -> bool {
        self.as_str().eq(other.as_ref())
    }
}

impl<Octets: AsRef<[u8]>> Eq for Str<Octets> { }

//--- Hash

impl<Octets: AsRef<[u8]>> hash::Hash for Str<Octets> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

//--- PartialOrd and Ord

impl<Octets, Other> PartialOrd<Other> for Str<Octets>
where
    Octets: AsRef<[u8]>,
    Other: AsRef<str>,
{
    fn partial_cmp(&self, other: &Other) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_ref())
    }
}

impl<Octets: AsRef<[u8]>> Ord for Str<Octets> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}


//------------ StrBuilder ----------------------------------------------------

/// A growable, UTF-8 encoded string atop an octets builder.
pub struct StrBuilder<Octets>(Octets);

impl<Octets> StrBuilder<Octets> {
    /// Creates a new, empty string builder.
    pub fn new() -> Self
    where Octets: EmptyBuilder {
        StrBuilder(Octets::empty())
    }

    /// Creates a new, empty string builder with a given minimum capacity.
    pub fn with_capacity(capacity: usize) -> Self
    where Octets: EmptyBuilder {
        StrBuilder(Octets::with_capacity(capacity))
    }

    /// Creates a new string builder from an octets builder.
    ///
    /// The function expects the contents of the octets builder to contain
    /// a sequence of UTF-8 encoded characters.
    pub fn from_utf8(octets: Octets) -> Result<Self, FromUtf8Error<Octets>>
    where Octets: AsRef<[u8]> {
        if let Err(error) = str::from_utf8(octets.as_ref()) {
            Err(FromUtf8Error { octets, error })
        }
        else {
            Ok(Self(octets))
        }
    }

    /// Converts on octets builder into a string builder.
    ///
    /// If the octets builder contains invalid octets, they are replaced with
    /// `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// If the content is UTF-8 encoded, it will remain unchanged. Otherwise,
    /// a new builder is created and the passed builder is dropped.
    pub fn try_from_utf8_lossy(
        octets: Octets
    ) -> Result<Self, <Octets as OctetsBuilder>::AppendError>
    where Octets: AsRef<[u8]> + OctetsBuilder + EmptyBuilder {
        const REPLACEMENT_CHAR: &[u8] = &[239, 191, 189];

        let mut err = match str::from_utf8(octets.as_ref()) {
            Ok(_) => return Ok(Self(octets)),
            Err(err) => err,
        };
        let mut octets = octets.as_ref();
        let mut res = Octets::with_capacity(octets.len());
        while !octets.is_empty() {
            if err.valid_up_to() > 0 {
                res.try_append_slice(&octets[..err.valid_up_to()])?;
            }
            res.try_append_slice(REPLACEMENT_CHAR)?;
            octets = match err.error_len() {
                Some(len) => &octets[err.valid_up_to() + len ..],
                None => b""
            };
            err = match str::from_utf8(octets) {
                Ok(_) => {
                    res.try_append_slice(octets)?;
                    break;
                }
                Err(err) => err,
            };
        }
        Ok(Self(res))
    }

    /// Converts an octets builder into a string builder.
    ///
    /// This is a simpler version of
    /// [try_from_utf8_lossy][Self::try_from_utf8_lossy]
    /// for infallible octets builders.
    pub fn from_utf8_lossy(octets: Octets) -> Self
    where
        Octets: AsRef<[u8]> + OctetsBuilder + EmptyBuilder,
        <Octets as OctetsBuilder>::AppendError: Into<Infallible>
    {
        match Self::try_from_utf8_lossy(octets) {
            Ok(ok) => ok,
            Err(_) => unreachable!(),
        }
    }

    /// Converts an octets builder into a string builder without checking.
    ///
    /// For the safe versions, see [from_utf8][Self::from_utf8],
    /// [try_from_utf8_lossy][Self::try_from_utf8_lossy] and
    /// [from_utf8_lossy][Self::from_utf8_lossy].
    ///
    /// # Safety
    ///
    /// The caller must ensure that `octets` contains data that is a correctly
    /// UTF-8 encoded string. It may be empty.
    pub unsafe fn from_utf8_unchecked(octets: Octets) -> Self {
        Self(octets)
    }

    /// Converts the string builder into the underlying octets builder.
    pub fn into_octets_builder(self) -> Octets {
        self.0
    }

    /// Converts the string builder into the final str.
    pub fn freeze(self) -> Str<Octets::Octets>
    where Octets: OctetsBuilder {
        Str(self.0.freeze())
    }

    /// Returns a slice of the already assembled string.
    pub fn as_str(&self) -> &str
    where Octets: AsRef<[u8]> {
        unsafe { str::from_utf8_unchecked(self.0.as_ref()) }
    }

    /// Returns a mutable slice of the already assembled string.
    pub fn as_str_mut(&mut self) -> &mut str
    where Octets: AsMut<[u8]> {
        unsafe { str::from_utf8_unchecked_mut(self.0.as_mut()) }
    }

    /// Returns the string’s octets as a slice.
    pub fn as_slice(&self) -> &[u8]
    where Octets: AsRef<[u8]> {
        self.0.as_ref()
    }

    /// Returns the length of the string in octets.
    pub fn len(&self) -> usize
    where Octets: AsRef<[u8]> {
        self.0.as_ref().len()
    }

    /// Returns whether the string is empty.
    pub fn is_empty(&self) -> bool
    where Octets: AsRef<[u8]> {
        self.0.as_ref().is_empty()
    }

    /// Appends a given string slice onto the end of this builder.
    pub fn try_push_str(
        &mut self, s: &str,
    ) -> Result<(), Octets::AppendError>
    where Octets: OctetsBuilder {
        self.0.try_append_slice(s.as_bytes())
    }

    /// Appends a given string slice onto the end of this builder.
    pub fn push_str(&mut self, s: &str)
    where
        Octets: OctetsBuilder,
        Octets::AppendError: Into<Infallible>,
    {
        self.0.append_slice(s.as_bytes())
    }

    /// Appends the given character to the end of the builder.
    pub fn try_push(&mut self, ch: char) -> Result<(), Octets::AppendError>
    where Octets: OctetsBuilder {
        let mut buf = [0u8; 4];
        self.0.try_append_slice(ch.encode_utf8(&mut buf).as_bytes())
    }

    /// Appends the given character to the end of the builder.
    pub fn push(&mut self, ch: char)
    where
        Octets: OctetsBuilder,
        Octets::AppendError: Into<Infallible>,
    {
        let mut buf = [0u8; 4];
        self.0.append_slice(ch.encode_utf8(&mut buf).as_bytes())
    }

    /// Truncates the builder, keeping the first `new_len` octets.
    ///
    /// # Panics
    ///
    /// The method panics if `new_len` does not lie on a `char` boundary.
    pub fn truncate(&mut self, new_len: usize)
    where Octets: AsRef<[u8]> + Truncate {
        if new_len < self.len() {
            assert!(self.as_str().is_char_boundary(new_len));
            self.0.truncate(new_len)
        }
    }

    /// Clears the builder into an empty builder.
    pub fn clear(&mut self)
    where Octets: AsRef<[u8]> + Truncate {
        self.truncate(0)
    }

    /// Removes the last character from the builder and returns it.
    ///
    /// Returns `None` if the builder is empty.
    pub fn pop(&mut self) -> Option<char>
    where Octets: AsRef<[u8]> + Truncate {
        let ch = self.as_str().chars().rev().next()?;
        self.truncate(self.len() - ch.len_utf8());
        Some(ch)
    }
}


//-- Default

impl<Octets: EmptyBuilder> Default for StrBuilder<Octets> {
    fn default() -> Self {
        Self::new()
    }
}


//--- Deref, DerefMut, AsRef, AsMut, Borrow, BorrowMut

impl<Octets: AsRef<[u8]>> ops::Deref for StrBuilder<Octets> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]> + AsMut<[u8]>> ops::DerefMut for StrBuilder<Octets> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl<Octets: AsRef<[u8]>> AsRef<str> for StrBuilder<Octets>{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]>> AsRef<[u8]> for StrBuilder<Octets>{
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<Octets: AsMut<[u8]>> AsMut<str> for StrBuilder<Octets> {
    fn as_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

impl<Octets: AsRef<[u8]>> borrow::Borrow<str> for StrBuilder<Octets>{
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<Octets: AsRef<[u8]>> borrow::Borrow<[u8]> for StrBuilder<Octets>{
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<Octets> borrow::BorrowMut<str> for StrBuilder<Octets> 
where Octets: AsRef<[u8]> +  AsMut<[u8]> {
    fn borrow_mut(&mut self) -> &mut str {
        self.as_str_mut()
    }
}

//--- Debug and Display

impl<Octets: AsRef<[u8]>> fmt::Debug for StrBuilder<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<Octets: AsRef<[u8]>> fmt::Display for StrBuilder<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

//--- PartialEq and Eq

impl<Octets, Other> PartialEq<Other> for StrBuilder<Octets>
where
    Octets: AsRef<[u8]>,
    Other: AsRef<str>,
{
    fn eq(&self, other: &Other) -> bool {
        self.as_str().eq(other.as_ref())
    }
}

impl<Octets: AsRef<[u8]>> Eq for StrBuilder<Octets> { }

//--- Hash

impl<Octets: AsRef<[u8]>> hash::Hash for StrBuilder<Octets> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

//--- PartialOrd and Ord

impl<Octets, Other> PartialOrd<Other> for StrBuilder<Octets>
where
    Octets: AsRef<[u8]>,
    Other: AsRef<str>,
{
    fn partial_cmp(&self, other: &Other) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_ref())
    }
}

impl<Octets: AsRef<[u8]>> Ord for StrBuilder<Octets> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}


//============ Error Types ===================================================

//------------ FromUtf8Error -------------------------------------------------

/// An error happened when converting octets into a string.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FromUtf8Error<Octets> {
    octets: Octets,
    error: str::Utf8Error,
}

impl<Octets> FromUtf8Error<Octets> {
    /// Returns an octets slice of the data that failed to convert.
    pub fn as_slice(&self) -> &[u8]
    where Octets: AsRef<[u8]> {
        self.octets.as_ref()
    }

    /// Returns the octets sequence that failed to convert.
    pub fn into_octets(self) -> Octets {
        self.octets
    }

    /// Returns the reason for the conversion error.
    pub fn utf8_error(&self) -> str::Utf8Error {
        self.error
    }
}

impl<Octets> fmt::Debug for FromUtf8Error<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FromUtf8Error")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl<Octets> fmt::Display for FromUtf8Error<Octets> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

#[cfg(feature = "std")]
impl<Octets> std::error::Error for FromUtf8Error<Octets> {}


//============ Testing =======================================================

#[cfg(test)]
mod test {
    use super::*;

    // Most of the test cases herein have been borrowed from the test cases
    // of the Rust standard library.

    #[test]
    #[cfg(feature = "std")]
    fn from_utf8_lossy() {
        fn check(src: impl AsRef<[u8]>) {
            assert_eq!(
                StrBuilder::from_utf8_lossy(std::vec::Vec::from(src.as_ref())),
                std::string::String::from_utf8_lossy(src.as_ref())
            );
        }

        check(b"hello");
        check("ศไทย中华Việt Nam");
        check(b"Hello\xC2 There\xFF Goodbye");
        check(b"Hello\xC0\x80 There\xE6\x83 Goodbye");
        check(b"\xF5foo\xF5\x80bar");
        check(b"\xF1foo\xF1\x80bar\xF1\x80\x80baz");
        check(b"\xF4foo\xF4\x80bar\xF4\xBFbaz");
        check(b"\xF0\x80\x80\x80foo\xF0\x90\x80\x80bar");
        check(b"\xED\xA0\x80foo\xED\xBF\xBFbar");
    }

    #[test]
    #[cfg(feature = "std")]
    fn push_str() {
        let mut s = StrBuilder::<std::vec::Vec<u8>>::new();
        s.push_str("");
        assert_eq!(&s[0..], "");
        s.push_str("abc");
        assert_eq!(&s[0..], "abc");
        s.push_str("ประเทศไทย中华Việt Nam");
        assert_eq!(&s[0..], "abcประเทศไทย中华Việt Nam");
    }

    #[test]
    #[cfg(feature = "std")]
    fn push() {
        let mut data = StrBuilder::from_utf8(
            std::vec::Vec::from("ประเทศไทย中".as_bytes())
        ).unwrap();
        data.push('华');
        data.push('b'); // 1 byte
        data.push('¢'); // 2 byte
        data.push('€'); // 3 byte
        data.push('𤭢'); // 4 byte
        assert_eq!(data, "ประเทศไทย中华b¢€𤭢");
    }

    #[test]
    #[cfg(feature = "std")]
    fn pop() {
        let mut data = StrBuilder::from_utf8(
            std::vec::Vec::from("ประเทศไทย中华b¢€𤭢".as_bytes())
        ).unwrap();
        assert_eq!(data.pop().unwrap(), '𤭢'); // 4 bytes
        assert_eq!(data.pop().unwrap(), '€'); // 3 bytes
        assert_eq!(data.pop().unwrap(), '¢'); // 2 bytes
        assert_eq!(data.pop().unwrap(), 'b'); // 1 bytes
        assert_eq!(data.pop().unwrap(), '华');
        assert_eq!(data, "ประเทศไทย中");
    }
}

