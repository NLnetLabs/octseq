//! Reading data from an octet sequence.
//!
//! Parsing is a little more complicated since encoded data may very well be
//! broken or ambiguously encoded. The helper type [`Parser`] wraps an octets
//! ref and allows to parse values from the octets.

use core::fmt;
use crate::octets::Octets;

//------------ Parser --------------------------------------------------------

/// A parser for sequentially extracting data from an octets sequence.
///
/// The parser wraps an [octets reference] and remembers the read position on
/// the referenced sequence. Methods allow reading out data and progressing
/// the position beyond processed data.
///
/// [octets reference]: trait.OctetsRef.html
#[derive(Debug)]
pub struct Parser<'a, Octs: ?Sized> {
    /// The underlying octets reference.
    octets: &'a Octs,

    /// The current position of the parser from the beginning of `octets`.
    pos: usize,

    /// The length of the octets sequence.
    ///
    /// This starts out as the length of the underlying sequence and is kept
    /// here to be able to temporarily limit the allowed length for
    /// `parse_blocks`.
    len: usize,
}

impl<'a, Octs: ?Sized> Parser<'a, Octs> {
    /// Creates a new parser atop a reference to an octet sequence.
    pub fn from_ref(octets: &'a Octs) -> Self
    where
        Octs: AsRef<[u8]>,
    {
        Parser {
            pos: 0,
            len: octets.as_ref().len(),
            octets,
        }
    }

    /// Returns the wrapped reference to the underlying octets sequence.
    pub fn octets_ref(&self) -> &'a Octs {
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

impl Parser<'static, [u8]> {
    /// Creates a new parser atop a static byte slice.
    ///
    /// This function is most useful for testing.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self::from_ref(slice)
    }
}

impl<'a, Octs: AsRef<[u8]> + ?Sized> Parser<'a, Octs> {
    /// Returns an octets slice of the underlying sequence.
    ///
    /// The slice covers the entire sequence, not just the remaining data. You
    /// can use [`peek`] for that.
    ///
    /// [`peek`]: #method.peek
    pub fn as_slice(&self) -> &[u8] {
        &self.octets.as_ref()[..self.len]
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

impl<'a, Octs: AsRef<[u8]> + ?Sized> Parser<'a, Octs> {
    /// Takes and returns the next `len` octets.
    ///
    /// Advances the parser by `len` octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_octets(
        &mut self,
        len: usize,
    ) -> Result<Octs::Range<'a>, ShortInput>
    where
        Octs: Octets,
    {
        let end = self.pos + len;
        if end > self.len {
            return Err(ShortInput(()));
        }
        let res = self.octets.range(self.pos..end);
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

    /// Takes as many octets as requested and returns a parser for them.
    ///
    /// If enough octets are remaining, the method clones `self`, limits
    /// its length to the requested number of octets, and returns it. The
    /// returned parser will be positioned at wherever `self` was positioned.
    /// The `self` parser will be advanced by the requested amount of octets.
    ///
    /// If there aren’t enough octets left in the parser to fill the buffer
    /// completely, returns an error and leaves the parser untouched.
    pub fn parse_parser(&mut self, len: usize) -> Result<Self, ShortInput> {
        self.check_len(len)?;
        let mut res = *self;
        res.len = res.pos + len;
        self.pos += len;
        Ok(res)
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
}

impl<'a, Octs: AsRef<[u8]> + ?Sized> Parser<'a, Octs> {
    /// Takes a big-endian `i16` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by two octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i16_be(&mut self) -> Result<i16, ShortInput> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(i16::from_be_bytes(res))
    }

    /// Takes a little-endian `i16` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by two octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i16_le(&mut self) -> Result<i16, ShortInput> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(i16::from_le_bytes(res))
    }

    /// Takes a big-endian `u16` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by two octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u16_be(&mut self) -> Result<u16, ShortInput> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(u16::from_be_bytes(res))
    }

