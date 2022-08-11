//! Reading data from an octet sequence.
//!
//! Parsing is a little more complicated since encoded data may very well be
//! broken or ambiguously encoded. The helper type [`Parser`] wraps an octets
//! ref and allows to parse values from the octets.
//!
//! While there is a trait [`Parse`] for types that know how parse a value
//! from an octets sequence, this trait is often not needed and a simple
//! `parse` function will do. We use it primarily as an extension trait to
//! provide this `parse` function for built-in types.


use core::fmt;
use super::traits::OctetsRef;

//------------ Parser --------------------------------------------------------

/// A parser for sequentially extracting data from an octets sequence.
///
/// The parser wraps an [octets reference] and remembers the read position on
/// the referenced sequence. Methods allow reading out data and progressing
/// the position beyond processed data.
///
/// [octets reference]: trait.OctetsRef.html
#[derive(Clone, Copy, Debug)]
pub struct Parser<Ref> {
    /// The underlying octets reference.
    octets: Ref,

    /// The current position of the parser from the beginning of `octets`.
    pos: usize,

    /// The length of the octets sequence.
    ///
    /// This starts out as the length of the underlying sequence and is kept
    /// here to be able to temporarily limit the allowed length for
    /// `parse_blocks`.
    len: usize,
}

impl<Ref> Parser<Ref> {
    /// Creates a new parser atop a reference to an octet sequence.
    pub fn from_ref(octets: Ref) -> Self
    where
        Ref: AsRef<[u8]>,
    {
        Parser {
            pos: 0,
            len: octets.as_ref().len(),
            octets,
        }
    }

    /// Returns the wrapped reference to the underlying octets sequence.
    pub fn octets_ref(&self) -> Ref
    where
        Ref: Copy,
    {
        self.octets
    }

    /// Returns the current parse position as an index into the octets.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the length of the underlying octet sequence.
    ///
    /// This is _not_ the number of octets left for parsing. Use
    /// [`remaining`] for that.
    ///
    /// [`remaining`]: #method.remaining
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the underlying octets sequence is empty.
    ///
    /// This does _not_ return whether there are no more octets left to parse.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Parser<&'static [u8]> {
    /// Creates a new parser atop a static byte slice.
    ///
    /// This function is most useful for testing.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self::from_ref(slice)
    }
}

impl<Ref: AsRef<[u8]>> Parser<Ref> {
    /// Returns an octets slice of the underlying sequence.
    ///
    /// The slice covers the entire sequence, not just the remaining data. You
    /// can use [`peek`] for that.
    ///
    /// [`peek`]: #method.peek
    pub fn as_slice(&self) -> &[u8] {
        &self.octets.as_ref()[..self.len]
    }

    /// Returns a mutable octets slice of the underlying sequence.
    ///
    /// The slice covers the entire sequence, not just the remaining data.
    pub fn as_slice_mut(&mut self) -> &mut [u8]
    where
        Ref: AsMut<[u8]>,
    {
        &mut self.octets.as_mut()[..self.len]
    }

    /// Returns the number of remaining octets to parse.
    pub fn remaining(&self) -> usize {
        self.len - self.pos
    }

    /// Returns a slice for the next `len` octets.
    ///
    /// If less than `len` octets are left, returns an error.
    pub fn peek(&self, len: usize) -> Result<&[u8], ParseError> {
        self.check_len(len)?;
        Ok(&self.peek_all()[..len])
    }

    /// Returns a slice of the data left to parse.
    pub fn peek_all(&self) -> &[u8] {
        &self.octets.as_ref()[self.pos..]
    }

    /// Repositions the parser to the given index.
    ///
    /// It is okay to reposition anywhere within the sequence. However,
    /// if `pos` is larger than the length of the sequence, an error is
    /// returned.
    pub fn seek(&mut self, pos: usize) -> Result<(), ParseError> {
        if pos > self.len {
            Err(ParseError::ShortInput)
        } else {
            self.pos = pos;
            Ok(())
        }
    }

    /// Advances the parser‘s position by `len` octets.
    ///
    /// If this would take the parser beyond its end, an error is returned.
    pub fn advance(&mut self, len: usize) -> Result<(), ParseError> {
        if len > self.remaining() {
            Err(ParseError::ShortInput)
        } else {
            self.pos += len;
            Ok(())
        }
    }

    /// Advances to the end of the parser.
    pub fn advance_to_end(&mut self) {
        self.pos = self.len
    }

