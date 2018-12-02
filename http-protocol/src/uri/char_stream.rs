use common_failures::Result;
use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::rc::Rc;

struct CharStream<'a, R: BufRead> {
    read: R,
    buf: &'a [u8],
}

impl<'a, T: Read> std::convert::From<T> for CharStream<'a, BufReader<T>> {
    fn from(read: T) -> CharStream<'a, BufReader<T>> {
        CharStream::new(read)
    }
}

impl<'a, T: Read> CharStream<'a, BufReader<T>> {
    fn new(read: T) -> CharStream<'a, BufReader<T>> {
        CharStream {
            read: BufReader::new(read),
            buf: &[0; 0],
        }
    }
}

impl<'a, R: BufRead> CharStream<'a, R> {
    fn next(&'a mut self) -> Result<Option<Char>> {
        if self.buf.len() < 3 {
            self.buf = &self.read.fill_buf()?;
        }

        let buf = self.buf;
        let mut c = Ok(None);
        if buf.len() > 0 {
            let b: u8 = buf[0];
            if b == b'%' {
                if buf.len() < 3 {
                    c = Err(format_err!("Unexpected end of escape sequence."));
                } else {
                    let bytes = (b, buf[1], buf[2]);
                    if is_escaped(bytes) {
                        c = Ok(Some(Char::Escaped(bytes)));
                    } else {
                        c = Err(format_err!("Invalid escape sequence."));
                    }
                }
            } else {
                c = Ok(Some(Char::Ascii(b)));
            }
        }
        c
    }

    // fn consume_size_of(&'a mut self, c: Char) {
    //     self.buf.consume(match c {
    //         Char::Ascii(_) => 1,
    //         Char::Escaped(_) => 3,
    //     });
    // }
}

fn is_escaped(bytes: (u8, u8, u8)) -> bool {
    bytes.0 == b'%' && is_hex(bytes.1) && is_hex(bytes.2)
}

fn is_hex(b: u8) -> bool {
    (b >= 65 && b <= 70) || (b >= 97 && b <= 102)
}

#[derive(Debug, PartialEq)]
enum Char {
    Ascii(u8),
    Escaped((u8, u8, u8)),
}

impl Char {
    fn is(&self, byte: u8) -> bool {
        match self {
            Char::Ascii(b) => *b == byte,
            _ => false,
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

struct Foo {}

impl Foo {
    fn foo(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    use std::io::BufReader;

    #[test]
    fn test_fill_buf() -> Result<()> {
        let mut foo = Foo {};
        foo.foo();
        foo.foo();
        foo.foo();

        let mut cs: CharStream<_> = "123".as_bytes().into();

        cs.next();

        cs.next();

        // let foo = &mut cs;
        // {
        //     let c = foo.next()?.unwrap();
        //     assert_eq!("1", c.to_string());
        // }

        // let c = foo.next()?.unwrap();
        // assert_eq!("1", c.to_string());

        // let mut reader = BufReader::with_capacity(2, "1234".as_bytes());
        // {
        //     let mut buf = reader.fill_buf().unwrap();
        //     assert_eq!(2, buf.len());

        //     buf.consume(1);
        //     assert_eq!(1, buf.len());
        // }

        // let mut buf = reader.fill_buf().unwrap();
        // assert_eq!(1, buf.len());

        // buf.consume(1);
        // assert_eq!(0, buf.len());

        Ok(())
    }
}