    /// Takes a little-endian `u16` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by two octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u16_le(&mut self) -> Result<u16, ShortInput> {
        let mut res = [0; 2];
        self.parse_buf(&mut res)?;
        Ok(u16::from_le_bytes(res))
    }

    /// Takes a big-endian `i32` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by four octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i32_be(&mut self) -> Result<i32, ShortInput> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(i32::from_be_bytes(res))
    }

    /// Takes a little-endian `i32` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by four octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i32_le(&mut self) -> Result<i32, ShortInput> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(i32::from_le_bytes(res))
    }

    /// Takes a big-endian `u32` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by four octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u32_be(&mut self) -> Result<u32, ShortInput> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(u32::from_be_bytes(res))
    }

    /// Takes a little-endian `u32` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by four octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u32_le(&mut self) -> Result<u32, ShortInput> {
        let mut res = [0; 4];
        self.parse_buf(&mut res)?;
        Ok(u32::from_le_bytes(res))
    }

    /// Takes a big-endian `i64` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by eight octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i64_be(&mut self) -> Result<i64, ShortInput> {
        let mut res = [0; 8];
        self.parse_buf(&mut res)?;
        Ok(i64::from_be_bytes(res))
    }

    /// Takes a little-endian `i64` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by eight octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i64_le(&mut self) -> Result<i64, ShortInput> {
        let mut res = [0; 8];
        self.parse_buf(&mut res)?;
        Ok(i64::from_le_bytes(res))
    }

    /// Takes a big-endian `u64` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by eight octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u64_be(&mut self) -> Result<u64, ShortInput> {
        let mut res = [0; 8];
        self.parse_buf(&mut res)?;
        Ok(u64::from_be_bytes(res))
    }

    /// Takes a little-endian `u64` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by eight octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u64_le(&mut self) -> Result<u64, ShortInput> {
        let mut res = [0; 8];
        self.parse_buf(&mut res)?;
        Ok(u64::from_le_bytes(res))
    }

    /// Takes a big-endian `i128` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by 16 octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i128_be(&mut self) -> Result<i128, ShortInput> {
        let mut res = [0; 16];
        self.parse_buf(&mut res)?;
        Ok(i128::from_be_bytes(res))
    }

    /// Takes a little-endian `i128` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by 16 octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_i128_le(&mut self) -> Result<i128, ShortInput> {
        let mut res = [0; 16];
        self.parse_buf(&mut res)?;
        Ok(i128::from_le_bytes(res))
    }

    /// Takes a big-endian `u128` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by 16 octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u128_be(&mut self) -> Result<u128, ShortInput> {
        let mut res = [0; 16];
        self.parse_buf(&mut res)?;
        Ok(u128::from_be_bytes(res))
    }

    /// Takes a little-endian `u128` from the beginning of the parser.
    ///
    /// The value is converted into the system’s own byte order if necessary.
    /// The parser is advanced by 16 octets. If there aren’t enough octets
    /// left, leaves the parser untouched and returns an error instead.
    pub fn parse_u128_le(&mut self) -> Result<u128, ShortInput> {
        let mut res = [0; 16];
        self.parse_buf(&mut res)?;
        Ok(u128::from_le_bytes(res))
    }
}


//--- Clone and Copy

impl<'a, Octs: ?Sized> Clone for Parser<'a, Octs> {
    fn clone(&self) -> Self {
        Parser {
            octets: self.octets,
            pos: self.pos,
            len: self.len
        }
    }
}

impl<'a, Octs: ?Sized> Copy for Parser<'a, Octs> { }


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


