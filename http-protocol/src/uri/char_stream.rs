use common_failures::Result;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::io::Read;

pub struct CharStream<R: Read> {
    read: R,
    buf: [u8; 1],
    current: Option<Char>,
}

impl<T: Read> std::convert::From<T> for CharStream<T> {
    fn from(read: T) -> CharStream<T> {
        CharStream::new(read)
    }
}

impl<T: Read> CharStream<T> {
    pub fn new(read: T) -> CharStream<T> {
        CharStream {
            read: read,
            buf: [0],
            current: None,
        }
    }

    pub fn next(&mut self) -> Result<Option<Char>> {
        match &self.current {
            Some(c) => return Ok(Some(*c)),
            None => (),
        };
        let b = match self.next_byte()? {
            Some(b) => b,
            None => return Ok(None),
        };
        let c: Char;
        if b == b'%' {
            let b2 = match self.next_byte()? {
                Some(b) => match is_hex(b) {
                    true => b,
                    false => return Err(format_err!("Invalid escape sequence.")),
                },
                None => return Err(format_err!("Unexpected end of escape sequence.")),
            };
            let b3 = match self.next_byte()? {
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
        self.current = Some(c);
        Ok(Some(c))
    }

    fn next_byte(&mut self) -> Result<Option<u8>> {
        match self.read.read(&mut self.buf)? {
            0 => Ok(None),
            1 => Ok(Some(self.buf[0])),
            _ => Err(format_err!("")),
        }
    }

    pub fn consume(&mut self) -> Result<()> {
        match self.current {
            Some(_) => {
                self.current = None;
                Ok(())
            }
            None => Err(format_err!("Nothing to consume")),
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
            Char::Escaped(bytes) => true,
        }
    }

    pub fn is_digit(&self) -> bool {
        match self {
            Char::Ascii(b) => is_digit(*b),
            Char::Escaped(bytes) => false,
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
    fn test_next_and_consume() -> Result<()> {
        let mut cs: CharStream<_> = "123".as_bytes().into();

        let c = cs.next()?.unwrap();
        assert_eq!("1", c.to_string());

        let c = cs.next()?.unwrap();
        assert_eq!("1", c.to_string());

        cs.consume()?;

        let c = cs.next()?.unwrap();
        assert_eq!("2", c.to_string());

        cs.consume()?;

        let c = cs.consume();
        assert!(c.is_err());

        let c = cs.next()?.unwrap();
        assert_eq!("3", c.to_string());

        cs.consume()?;

        let c = cs.next()?;
        assert_eq!(None, c);

        Ok(())
    }

    #[test]
    fn test_escaped() -> Result<()> {
        let mut cs: CharStream<_> = "%FF".as_bytes().into();
        let c = cs.next()?.unwrap();
        assert_eq!(Char::Escaped((b'%', b'F', b'F')), c);
        cs.consume()?;
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
