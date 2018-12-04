mod char_stream;

#[cfg(test)]
mod tests;

//https://tools.ietf.org/html/rfc2396#appendix-A

use common_failures::prelude::*;

use std::fmt::Write;
use std::io::Read;
use uri::char_stream::Char;
use uri::char_stream::CharStream;
// path          = [ abs_path | opaque_part ]

// IPv4address   = 1*digit "." 1*digit "." 1*digit "." 1*digit

#[derive(Debug, PartialEq)]
struct IPv4address(Vec<Char>, Vec<Char>, Vec<Char>, Vec<Char>);

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

fn port<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Port>> {
    let mut digits: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = cs.next()? {
            if c.is_digit() {
                cs.consume();
                digits.push(c);
                continue;
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

fn path_segments<R: Read>(cs: &mut CharStream<R>) -> Result<Option<PathSegments>> {
    let mut segments: Vec<Segment> = Vec::new();
    match segment(cs)? {
        Some(segment) => segments.push(segment),
        None => (),
    };
    if !segments.is_empty() {
        loop {
            if let Some(c) = cs.next()? {
                if c.is(b'/') {
                    cs.consume();
                    match segment(cs)? {
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

fn segment<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Segment>> {
    let mut s = match pchars(cs)? {
        Some(pchars) => Some(Segment {
            pchars: pchars,
            params: None,
        }),
        None => None,
    };
    if let Some(s) = &mut s {
        let mut params: Vec<Param> = Vec::new();
        loop {
            if let Some(c) = cs.next()? {
                if c.is(b';') {
                    cs.consume();
                    match param(cs)? {
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

fn param<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Param>> {
    let p = match pchars(cs)? {
        Some(pchars) => Some(Param { pchars }),
        None => None,
    };
    Ok(p)
}

fn pchars<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Vec<Char>>> {
    let mut param: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = cs.next()? {
            if c.is_pchar() {
                cs.consume();
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

fn query<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Vec<Char>>> {
    fragment(cs)
}

fn fragment<R: Read>(cs: &mut CharStream<R>) -> Result<Option<Vec<Char>>> {
    let mut fragment: Vec<Char> = Vec::new();
    loop {
        if let Some(c) = cs.next()? {
            if c.is_uric() {
                cs.consume();
                fragment.push(c);
            } else {
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
