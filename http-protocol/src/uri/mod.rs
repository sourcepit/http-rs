mod char_stream;
mod token_buffer;

//https://tools.ietf.org/html/rfc2396#appendix-A

use common_failures::prelude::*;

use std::fmt::Write;
use uri::char_stream::Char;
use uri::token_buffer::TokenStream;
use uri::token_buffer::*;

// path          = [ abs_path | opaque_part ]

fn ipv4_address<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<IPv4address>>
where
    T: TokenStream<Char>,
{
    let d1 = match digits(tb)? {
        Some(d) => d,
        None => return Ok(None),
    };

    let dot1 = match tb.pop()? {
        Some(c) => {
            if c.is(b'.') {
                Some(c)
            } else {
                tb.push(c);
                None
            }
        }
        None => None,
    };

    let dot1 = match dot1 {
        Some(c) => c,
        None => {
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    let d2 = match digits(tb)? {
        Some(d) => d,
        None => {
            tb.push(dot1);
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    let dot2 = match tb.pop()? {
        Some(c) => {
            if c.is(b'.') {
                Some(c)
            } else {
                tb.push(c);
                None
            }
        }
        None => None,
    };

    let dot2 = match dot2 {
        Some(c) => c,
        None => {
            tb.push_tokens(d2);
            tb.push(dot1);
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    let d3 = match digits(tb)? {
        Some(d) => d,
        None => {
            tb.push(dot2);
            tb.push_tokens(d2);
            tb.push(dot1);
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    let dot3 = match tb.pop()? {
        Some(c) => {
            if c.is(b'.') {
                Some(c)
            } else {
                tb.push(c);
                None
            }
        }
        None => None,
    };

    let dot3 = match dot3 {
        Some(c) => c,
        None => {
            tb.push_tokens(d3);
            tb.push(dot2);
            tb.push_tokens(d2);
            tb.push(dot1);
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    let d4 = match digits(tb)? {
        Some(d) => d,
        None => {
            tb.push(dot3);
            tb.push_tokens(d3);
            tb.push(dot2);
            tb.push_tokens(d2);
            tb.push(dot1);
            tb.push_tokens(d1);
            return Ok(None);
        }
    };

    Ok(Some(IPv4address(d1, d2, d3, d4)))
}

#[derive(Debug, PartialEq)]
struct IPv4address(Vec<Char>, Vec<Char>, Vec<Char>, Vec<Char>);

impl std::fmt::Display for IPv4address {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        fmt.write_char('.')?;
        for c in &self.1 {
            fmt.write_str(c.to_string().as_str())?;
        }
        fmt.write_char('.')?;
        for c in &self.2 {
            fmt.write_str(c.to_string().as_str())?;
        }
        fmt.write_char('.')?;
        for c in &self.3 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct Port {
    digits: Vec<Char>,
}

impl std::fmt::Display for Port {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for d in &self.digits {
            fmt.write_str(d.to_string().as_str())?;
        }
        Ok(())
    }
}

fn port<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Port>>
where
    T: TokenStream<Char>,
{
    match digits(tb)? {
        None => Ok(None),
        Some(digits) => Ok(Some(Port { digits })),
    }
}

fn digits<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Vec<Char>>>
where
    T: TokenStream<Char>,
{
    let mut digits: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = tb.pop()? {
            if c.is_digit() {
                digits.push(c);
                continue;
            } else {
                tb.push(c);
            }
        }
        break;
    }
    match digits.is_empty() {
        true => Ok(None),
        false => Ok(Some(digits)),
    }
}

#[derive(Debug, PartialEq)]
struct PathSegments {
    segments: Vec<Segment>,
}

impl std::fmt::Display for PathSegments {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, s) in self.segments.iter().enumerate() {
            if i > 0 {
                fmt.write_char('/')?;
            }
            fmt.write_str(s.to_string().as_str())?;
        }
        Ok(())
    }
}

fn path_segments<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<PathSegments>>
where
    T: TokenStream<Char>,
{
    let mut segments: Vec<Segment> = Vec::new();
    match segment(tb)? {
        Some(segment) => segments.push(segment),
        None => (),
    };
    if !segments.is_empty() {
        loop {
            if let Some(c) = tb.pop()? {
                if c.is(b'/') {
                    match segment(tb)? {
                        Some(segment) => {
                            segments.push(segment);
                            continue;
                        }
                        None => {
                            segments.push(Segment::new());
                            break;
                        }
                    }
                } else {
                    tb.push(c);
                }
            }
            break;
        }
    }
    match segments.is_empty() {
        true => Ok(None),
        false => Ok(Some(PathSegments { segments })),
    }
}

#[derive(Debug, PartialEq, Default)]
struct Segment {
    pchars: Vec<Char>,
    params: Option<Vec<Param>>,
}

impl Segment {
    fn new() -> Segment {
        Segment {
            ..Default::default()
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.pchars {
            fmt.write_str(c.to_string().as_str())?;
        }
        if let Some(params) = &self.params {
            for p in params {
                fmt.write_char(';')?;
                fmt.write_str(p.to_string().as_str())?;
            }
        }
        Ok(())
    }
}

fn segment<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Segment>>
where
    T: TokenStream<Char>,
{
    let mut s = match pchars(tb)? {
        Some(pchars) => Some(Segment {
            pchars: pchars,
            params: None,
        }),
        None => None,
    };
    if let Some(s) = &mut s {
        let mut params: Vec<Param> = Vec::new();
        loop {
            if let Some(c) = tb.pop()? {
                if c.is(b';') {
                    match param(tb)? {
                        Some(p) => {
                            params.push(p);
                            continue;
                        }
                        None => {
                            params.push(Param { pchars: Vec::new() });
                            break;
                        }
                    }
                } else {
                    tb.push(c);
                }
            }
            break;
        }
        if !params.is_empty() {
            s.params = Some(params);
        }
    }
    Ok(s)
}

#[derive(Debug, PartialEq)]
struct Param {
    pchars: Vec<Char>,
}

impl std::fmt::Display for Param {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.pchars {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn param<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Param>>
where
    T: TokenStream<Char>,
{
    let p = match pchars(tb)? {
        Some(pchars) => Some(Param { pchars }),
        None => None,
    };
    Ok(p)
}

fn pchars<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Vec<Char>>>
where
    T: TokenStream<Char>,
{
    let mut pchars: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = tb.pop()? {
            if c.is_pchar() {
                pchars.push(c);
            } else {
                tb.push(c);
                break;
            }
        } else {
            break;
        }
    }
    match pchars.len() {
        0 => Ok(None),
        _ => Ok(Some(pchars)),
    }
}

fn query<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Vec<Char>>>
where
    T: TokenStream<Char>,
{
    fragment(tb)
}

fn fragment<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Vec<Char>>>
where
    T: TokenStream<Char>,
{
    let mut fragment: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = tb.pop()? {
            if c.is_uric() {
                fragment.push(c);
            } else {
                tb.push(c);
                break;
            }
        } else {
            break;
        }
    }
    match fragment.len() {
        0 => Ok(None),
        _ => Ok(Some(fragment)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_address() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let ip = ipv4_address(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, ip);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let ip = ipv4_address(&mut tb).unwrap();
        assert_eq!(None, ip);
        assert_eq!(1, tb.buffer.len());
        assert_eq!("f", tb.pop().unwrap().unwrap().to_string());

        let mut tb = TokenBuffer::from("12.34.56.foo".as_bytes());
        let ip = ipv4_address(&mut tb).unwrap();
        assert_eq!(None, ip);
        assert_eq!(10, tb.buffer.len());
        assert_eq!("1", tb.pop().unwrap().unwrap().to_string());

        let mut tb = TokenBuffer::from("12.34.56.78.foo".as_bytes());
        let ip = ipv4_address(&mut tb).unwrap().unwrap();
        assert_eq!("12.34.56.78", ip.to_string());
        assert_eq!(1, tb.buffer.len());
        assert_eq!(".", tb.pop().unwrap().unwrap().to_string());

        let mut tb = TokenBuffer::from("12.34.56.78".as_bytes());
        let ip = ipv4_address(&mut tb).unwrap().unwrap();
        assert_eq!("12.34.56.78", ip.to_string());
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, tb.pop().unwrap());
    }

    #[test]
    fn test_path_segments() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let ps = path_segments(&mut tb).unwrap();
        assert_eq!(None, ps);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let ps = path_segments(&mut tb).unwrap().unwrap();
        assert_eq!("foo", ps.to_string());
        let segments = ps.segments;
        assert_eq!(1, segments.len());

        let mut tb = TokenBuffer::from("foo/".as_bytes());
        let ps = path_segments(&mut tb).unwrap().unwrap();
        assert_eq!("foo/", ps.to_string());
        let segments = ps.segments;
        assert_eq!(2, segments.len());

        let mut tb = TokenBuffer::from("foo/bar".as_bytes());
        let ps = path_segments(&mut tb).unwrap().unwrap();
        assert_eq!("foo/bar", ps.to_string());
        let segments = ps.segments;
        assert_eq!(2, segments.len());

        let mut tb = TokenBuffer::from("foo;bar/bar".as_bytes());
        let ps = path_segments(&mut tb).unwrap().unwrap();
        assert_eq!("foo;bar/bar", ps.to_string());
        let segments = ps.segments;
        assert_eq!(2, segments.len());
    }

    #[test]
    fn test_segment() -> Result<()> {
        let mut tb = TokenBuffer::from("".as_bytes());
        let p = segment(&mut tb).unwrap();
        assert_eq!(None, p);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let p = segment(&mut tb).unwrap().unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            p.pchars
        );
        assert_eq!(None, p.params);

        let mut tb = TokenBuffer::from("foo;bar;buh".as_bytes());
        let p = segment(&mut tb).unwrap().unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            p.pchars
        );
        let params = p.params.unwrap();
        assert_eq!(2, params.len());
        assert_eq!("bar", params[0].to_string());
        assert_eq!("buh", params[1].to_string());

        let mut tb = TokenBuffer::from("foo;".as_bytes());
        let p = segment(&mut tb).unwrap().unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            p.pchars
        );
        let params = p.params.unwrap();
        assert_eq!(1, params.len());
        assert_eq!("", params[0].to_string());

        Ok(())
    }

    #[test]
    fn test_param() -> Result<()> {
        let mut tb = TokenBuffer::from("foo?bar".as_bytes());

        let p = param(&mut tb)?.unwrap();
        assert_eq!("foo", p.to_string());

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'?'), c);

        let p = param(&mut tb)?.unwrap();
        assert_eq!("bar", p.to_string());

        assert_eq!(None, tb.pop()?);

        Ok(())
    }

    #[test]
    fn test_pchars() -> Result<()> {
        let mut tb = TokenBuffer::from("foo?bar".as_bytes());

        let q = pchars(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            q
        );

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'?'), c);

        let q = pchars(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'b'), Char::Ascii(b'a'), Char::Ascii(b'r')],
            q
        );

        assert_eq!(None, tb.pop()?);

        Ok(())
    }

    #[test]
    fn test_query() -> Result<()> {
        let mut tb = TokenBuffer::from("foo}bar".as_bytes());

        let q = query(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            q
        );

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'}'), c);

        let q = query(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'b'), Char::Ascii(b'a'), Char::Ascii(b'r')],
            q
        );

        assert_eq!(None, tb.pop()?);

        Ok(())
    }

    #[test]
    fn test_fragment() -> Result<()> {
        let mut tb = TokenBuffer::from("foo}bar".as_bytes());

        let f = fragment(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            f
        );

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'}'), c);

        let f = fragment(&mut tb)?.unwrap();
        assert_eq!(
            vec![Char::Ascii(b'b'), Char::Ascii(b'a'), Char::Ascii(b'r')],
            f
        );

        assert_eq!(None, tb.pop()?);

        Ok(())
    }
}
