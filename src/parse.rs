//! Reading data from an octet sequence.
//!
//! Parsing is a little more complicated since encoded data may very well be
//! broken or ambiguously encoded. The helper type [`Parser`] wraps an octets
//! ref and allows to parse values from the octets.

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
    pub fn peek(&self, len: usize) -> Result<&[u8], ShortInput> {
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
    pub fn seek(&mut self, pos: usize) -> Result<(), ShortInput> {
        if pos > self.len {
            Err(ShortInput(()))
        } else {
            self.pos = pos;
            Ok(())
        }
    }

    /// Advances the parser‘s position by `len` octets.
    ///
    /// If this would take the parser beyond its end, an error is returned.
    pub fn advance(&mut self, len: usize) -> Result<(), ShortInput> {
        if len > self.remaining() {
            Err(ShortInput(()))
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
    pub fn check_len(&self, len: usize) -> Result<(), ShortInput> {
        if self.remaining() < len {
            Err(ShortInput(()))
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
    ) -> Result<Ref::Range, ShortInput>
    where
        Ref: OctetsRef,
    {
        let end = self.pos + len;
        if end > self.len {
            return Err(ShortInput(()));
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
    pub fn parse_buf(&mut self, buf: &mut [u8]) -> Result<(), ShortInput> {
        let pos = self.pos;
        self.advance(buf.len())?;
        buf.copy_from_slice(&self.octets.as_ref()[pos..self.pos]);
        Ok(())
    }

    /// Takes an `i8` from the beginning of the parser.
    ///
    /// Advances the parser by one octet. If there aren’t enough octets left,
    /// leaves the parser untouched and returns an error instead.
    pub fn parse_i8(&mut self) -> Result<i8, ShortInput> {
        let res = self.peek(1)?[0] as i8;
        self.pos += 1;
        Ok(res)
    }

    /// Takes a `u8` from the beginning of the parser.
    ///
    /// Advances the parser by one octet. If there aren’t enough octets left,
    /// leaves the parser untouched and returns an error instead.
    pub fn parse_u8(&mut self) -> Result<u8, ShortInput> {
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
    pub fn parse_i16(&mut self) -> Result<i16, ShortInput> {
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
    pub fn parse_u16(&mut self) -> Result<u16, ShortInput> {
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
    pub fn parse_i32(&mut self) -> Result<i32, ShortInput> {
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
    pub fn parse_u32(&mut self) -> Result<u32, ShortInput> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(u32::from_be_bytes(res))
    }
}


//--------- ShortInput -------------------------------------------------------

/// An attempt was made to go beyond the end of the parser.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShortInput(());

//--- Display and Error

impl fmt::Display for ShortInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("unexpected end of input")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ShortInput {}

