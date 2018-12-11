use common_failures::Result;
use std::convert::From;
use std::io::Read;

pub trait TokenStream<T> {
    fn next(&mut self) -> Result<Option<T>>;
}

pub struct ByteStream<R: Read> {
    read: R,
    buffer: [u8; 1],
}

impl<R: Read> From<R> for ByteStream<R> {
    fn from(from: R) -> ByteStream<R> {
        ByteStream {
            read: from,
            buffer: [0],
        }
    }
}

impl<R: Read> TokenStream<u8> for ByteStream<R> {
    fn next(&mut self) -> Result<Option<u8>> {
        match self.read.read(&mut self.buffer)? {
            0 => Ok(None),
            1 => Ok(Some(self.buffer[0])),
            _ => Err(format_err!("")),
        }
    }
}

pub struct TokenBuffer<T, S: TokenStream<T>> {
    pub(crate) stream: S,
    pub(crate) buffer: Vec<T>,
}

impl<R: Read> From<R> for TokenBuffer<u8, ByteStream<R>> {
    fn from(from: R) -> TokenBuffer<u8, ByteStream<R>> {
        TokenBuffer {
            stream: ByteStream::from(from),
            buffer: Vec::new(),
        }
    }
}

impl<T, S: TokenStream<T>> TokenBuffer<T, S> {
    pub fn pop(&mut self) -> Result<Option<T>> {
        let token;
        if self.buffer.is_empty() {
            token = self.stream.next()?;
        } else {
            token = Some(self.buffer.remove(0));
        }
        Ok(token)
    }

    pub fn push(&mut self, token: T) {
        self.buffer.insert(0, token);
    }

    pub fn push_tokens(&mut self, mut tokens: Vec<T>) {
        for i in 0..tokens.len() {
            self.buffer.insert(i, tokens.remove(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let mut tb = TokenBuffer::from("Hello World".as_bytes());

        let mut word1: Vec<u8> = Vec::new();
        word1.push(tb.pop().unwrap().unwrap());
        word1.push(tb.pop().unwrap().unwrap());
        word1.push(tb.pop().unwrap().unwrap());
        word1.push(tb.pop().unwrap().unwrap());
        word1.push(tb.pop().unwrap().unwrap());

        let mut blank: Vec<u8> = Vec::new();
        blank.push(tb.pop().unwrap().unwrap());

        let mut word2: Vec<u8> = Vec::new();
        word2.push(tb.pop().unwrap().unwrap());
        word2.push(tb.pop().unwrap().unwrap());
        word2.push(tb.pop().unwrap().unwrap());
        word2.push(tb.pop().unwrap().unwrap());
        word2.push(tb.pop().unwrap().unwrap());

        assert_eq!(None, tb.pop().unwrap());

        tb.push_tokens(word2);
        tb.push_tokens(blank);
        tb.push_tokens(word1);

        let foo = tb.buffer;
        println!("{:?}", foo)
    }
}
