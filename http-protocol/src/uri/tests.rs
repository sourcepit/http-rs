use super::*;

use uri::char_stream::CharStream;

#[test]
fn test_path_segments() {
    let mut cs = CharStream::new("".as_bytes());
    let ps = path_segments(&mut cs).unwrap();
    assert_eq!(None, ps);

    let mut cs = CharStream::new("foo".as_bytes());
    let ps = path_segments(&mut cs).unwrap().unwrap();
    assert_eq!("foo", ps.to_string());
    let segments = ps.segments;
    assert_eq!(1, segments.len());

    let mut cs = CharStream::new("foo/".as_bytes());
    let ps = path_segments(&mut cs).unwrap().unwrap();
    assert_eq!("foo/", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());

    let mut cs = CharStream::new("foo/bar".as_bytes());
    let ps = path_segments(&mut cs).unwrap().unwrap();
    assert_eq!("foo/bar", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());

    let mut cs = CharStream::new("foo;bar/bar".as_bytes());
    let ps = path_segments(&mut cs).unwrap().unwrap();
    assert_eq!("foo;bar/bar", ps.to_string());
    let segments = ps.segments;
    assert_eq!(2, segments.len());
}

#[test]
fn test_segment() {
    let mut cs = CharStream::new("".as_bytes());
    let p = segment(&mut cs).unwrap();
    assert_eq!(None, p);

    let mut cs = CharStream::new("foo".as_bytes());
    let p = segment(&mut cs).unwrap().unwrap();
    assert_eq!(
        vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
        p.pchars
    );
    assert_eq!(None, p.params);

    let mut cs = CharStream::new("foo;bar;buh".as_bytes());
    let p = segment(&mut cs).unwrap().unwrap();
    assert_eq!(
        vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
        p.pchars
    );
    let params = p.params.unwrap();
    assert_eq!(2, params.len());
    assert_eq!("bar", params[0].to_string());
    assert_eq!("buh", params[1].to_string());

    let mut cs = CharStream::new("foo;".as_bytes());
    let p = segment(&mut cs).unwrap().unwrap();
    assert_eq!(
        vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
        p.pchars
    );
    let params = p.params.unwrap();
    assert_eq!(1, params.len());
    assert_eq!("", params[0].to_string());
}

#[test]
fn test_param() {
    let mut cs = CharStream::new("".as_bytes());
    let p = param(&mut cs).unwrap();
    assert_eq!(None, p);

    let mut cs = CharStream::new("f oo".as_bytes());
    let p = param(&mut cs).unwrap().unwrap();
    assert_eq!(
        Param {
            pchars: vec![Char::Ascii(b'f')]
        },
        p
    );
    let p = param(&mut cs).unwrap();
    assert_eq!(None, p);
    let c = cs.next().unwrap().unwrap();
    cs.consume().unwrap();
    assert_eq!(Char::Ascii(b' '), c);
    let p = param(&mut cs).unwrap().unwrap();
    assert_eq!(
        Param {
            pchars: vec![Char::Ascii(b'o'), Char::Ascii(b'o')]
        },
        p
    );
    let p = param(&mut cs).unwrap();
    assert_eq!(None, p);
}
