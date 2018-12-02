use super::*;

use std::io::prelude::*;

use std::io::BufReader;

use uri::char_stream::CharStream;

#[test]
fn test_path_segments() {
    CharStream::new("".as_bytes());

    let mut r = BufReader::new("".as_bytes());
    let ps = path_segments(&mut r).unwrap();
    assert_eq!(None, ps);

    let mut r = BufReader::new("foo".as_bytes());
    let ps = path_segments(&mut r).unwrap().unwrap();
    assert_eq!("foo", ps.to_string());
    let segments = ps.segments;
    assert_eq!(1, segments.len());

    let mut r = BufReader::new("foo/".as_bytes());
    let ps = path_segments(&mut r).unwrap().unwrap();
    assert_eq!("foo/", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());

    let mut r = BufReader::new("foo/bar".as_bytes());
    let ps = path_segments(&mut r).unwrap().unwrap();
    assert_eq!("foo/bar", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());

    let mut r = BufReader::new("foo;bar/bar".as_bytes());
    let ps = path_segments(&mut r).unwrap().unwrap();
    assert_eq!("foo;bar/bar", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());
}

#[test]
fn test_segment() {
    let mut r = BufReader::new("".as_bytes());
    let p = segment(&mut r).unwrap();
    assert_eq!(None, p);

    let mut r = BufReader::new("foo".as_bytes());
    let p = segment(&mut r).unwrap().unwrap();
    assert_eq!(
        vec![Char::Normal(b'f'), Char::Normal(b'o'), Char::Normal(b'o')],
        p.pchars
    );
    assert_eq!(None, p.params);

    let mut r = BufReader::new("foo;bar;buh".as_bytes());
    let p = segment(&mut r).unwrap().unwrap();
    assert_eq!(
        vec![Char::Normal(b'f'), Char::Normal(b'o'), Char::Normal(b'o')],
        p.pchars
    );
    let params = p.params.unwrap();
    assert_eq!(2, params.len());
    assert_eq!("bar", params[0].to_string());
    assert_eq!("buh", params[1].to_string());

    let mut r = BufReader::new("foo;".as_bytes());
    let p = segment(&mut r).unwrap().unwrap();
    assert_eq!(
        vec![Char::Normal(b'f'), Char::Normal(b'o'), Char::Normal(b'o')],
        p.pchars
    );
    let params = p.params.unwrap();
    assert_eq!(1, params.len());
    assert_eq!("", params[0].to_string());
}

#[test]
fn test_param() {
    let mut r = BufReader::new("".as_bytes());
    let p = param(&mut r).unwrap();
    assert_eq!(None, p);

    let mut r = BufReader::new("f oo".as_bytes());
    let p = param(&mut r).unwrap().unwrap();
    assert_eq!(
        Param {
            pchars: vec![Char::Normal(b'f')]
        },
        p
    );
    let p = param(&mut r).unwrap();
    assert_eq!(None, p);
    let c = next_char(&mut r).unwrap().unwrap();
    consume_char(&mut r, &c);
    assert_eq!(Char::Normal(b' '), c);
    let p = param(&mut r).unwrap().unwrap();
    assert_eq!(
        Param {
            pchars: vec![Char::Normal(b'o'), Char::Normal(b'o')]
        },
        p
    );
    let p = param(&mut r).unwrap();
    assert_eq!(None, p);
}