    /// Checks that there are `len` octets left to parse.
    ///
    /// If there aren’t, returns an error.
    pub fn check_len(&self, len: usize) -> Result<(), ParseError> {
        if self.remaining() < len {
            Err(ParseError::ShortInput)
        } else {
            Ok(())
        }
    }
}

impl<Ref: AsRef<[u8]>> Parser<Ref> {
    /// Takes and returns the next `len` octets.
    ///
    /// Advances the parser by `len` octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_octets(
        &mut self,
        len: usize,
    ) -> Result<Ref::Range, ParseError>
    where
        Ref: OctetsRef,
    {
        let end = self.pos + len;
        if end > self.len {
            return Err(ParseError::ShortInput);
        }
        let res = self.octets.range(self.pos, end);
        self.pos = end;
        Ok(res)
    }

    /// Fills the provided buffer by taking octets from the parser.
    ///
    /// Copies as many octets as the buffer is long from the parser into the
    /// buffer and advances the parser by that many octets.
    ///
    /// If there aren’t enough octets left in the parser to fill the buffer
    /// completely, returns an error and leaves the parser untouched.
    pub fn parse_buf(&mut self, buf: &mut [u8]) -> Result<(), ParseError> {
        let pos = self.pos;
        self.advance(buf.len())?;
        buf.copy_from_slice(&self.octets.as_ref()[pos..self.pos]);
        Ok(())
    }

    /// Takes an `i8` from the beginning of the parser.
    ///
    /// Advances the parser by one octet. If there aren’t enough octets left,
    /// leaves the parser untouched and returns an error instead.
    pub fn parse_i8(&mut self) -> Result<i8, ParseError> {
        let res = self.peek(1)?[0] as i8;
        self.pos += 1;
        Ok(res)
    }

    /// Takes a `u8` from the beginning of the parser.
    ///
    /// Advances the parser by one octet. If there aren’t enough octets left,
    /// leaves the parser untouched and returns an error instead.
    pub fn parse_u8(&mut self) -> Result<u8, ParseError> {
        let res = self.peek(1)?[0];
        self.pos += 1;
        Ok(res)
    }

    /// Takes an `i16` from the beginning of the parser.
    ///
    /// The value is converted from network byte order into the system’s own
    /// byte order if necessary. The parser is advanced by two octets. If
    /// there aren’t enough octets left, leaves the parser untouched and
    /// returns an error instead.
    pub fn parse_i16(&mut self) -> Result<i16, ParseError> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(i16::from_be_bytes(res))
    }

    /// Takes a `u16` from the beginning of the parser.
    ///
    /// The value is converted from network byte order into the system’s own
    /// byte order if necessary. The parser is advanced by two ocetets. If
    /// there aren’t enough octets left, leaves the parser untouched and
    /// returns an error instead.
    pub fn parse_u16(&mut self) -> Result<u16, ParseError> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(u16::from_be_bytes(res))
    }

    /// Takes an `i32` from the beginning of the parser.
    ///
    /// The value is converted from network byte order into the system’s own
    /// byte order if necessary. The parser is advanced by four octets. If
    /// there aren’t enough octets left, leaves the parser untouched and
    /// returns an error instead.
    pub fn parse_i32(&mut self) -> Result<i32, ParseError> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(i32::from_be_bytes(res))
    }

    /// Takes a `u32` from the beginning of the parser.
    ///
    /// The value is converted from network byte order into the system’s own
    /// byte order if necessary. The parser is advanced by four octets. If
    /// there aren’t enough octets left, leaves the parser untouched and
    /// returns an error instead.
    pub fn parse_u32(&mut self) -> Result<u32, ParseError> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(u32::from_be_bytes(res))
    }

    /// Parses a given amount of octets through a closure.
    ///
    /// Parses a block of `limit` octets and moves the parser to the end of
    /// that block or, if less than `limit` octets are still available, to
    /// the end of the parser.
    ///
    /// The closure `op` will be allowed to parse up to `limit` octets. If it
    /// does so successfully or returns with a form error, the method returns
    /// its return value. If it returns with a short buffer error, the method
    /// returns a form error. If it returns successfully with less than
    /// `limit` octets parsed, returns a form error indicating trailing data.
    /// If the limit is larger than the remaining number of octets, returns a
    /// `ParseError::ShortInput`.
    ///
    //  XXX NEEDS TESTS!!!
    pub fn parse_block<F, U>(
        &mut self,
        limit: usize,
        op: F,
    ) -> Result<U, ParseError>
    where
        F: FnOnce(&mut Self) -> Result<U, ParseError>,
    {
        let end = self.pos + limit;
        if end > self.len {
            self.advance_to_end();
            return Err(ParseError::ShortInput);
        }
        let len = self.len;
        self.len = end;
        let res = op(self);
        self.len = len;
        let res = if self.pos != end {
            Err(ParseError::Form(FormError::new("trailing data in field")))
        } else if let Err(ParseError::ShortInput) = res {
            Err(ParseError::Form(FormError::new("short field")))
        } else {
            res
        };
        self.pos = end;
        res
    }
}

