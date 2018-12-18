use common_failures::Result;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::io::Read;
use uri::token_buffer::ByteStream;
use uri::token_buffer::TokenBuffer;
use uri::token_buffer::TokenStream;

pub struct CharStream<T: TokenStream<u8>> {
    byte_stream: T,
}

impl<R: Read> From<R> for CharStream<ByteStream<R>> {
    fn from(read: R) -> CharStream<ByteStream<R>> {
        let byte_stream = ByteStream::from(read);
        CharStream { byte_stream }
    }
}

impl<T: TokenStream<u8>> TokenStream<Char> for CharStream<T> {
    fn next(&mut self) -> Result<Option<Char>> {
        let b = match self.byte_stream.next()? {
            Some(b) => b,
            None => return Ok(None),
        };
        let c: Char;
        if b == b'%' {
            let b2 = match self.byte_stream.next()? {
                Some(b) => match is_hex(b) {
                    true => b,
                    false => return Err(format_err!("Invalid escape sequence.")),
                },
                None => return Err(format_err!("Unexpected end of escape sequence.")),
            };
            let b3 = match self.byte_stream.next()? {
                Some(b) => match is_hex(b) {
                    true => b,
                    false => return Err(format_err!("Invalid escape sequence.")),
                },
                None => return Err(format_err!("Unexpected end of escape sequence.")),
            };
            c = Char::Escaped((b, b2, b3));
        } else {
            c = Char::Ascii(b);
        }
        Ok(Some(c))
    }
}

impl<R: Read> From<R> for TokenBuffer<Char, CharStream<ByteStream<R>>> {
    fn from(from: R) -> TokenBuffer<Char, CharStream<ByteStream<R>>> {
        TokenBuffer {
            stream: CharStream::from(from),
            buffer: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Char {
    Ascii(u8),
    Escaped((u8, u8, u8)),
}

impl Char {
    pub fn is(&self, byte: u8) -> bool {
        match self {
            Char::Ascii(b) => *b == byte,
            _ => false,
        }
    }

    pub fn is_escaped(&self) -> bool {
        match self {
            Char::Ascii(_) => false,
            Char::Escaped(_) => true,
        }
    }

    pub fn is_pchar(&self) -> bool {
        match self {
            Char::Escaped(_) => true,
            Char::Ascii(b) => {
                is_unreserved(*b) || match b {
                    b':' => true,
                    b'@' => true,
                    b'&' => true,
                    b'=' => true,
                    b'+' => true,
                    b'$' => true,
                    b',' => true,
                    _ => false,
                }
            }
        }
    }

    pub fn is_uric(&self) -> bool {
        match self {
            Char::Ascii(b) => is_reserved(*b) || is_unreserved(*b),
            Char::Escaped(_) => true,
        }
    }

    pub fn is_uric_no_slash(&self) -> bool {
        match self {
            Char::Escaped(_) => true,
            Char::Ascii(b) => {
                is_unreserved(*b) || match b {
                    b';' => true,
                    b'?' => true,
                    b':' => true,
                    b'@' => true,
                    b'&' => true,
                    b'=' => true,
                    b'+' => true,
                    b'$' => true,
                    b',' => true,
                    _ => false,
                }
            }
        }
    }

    pub fn is_digit(&self) -> bool {
        match self {
            Char::Ascii(b) => is_digit(*b),
            Char::Escaped(_) => false,
        }
    }

    pub fn is_alpha(&self) -> bool {
        match self {
            Char::Ascii(b) => is_alpha(*b),
            Char::Escaped(_) => false,
        }
    }

    pub fn is_alphanum(&self) -> bool {
        match self {
            Char::Ascii(b) => is_alphanum(*b),
            Char::Escaped(_) => false,
        }
    }

    pub fn is_unreserved(&self) -> bool {
        match self {
            Char::Ascii(b) => is_unreserved(*b),
            Char::Escaped(_) => false,
        }
    }
}

impl Display for Char {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match &self {
            Char::Ascii(b) => fmt.write_char(*b as char)?,
            Char::Escaped(bytes) => {
                fmt.write_char(bytes.0 as char)?;
                fmt.write_char(bytes.1 as char)?;
                fmt.write_char(bytes.2 as char)?;
            }
        };
        Ok(())
    }
}

fn is_reserved(b: u8) -> bool {
    match b {
        b';' => true,
        b'/' => true,
        b'?' => true,
        b':' => true,
        b'@' => true,
        b'&' => true,
        b'=' => true,
        b'+' => true,
        b'$' => true,
        b',' => true,
        _ => false,
    }
}

fn is_unreserved(b: u8) -> bool {
    is_alphanum(b) || is_mark(b)
}

fn is_mark(b: u8) -> bool {
    match b {
        b'-' => true,
        b'_' => true,
        b'.' => true,
        b'!' => true,
        b'~' => true,
        b'*' => true,
        b'\'' => true,
        b'(' => true,
        b')' => true,
        _ => false,
    }
}

fn is_escaped(bytes: (u8, u8, u8)) -> bool {
    bytes.0 == b'%' && is_hex(bytes.1) && is_hex(bytes.2)
}

fn is_hex(b: u8) -> bool {
    (b >= 65 && b <= 70) || (b >= 97 && b <= 102)
}

fn is_alphanum(b: u8) -> bool {
    is_alpha(b) || is_digit(b)
}

fn is_alpha(b: u8) -> bool {
    is_low_alpha(b) || is_up_alpha(b)
}

fn is_low_alpha(b: u8) -> bool {
    b >= 97 && b <= 122
}

fn is_up_alpha(b: u8) -> bool {
    b >= 65 && b <= 90
}

fn is_digit(b: u8) -> bool {
    b >= 48 && b <= 57
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escaped() -> Result<()> {
        let mut cs: CharStream<_> = "%FF".as_bytes().into();
        let c = cs.next()?.unwrap();
        assert_eq!(Char::Escaped((b'%', b'F', b'F')), c);
        assert_eq!(None, cs.next()?);

        let mut cs: CharStream<_> = "%GG".as_bytes().into();
        let c = cs.next();
        assert!(c.is_err());

        let mut cs: CharStream<_> = "%F".as_bytes().into();
        let c = cs.next();
        assert!(c.is_err());

        Ok(())
    }
}
