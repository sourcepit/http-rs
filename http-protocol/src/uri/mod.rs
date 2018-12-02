mod char_stream;

#[cfg(test)]
mod tests;

//https://tools.ietf.org/html/rfc2396#appendix-A

use common_failures::prelude::*;

use std::fmt::Write;
use std::io::BufRead;

// path          = [ abs_path | opaque_part ]

// port          = *digit
#[derive(Debug, PartialEq)]
struct Port {
    digits: Vec<u8>,
}

impl std::fmt::Display for Port {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for d in &self.digits {
            fmt.write_char(*d as char)?;
        }
        Ok(())
    }
}

fn port(r: &mut BufRead) -> Result<Option<Port>> {
    let mut digits: Vec<u8> = Vec::new();
    loop {
        if let Some(c) = next_char(r)? {
            match c {
                Char::Normal(b) => if is_digit(b) {
                    consume_char(r, &c);
                    digits.push(b);
                    continue;
                },
                _ => (),
            }
        }
        break;
    }
    match digits.is_empty() {
        true => Ok(None),
        false => Ok(Some(Port { digits })),
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

fn path_segments(r: &mut BufRead) -> Result<Option<PathSegments>> {
    let mut segments: Vec<Segment> = Vec::new();
    match segment(r)? {
        Some(segment) => segments.push(segment),
        None => (),
    };
    if !segments.is_empty() {
        loop {
            if let Some(c) = next_char(r)? {
                if c.is(b'/') {
                    consume_char(r, &c);
                    match segment(r)? {
                        Some(segment) => {
                            segments.push(segment);
                            continue;
                        }
                        None => {
                            segments.push(Segment::new());
                            break;
                        }
                    }
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

fn segment(r: &mut BufRead) -> Result<Option<Segment>> {
    let mut s = match pchars(r)? {
        Some(pchars) => Some(Segment {
            pchars: pchars,
            params: None,
        }),
        None => None,
    };
    if let Some(s) = &mut s {
        let mut params: Vec<Param> = Vec::new();
        loop {
            if let Some(c) = next_char(r)? {
                if c.is(b';') {
                    consume_char(r, &c);
                    match param(r)? {
                        Some(p) => {
                            params.push(p);
                            continue;
                        }
                        None => {
                            params.push(Param { pchars: Vec::new() });
                            break;
                        }
                    }
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

fn param(r: &mut BufRead) -> Result<Option<Param>> {
    let p = match pchars(r)? {
        Some(pchars) => Some(Param { pchars }),
        None => None,
    };
    Ok(p)
}

fn pchars(r: &mut BufRead) -> Result<Option<Vec<Char>>> {
    let mut param: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = next_char(r)? {
            if is_pchar(&c) {
                consume_char(r, &c);
                param.push(c);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    match param.is_empty() {
        true => Ok(None),
        false => Ok(Some(param)),
    }
}

fn is_pchar(c: &Char) -> bool {
    match c {
        Char::Escaped(_) => true,
        Char::Normal(b) => {
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

#[derive(Debug, PartialEq)]
enum Char {
    Normal(u8),
    Escaped((u8, u8, u8)),
}

impl Char {
    fn is(&self, byte: u8) -> bool {
        match self {
            Char::Normal(b) => *b == byte,
            _ => false,
        }
    }
}

impl std::fmt::Display for Char {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Char::Normal(b) => fmt.write_char(*b as char)?,
            Char::Escaped(bytes) => {
                fmt.write_char(bytes.0 as char)?;
                fmt.write_char(bytes.1 as char)?;
                fmt.write_char(bytes.2 as char)?;
            }
        };
        Ok(())
    }
}

fn next_char(r: &mut BufRead) -> Result<Option<Char>> {
    let mut c = Ok(None);
    let buf = r.fill_buf()?;
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
            c = Ok(Some(Char::Normal(b)));
        }
    }
    c
}

fn consume_char(r: &mut BufRead, c: &Char) {
    match c {
        Char::Normal(_) => r.consume(1),
        Char::Escaped(_) => r.consume(3),
    }
}

enum Uric {
    Simple(u8),
    Escaped((u8, u8, u8)),
}

fn query(r: &mut BufRead) -> Result<Option<Vec<Uric>>> {
    fragment(r)
}

fn fragment(r: &mut BufRead) -> Result<Option<Vec<Uric>>> {
    let mut fragment: Vec<Uric> = Vec::new();
    while {
        let uric = uric(r)?;
        match uric {
            Some(uric) => {
                fragment.push(uric);
                true
            }
            None => false,
        }
    } {}
    match fragment.len() {
        0 => Ok(None),
        _ => Ok(Some(fragment)),
    }
}

fn uric(r: &mut BufRead) -> Result<Option<Uric>> {
    let uric = {
        let mut uric = None;
        let buf = r.fill_buf()?;
        if buf.len() > 0 {
            let b: u8 = buf[0];
            if is_reserved(b) || is_unreserved(b) {
                uric = Some(Uric::Simple(b))
            } else {
                if buf.len() > 2 {
                    let bytes = (b, buf[1], buf[2]);
                    if is_escaped(bytes) {
                        uric = Some(Uric::Escaped(bytes))
                    }
                }
            }
        }
        uric
    };
    match &uric {
        Some(uric) => match uric {
            Uric::Simple(_) => r.consume(1),
            Uric::Escaped(_) => r.consume(3),
        },
        None => (),
    };
    Ok(uric)
}

fn is_uric(uric: Uric) -> bool {
    match uric {
        Uric::Simple(b) => is_reserved(b) || is_unreserved(b),
        Uric::Escaped(bytes) => is_escaped(bytes),
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