//------------ Parse ------------------------------------------------------

/// A type that can extract a value from a parser.
///
/// The trait is a companion to [`Parser<Ref>`]: it allows a type to use a
/// parser to create a value of itself. Because types may be generic over
/// octets types, the trait is generic over the octets reference of the
/// parser in question. Implementations should use minimal trait bounds
/// matching the parser methods they use.
///
/// For types that are generic over an octets sequence, the reference type
/// should be tied to the type’s own type argument. This will avoid having
/// to provide type annotations when simply calling `Parse::parse` for the
/// type. Typically this will happen via `OctetsRef::Range`. For instance,
/// a type `Foo<Octets>` should provide:
///
/// ```ignore
/// impl<Ref: OctetsRef> Parse<Ref> for Foo<Ref::Range> {
///     // etc.
/// }
/// ```
///
/// [`Parser<Ref>`]: struct.Parser.html
pub trait Parse<Ref>: Sized {
    /// Extracts a value from the beginning of `parser`.
    ///
    /// If parsing fails and an error is returned, the parser’s position
    /// should be considered to be undefined. If it is supposed to be reused
    /// in this case, you should store the position before attempting to parse
    /// and seek to that position again before continuing.
    fn parse(parser: &mut Parser<Ref>) -> Result<Self, ParseError>;

    /// Skips over a value of this type at the beginning of `parser`.
    ///
    /// This function is the same as `parse` but doesn’t return the result.
    /// It can be used to check if the content of `parser` is correct or to
    /// skip over unneeded parts of the parser.
    fn skip(parser: &mut Parser<Ref>) -> Result<(), ParseError>;
}

impl<T: AsRef<[u8]>> Parse<T> for i8 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_i8().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(1).map_err(Into::into)
    }
}

impl<T: AsRef<[u8]>> Parse<T> for u8 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_u8().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(1).map_err(Into::into)
    }
}

impl<T: AsRef<[u8]>> Parse<T> for i16 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_i16().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(2).map_err(Into::into)
    }
}

impl<T: AsRef<[u8]>> Parse<T> for u16 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_u16().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(2).map_err(Into::into)
    }
}

impl<T: AsRef<[u8]>> Parse<T> for i32 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_i32().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(4).map_err(Into::into)
    }
}

impl<T: AsRef<[u8]>> Parse<T> for u32 {
    fn parse(parser: &mut Parser<T>) -> Result<Self, ParseError> {
        parser.parse_u32().map_err(Into::into)
    }

    fn skip(parser: &mut Parser<T>) -> Result<(), ParseError> {
        parser.advance(4).map_err(Into::into)
    }
}


//--------- ParseError -------------------------------------------------------

/// An error happened while parsing data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// An attempt was made to go beyond the end of the parser.
    ShortInput,

    /// A formatting error occurred.
    Form(FormError),
}

impl ParseError {
    /// Creates a new parse error as a form error with the given message.
    pub fn form_error(msg: &'static str) -> Self {
        FormError::new(msg).into()
    }
}

//--- From

impl From<FormError> for ParseError {
    fn from(err: FormError) -> Self {
        ParseError::Form(err)
    }
}

//--- Display and Error

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::ShortInput => f.write_str("unexpected end of input"),
            ParseError::Form(ref err) => err.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}


//------------ FormError -----------------------------------------------------

/// A formatting error occured.
///
/// This is a generic error for all kinds of error cases that result in data
/// not being accepted. For diagnostics, the error is being given a static
/// string describing the error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormError(&'static str);

impl FormError {
    /// Creates a new form error value with the given diagnostics string.
    pub fn new(msg: &'static str) -> Self {
        FormError(msg)
    }
}

//--- Display and Error

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FormError {}

