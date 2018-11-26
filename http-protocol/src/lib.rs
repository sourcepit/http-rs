//https://tools.ietf.org/html/rfc2616

// SP             = <US-ASCII SP, space (32)>
// CTL            = <any US-ASCII control character
//                         (octets 0 - 31) and DEL (127)>

// HTTP-message   = Request | Response

// generic-message = start-line
//                           *(message-header CRLF)
//                           CRLF
//                           [ message-body ]
// start-line      = Request-Line | Status-Line

// Request-Line   = Method SP Request-URI SP HTTP-Version CRLF

// Method         = "OPTIONS"                ; Section 9.2
//                       | "GET"                    ; Section 9.3
//                       | "HEAD"                   ; Section 9.4
//                       | "POST"                   ; Section 9.5
//                       | "PUT"                    ; Section 9.6
//                       | "DELETE"                 ; Section 9.7
//                       | "TRACE"                  ; Section 9.8
//                       | "CONNECT"                ; Section 9.9
//                       | extension-method
// extension-method = token

// token          = 1*<any CHAR except CTLs or separators>
//        separators     = "(" | ")" | "<" | ">" | "@"
//                       | "," | ";" | ":" | "\" | <">
//                       | "/" | "[" | "]" | "?" | "="
//                       | "{" | "}" | SP | HT

// Request-URI    = "*" | absoluteURI | abs_path | authority

// HTTP-Version   = "HTTP" "/" 1*DIGIT "." 1*DIGIT
#[macro_use]
extern crate common_failures;
#[macro_use]
extern crate failure;

pub mod uri;

use common_failures::prelude::*;

use std::io::BufRead;

enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Extension(String),
}

impl From<String> for Method {
    fn from(string: String) -> Method {
        match string.as_str() {
            "OPTIONS" => Method::Options,
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "TRACE" => Method::Trace,
            "CONNECT" => Method::Connect,
            _ => Method::Extension(string),
        }
    }
}

fn method(r: &mut BufRead) -> Result<Option<Method>> {
    match next_token(r)? {
        Some(token) => Ok(Some(Method::from(token))),
        None => Ok(None),
    }
}

struct ReadStep {
    len: usize,
    done: bool,
}

fn next_token(r: &mut BufRead) -> Result<Option<String>> {
    let mut token: String = String::new();
    while {
        let step: ReadStep = {
            let buf: &[u8] = r.fill_buf()?;
            let mut len: usize = 0;
            let mut done: bool = false;
            for b in buf {
                if is_ctl(*b) || is_separator(*b) {
                    done = true;
                    break;
                } else {
                    token.push(*b as char);
                    len = len + 1;
                }
            }
            ReadStep {
                len: len,
                done: done || len == 0,
            }
        };
        r.consume(step.len);
        !step.done
    } {}
    let token = match token.len() {
        0 => None,
        _ => Some(token),
    };
    Ok(token)
}

fn is_separator(c: u8) -> bool {
    match c {
        b'(' => true,
        b')' => true,
        b'<' => true,
        b'>' => true,
        b'@' => true,
        b',' => true,
        b';' => true,
        b':' => true,
        b'\\' => true,
        b'"' => true,
        b'/' => true,
        b'[' => true,
        b']' => true,
        b'?' => true,
        b'=' => true,
        b'{' => true,
        b'}' => true,
        b' ' => true,
        b'\t' => true,
        _ => false,
    }
}

fn is_ctl(c: u8) -> bool {
    c < 32 || c == 127
}

fn is_sp(c: u8) -> bool {
    c == 32
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;

    use std::io::BufReader;

    #[test]
    fn test_next_token() {
        let mut r = BufReader::new("".as_bytes());
        let t = next_token(&mut r).unwrap();
        assert_eq!(None, t);

        let mut r = BufReader::new("hello".as_bytes());
        let t = next_token(&mut r).unwrap();
        assert_eq!(Some(String::from("hello")), t);

        let mut r = BufReader::new("hello world".as_bytes());
        let t = next_token(&mut r).unwrap();
        assert_eq!(Some(String::from("hello")), t);

        let t = next_token(&mut r).unwrap();
        assert_eq!(None, t);

        let mut b: [u8; 1] = [0; 1];
        r.read_exact(&mut b).unwrap();
        assert_eq!(b' ', b[0]);

        let t = next_token(&mut r).unwrap();
        assert_eq!(Some(String::from("world")), t);

        let t = next_token(&mut r).unwrap();
        assert_eq!(None, t);
    }
}