//============ Testing =======================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pos_seek_remaining() {
        let mut parser = Parser::from_static(b"0123456789");
        assert_eq!(parser.peek(1).unwrap(), b"0");
        assert_eq!(parser.pos(), 0);
        assert_eq!(parser.remaining(), 10);
        assert_eq!(parser.seek(2), Ok(()));
        assert_eq!(parser.pos(), 2);
        assert_eq!(parser.remaining(), 8);
        assert_eq!(parser.peek(1).unwrap(), b"2");
        assert_eq!(parser.seek(10), Ok(()));
        assert_eq!(parser.pos(), 10);
        assert_eq!(parser.remaining(), 0);
        assert_eq!(parser.peek_all(), b"");
        assert!(parser.seek(11).is_err());
        assert_eq!(parser.pos(), 10);
        assert_eq!(parser.remaining(), 0);
    }

    #[test]
    fn peek_check_len() {
        let mut parser = Parser::from_static(b"0123456789");
        assert_eq!(parser.peek(2), Ok(b"01".as_ref()));
        assert_eq!(parser.check_len(2), Ok(()));
        assert_eq!(parser.peek(10), Ok(b"0123456789".as_ref()));
        assert_eq!(parser.check_len(10), Ok(()));
        assert!(parser.peek(11).is_err());
        assert!(parser.check_len(11).is_err());
        parser.advance(2).unwrap();
        assert_eq!(parser.peek(2), Ok(b"23".as_ref()));
        assert_eq!(parser.check_len(2), Ok(()));
        assert_eq!(parser.peek(8), Ok(b"23456789".as_ref()));
        assert_eq!(parser.check_len(8), Ok(()));
        assert!(parser.peek(9).is_err());
        assert!(parser.check_len(9).is_err());
    }

    #[test]
    fn peek_all() {
        let mut parser = Parser::from_static(b"0123456789");
        assert_eq!(parser.peek_all(), b"0123456789");
        parser.advance(2).unwrap();
        assert_eq!(parser.peek_all(), b"23456789");
    }

    #[test]
    fn advance() {
        let mut parser = Parser::from_static(b"0123456789");
        assert_eq!(parser.pos(), 0);
        assert_eq!(parser.peek(1).unwrap(), b"0");
        assert_eq!(parser.advance(2), Ok(()));
        assert_eq!(parser.pos(), 2);
        assert_eq!(parser.peek(1).unwrap(), b"2");
        assert!(parser.advance(9).is_err());
        assert_eq!(parser.advance(8), Ok(()));
        assert_eq!(parser.pos(), 10);
        assert_eq!(parser.peek_all(), b"");
    }

    #[test]
    fn parse_octets() {
        let mut parser = Parser::from_static(b"0123456789");
        assert_eq!(parser.parse_octets(2).unwrap(), b"01");
        assert_eq!(parser.parse_octets(2).unwrap(), b"23");
        assert!(parser.parse_octets(7).is_err());
        assert_eq!(parser.parse_octets(6).unwrap(), b"456789");
    }

    #[test]
    fn parse_buf() {
        let mut parser = Parser::from_static(b"0123456789");
        let mut buf = [0u8; 2];
        assert_eq!(parser.parse_buf(&mut buf), Ok(()));
        assert_eq!(&buf, b"01");
        assert_eq!(parser.parse_buf(&mut buf), Ok(()));
        assert_eq!(&buf, b"23");
        let mut buf = [0u8; 7];
        assert!(parser.parse_buf(&mut buf).is_err());
        let mut buf = [0u8; 6];
        assert_eq!(parser.parse_buf(&mut buf), Ok(()));
        assert_eq!(&buf, b"456789");
    }

    #[test]
    fn parse_i8() {
        let mut parser = Parser::from_static(b"\x12\xd6");
        assert_eq!(parser.parse_i8(), Ok(0x12_i8));
        assert_eq!(parser.parse_i8(), Ok(-42_i8));
        assert!(parser.parse_i8().is_err());
    }

    #[test]
    fn parse_u8() {
        let mut parser = Parser::from_static(b"\x12\xd6");
        assert_eq!(parser.parse_u8(), Ok(0x12_u8));
        assert_eq!(parser.parse_u8(), Ok(0xd6_u8));
        assert!(parser.parse_u8().is_err());
    }

    #[test]
    fn parse_i16_be() {
        let mut parser = Parser::from_static(b"\x12\x34\xef\x6e\0");
        assert_eq!(parser.parse_i16_be(), Ok(0x1234_i16));
        assert_eq!(parser.parse_i16_be(), Ok(-4242_i16));
        assert!(parser.parse_i16_be().is_err());
    }

    #[test]
    fn parse_i16_le() {
        let mut parser = Parser::from_static(b"\x34\x12\x6e\xef\0");
        assert_eq!(parser.parse_i16_le(), Ok(0x1234_i16));
        assert_eq!(parser.parse_i16_le(), Ok(-4242_i16));
        assert!(parser.parse_i16_le().is_err());
    }

    #[test]
    fn parse_u16_be() {
        let mut parser = Parser::from_static(b"\x12\x34\xef\x6e\0");
        assert_eq!(parser.parse_u16_be(), Ok(0x1234_u16));
        assert_eq!(parser.parse_u16_be(), Ok(0xef6e_u16));
        assert!(parser.parse_u16_be().is_err());
    }

    #[test]
    fn parse_u16_le() {
        let mut parser = Parser::from_static(b"\x34\x12\x6e\xef\0");
        assert_eq!(parser.parse_u16_le(), Ok(0x1234_u16));
        assert_eq!(parser.parse_u16_le(), Ok(0xef6e_u16));
        assert!(parser.parse_u16_le().is_err());
    }

    #[test]
    fn parse_i32_be() {
        let mut parser =
            Parser::from_static(b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\0\0\0");
        assert_eq!(parser.parse_i32_be(), Ok(0x12345678_i32));
        assert_eq!(parser.parse_i32_be(), Ok(-42424242_i32));
        assert!(parser.parse_i32_be().is_err());
    }

    #[test]
    fn parse_i32_le() {
        let mut parser =
            Parser::from_static(b"\x78\x56\x34\x12\x4e\xa8\x78\xfd\0\0\0");
        assert_eq!(parser.parse_i32_le(), Ok(0x12345678_i32));
        assert_eq!(parser.parse_i32_le(), Ok(-42424242_i32));
        assert!(parser.parse_i32_le().is_err());
    }

    #[test]
    fn parse_u32_be() {
        let mut parser =
            Parser::from_static(b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\0\0\0");
        assert_eq!(parser.parse_u32_be(), Ok(0x12345678_u32));
        assert_eq!(parser.parse_u32_be(), Ok(0xfd78a84e_u32));
        assert!(parser.parse_u32_be().is_err());
    }

    #[test]
    fn parse_u32_le() {
        let mut parser =
            Parser::from_static(b"\x78\x56\x34\x12\x4e\xa8\x78\xfd\0\0\0");
        assert_eq!(parser.parse_u32_le(), Ok(0x12345678_u32));
        assert_eq!(parser.parse_u32_le(), Ok(0xfd78a84e_u32));
        assert!(parser.parse_u32_le().is_err());
    }

    #[test]
    fn parse_i64_be() {
        let mut parser = Parser::from_static(
            b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\
              \xce\x7a\xba\x26\xdd\x0f\x29\x99\
              \0\0\0"
        );
        assert_eq!(parser.parse_i64_be(), Ok(0x12345678fd78a84e_i64));
        assert_eq!(parser.parse_i64_be(), Ok(-3568335078657414759_i64));
        assert!(parser.parse_i64_be().is_err());
    }

    #[test]
    fn parse_i64_le() {
        let mut parser = Parser::from_static(
            b"\x4e\xa8\x78\xfd\x78\x56\x34\x12\
              \x99\x29\x0f\xdd\x26\xba\x7a\xce\
              \0\0\0"
        );
        assert_eq!(parser.parse_i64_le(), Ok(0x12345678fd78a84e_i64));
        assert_eq!(parser.parse_i64_le(), Ok(-3568335078657414759_i64));
        assert!(parser.parse_i64_le().is_err());
    }

    #[test]
    fn parse_u64_be() {
        let mut parser = Parser::from_static(
            b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\
              \xce\x7a\xba\x26\xdd\x0f\x29\x99\
              \0\0\0"
        );
        assert_eq!(parser.parse_u64_be(), Ok(0x12345678fd78a84e_u64));
        assert_eq!(parser.parse_u64_be(), Ok(0xce7aba26dd0f2999_u64));
        assert!(parser.parse_u64_be().is_err());
    }

    #[test]
    fn parse_u64_le() {
        let mut parser = Parser::from_static(
            b"\x4e\xa8\x78\xfd\x78\x56\x34\x12\
              \x99\x29\x0f\xdd\x26\xba\x7a\xce\
              \0\0\0"
        );
        assert_eq!(parser.parse_u64_le(), Ok(0x12345678fd78a84e_u64));
        assert_eq!(parser.parse_u64_le(), Ok(0xce7aba26dd0f2999_u64));
        assert!(parser.parse_u64_le().is_err());
    }

    #[test]
    fn parse_i128_be() {
        let mut parser = Parser::from_static(
            b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\
              \xce\x7a\xba\x26\xdd\x0f\x29\x99\
              \xf8\xc6\x0e\x5d\x3f\x5e\x3a\x74\
              \x38\x38\x8f\x3f\x57\xa7\x94\xa0\
              \0\0\0\0\0"
        );
        assert_eq!(parser.parse_i128_be(),
            Ok(0x12345678fd78a84ece7aba26dd0f2999_i128)
        );
        assert_eq!(parser.parse_i128_be(),
            Ok(-9605457846724395475894107919101750112_i128)
        );
        assert!(parser.parse_i128_be().is_err());
    }

    #[test]
    fn parse_i128_le() {
        let mut parser = Parser::from_static(
            b"\x99\x29\x0f\xdd\x26\xba\x7a\xce\
              \x4e\xa8\x78\xfd\x78\x56\x34\x12\
              \xa0\x94\xa7\x57\x3f\x8f\x38\x38\
              \x74\x3a\x5e\x3f\x5d\x0e\xc6\xf8\
              \0\0\0\0\0"
        );
        assert_eq!(parser.parse_i128_le(),
            Ok(0x12345678fd78a84ece7aba26dd0f2999_i128)
        );
        assert_eq!(parser.parse_i128_le(),
            Ok(-9605457846724395475894107919101750112_i128)
        );
        assert!(parser.parse_i128_le().is_err());
    }

    #[test]
    fn parse_u128_be() {
        let mut parser = Parser::from_static(
            b"\x12\x34\x56\x78\xfd\x78\xa8\x4e\
              \xce\x7a\xba\x26\xdd\x0f\x29\x99\
              \xf8\xc6\x0e\x5d\x3f\x5e\x3a\x74\
              \x38\x38\x8f\x3f\x57\xa7\x94\xa0\
              \0\0\0\0\0"
        );
        assert_eq!(parser.parse_u128_be(),
            Ok(0x12345678fd78a84ece7aba26dd0f2999_u128)
        );
        assert_eq!(parser.parse_u128_be(),
            Ok(0xf8c60e5d3f5e3a7438388f3f57a794a0_u128)
        );
        assert!(parser.parse_u128_be().is_err());
    }

    #[test]
    fn parse_u128_le() {
        let mut parser = Parser::from_static(
            b"\x99\x29\x0f\xdd\x26\xba\x7a\xce\
              \x4e\xa8\x78\xfd\x78\x56\x34\x12\
              \xa0\x94\xa7\x57\x3f\x8f\x38\x38\
              \x74\x3a\x5e\x3f\x5d\x0e\xc6\xf8\
              \0\0\0\0\0"
        );
        assert_eq!(parser.parse_u128_le(),
            Ok(0x12345678fd78a84ece7aba26dd0f2999_u128)
        );
        assert_eq!(parser.parse_u128_le(),
            Ok(0xf8c60e5d3f5e3a7438388f3f57a794a0_u128)
        );
        assert!(parser.parse_u128_le().is_err());
    }
}

