//https://tools.ietf.org/html/rfc2396#appendix-A

use common_failures::prelude::*;

use std::io::BufRead;

// pchar  = unreserved | escaped | ":" | "@" | "&" | "=" | "+" | "$" | ","

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
