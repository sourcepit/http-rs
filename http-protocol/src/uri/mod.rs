mod char_stream;
mod token_buffer;

//https://tools.ietf.org/html/rfc2396#appendix-A

use common_failures::prelude::*;

use std::fmt::Write;
use uri::char_stream::Char;
use uri::token_buffer::TokenStream;
use uri::token_buffer::*;

fn uri<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Uri>>
where
    T: TokenStream<Char>,
{
    let u: Option<Uri>;
    if let Some(au) = absolute_uri(tb)? {
        let f: Option<String>;
        if let Some(t) = tb.pop()? {
            if t.is(b'#') {
                f = Some(fragment(tb)?.to_string());
            } else {
                tb.push(t);
                f = None;
            }
        } else {
            f = None;
        }
        u = Some(Uri::AbsoluteUri(au, f));
    } else if let Some(ru) = relative_uri(tb)? {
        let f: Option<String>;
        if let Some(t) = tb.pop()? {
            if t.is(b'#') {
                f = Some(fragment(tb)?.to_string());
            } else {
                tb.push(t);
                f = None;
            }
        } else {
            f = None;
        }
        u = Some(Uri::RelativeUri(ru, f));
    } else {
        u = None;
    }
    Ok(u)
}

#[derive(Debug, PartialEq)]
enum Uri {
    AbsoluteUri(AbsoluteUri, Option<String>),
    RelativeUri(RelativeUri, Option<String>),
}

impl Uri {
    pub fn is_absolute(&self) -> bool {
        match self {
            Uri::AbsoluteUri(_, _) => true,
            _ => false,
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            Uri::AbsoluteUri(uri, _) => match &uri.1 {
                HierOrOpaquePart::OpaquePart(_) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_relative(&self) -> bool {
        match self {
            Uri::RelativeUri(_, _) => true,
            _ => false,
        }
    }

    pub fn scheme(&self) -> Option<&Scheme> {
        match self {
            Uri::AbsoluteUri(uri, _) => Some(&uri.0),
            _ => None,
        }
    }

    fn net_path(&self) -> Option<&NetPath> {
        match self {
            Uri::AbsoluteUri(uri, _) => match &uri.1 {
                HierOrOpaquePart::HierPart(hier_part) => match &hier_part.0 {
                    HierPartPath::NetPath(path) => Some(&path),
                    _ => None,
                },
                _ => None,
            },
            Uri::RelativeUri(uri, _) => match &uri.0 {
                RelativeUriPath::NetPath(path) => Some(&path),
                _ => None,
            },
        }
    }

    pub fn userinfo(&self) -> Option<&String> {
        match self.net_path() {
            Some(net_path) => match &net_path.authority {
                Authority::Server(server) => match &server.0 {
                    Some(userinfo) => Some(&userinfo),
                    _ => None,
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn host(&self) -> Option<&Host> {
        match self.net_path() {
            Some(net_path) => match &net_path.authority {
                Authority::Server(server) => {
                    let hostport = &server.1;
                    Some(&hostport.0)
                }
                _ => None,
            },
            None => None,
        }
    }

    pub fn port(&self) -> Option<u16> {
        match self.net_path() {
            Some(net_path) => match &net_path.authority {
                Authority::Server(server) => {
                    let hostport = &server.1;
                    match &hostport.1 {
                        Some(port) => {
                            let port_string = port.to_string();
                            match port_string.is_empty() {
                                true => None,
                                false => Some(port_string.parse::<u16>().unwrap()),
                            }
                        }
                        None => None,
                    }
                }
                _ => None,
            },
            None => None,
        }
    }

    pub fn path(&self) -> Option<&String> {
        match self {
            Uri::AbsoluteUri(uri, _) => match &uri.1 {
                HierOrOpaquePart::HierPart(hier_part) => match &hier_part.0 {
                    HierPartPath::NetPath(net_path) => match &net_path.abs_path {
                        Some(abs_path) => Some(&abs_path),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
            Uri::RelativeUri(uri, _) => match &uri.0 {
                RelativeUriPath::NetPath(net_path) => match &net_path.abs_path {
                    Some(abs_path) => Some(&abs_path),
                    _ => None,
                },
                RelativeUriPath::AbsPath(abs_path) => Some(&abs_path),
                RelativeUriPath::RelPath(rel_path) => Some(&rel_path),
            },
        }
    }

    pub fn query(&self) -> Option<&String> {
        match self {
            Uri::AbsoluteUri(uri, _) => match &uri.1 {
                HierOrOpaquePart::HierPart(hier_part) => match &hier_part.1 {
                    Some(query) => Some(&query),
                    _ => None,
                },
                _ => None,
            },
            Uri::RelativeUri(uri, _) => match &uri.1 {
                Some(query) => Some(&query),
                _ => None,
            },
        }
    }

    pub fn fragment(&self) -> Option<&String> {
        match self {
            Uri::AbsoluteUri(_, fragment) => match fragment {
                Some(fragment) => Some(&fragment),
                _ => None,
            },
            Uri::RelativeUri(_, fragment) => match fragment {
                Some(fragment) => Some(&fragment),
                _ => None,
            },
        }
    }

    pub fn opaque_part(&self) -> Option<&String> {
        match self {
            Uri::AbsoluteUri(uri, _) => match &uri.1 {
                HierOrOpaquePart::OpaquePart(opaque_part) => Some(&opaque_part),
                _ => None,
            },
            _ => None,
        }
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Uri::AbsoluteUri(u, f) => {
                fmt.write_str(u.to_string().as_str())?;
                if let Some(f) = f {
                    fmt.write_char('#')?;
                    fmt.write_str(f.to_string().as_str())?;
                }
            }
            Uri::RelativeUri(u, f) => {
                fmt.write_str(u.to_string().as_str())?;
                if let Some(f) = f {
                    fmt.write_char('#')?;
                    fmt.write_str(f.to_string().as_str())?;
                }
            }
        };
        Ok(())
    }
}

fn absolute_uri<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<AbsoluteUri>>
where
    T: TokenStream<Char>,
{
    let au: Option<AbsoluteUri>;
    if let Some(s) = scheme(tb)? {
        if let Some(t) = tb.pop()? {
            if t.is(b':') {
                let hop: Option<HierOrOpaquePart>;
                if let Some(hp) = hier_part(tb)? {
                    hop = Some(HierOrOpaquePart::HierPart(hp));
                } else if let Some(op) = opaque_part(tb)? {
                    hop = Some(HierOrOpaquePart::OpaquePart(op.to_string()));
                } else {
                    hop = None;
                }
                if let Some(hop) = hop {
                    au = Some(AbsoluteUri(s, hop));
                } else {
                    tb.push(t);
                    tb.push_tokens(s.0);
                    au = None;
                }
            } else {
                tb.push(t);
                tb.push_tokens(s.0);
                au = None;
            }
        } else {
            tb.push_tokens(s.0);
            au = None;
        }
    } else {
        au = None;
    }
    Ok(au)
}

#[derive(Debug, PartialEq)]
struct AbsoluteUri(Scheme, HierOrOpaquePart);

impl std::fmt::Display for AbsoluteUri {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.0.to_string().as_str())?;
        fmt.write_char(':')?;
        fmt.write_str(self.1.to_string().as_str())?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum HierOrOpaquePart {
    HierPart(HierPart),
    OpaquePart(String),
}

impl std::fmt::Display for HierOrOpaquePart {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HierOrOpaquePart::HierPart(o) => fmt.write_str(o.to_string().as_str()),
            HierOrOpaquePart::OpaquePart(o) => fmt.write_str(o.to_string().as_str()),
        }
    }
}

fn relative_uri<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<RelativeUri>>
where
    T: TokenStream<Char>,
{
    let rup: Option<RelativeUriPath>;
    if let Some(np) = net_path(tb)? {
        rup = Some(RelativeUriPath::NetPath(np));
    } else if let Some(ap) = abs_path(tb)? {
        rup = Some(RelativeUriPath::AbsPath(ap.to_string()));
    } else if let Some(rp) = rel_path(tb)? {
        rup = Some(RelativeUriPath::RelPath(rp.to_string()));
    } else {
        rup = None;
    }
    let ru: Option<RelativeUri>;
    if let Some(rup) = rup {
        let q: Option<String>;
        if let Some(t) = tb.pop()? {
            if t.is(b'?') {
                q = Some(query(tb)?.to_string());
            } else {
                tb.push(t);
                q = None;
            }
        } else {
            q = None;
        }
        ru = Some(RelativeUri(rup, q));
    } else {
        ru = None;
    }
    Ok(ru)
}

#[derive(Debug, PartialEq)]
struct RelativeUri(RelativeUriPath, Option<String>);

impl std::fmt::Display for RelativeUri {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string().as_str())?;
        if let Some(q) = &self.1 {
            fmt.write_char('?')?;
            fmt.write_str(q.to_string().as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum RelativeUriPath {
    NetPath(NetPath),
    AbsPath(String),
    RelPath(String),
}

impl std::fmt::Display for RelativeUriPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RelativeUriPath::NetPath(o) => fmt.write_str(o.to_string().as_str()),
            RelativeUriPath::AbsPath(o) => fmt.write_str(o.to_string().as_str()),
            RelativeUriPath::RelPath(o) => fmt.write_str(o.to_string().as_str()),
        }
    }
}

fn hier_part<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<HierPart>>
where
    T: TokenStream<Char>,
{
    let hpp: Option<HierPartPath>;
    if let Some(np) = net_path(tb)? {
        hpp = Some(HierPartPath::NetPath(np));
    } else if let Some(ap) = abs_path(tb)? {
        hpp = Some(HierPartPath::AbsPath(ap));
    } else {
        hpp = None;
    }
    let hp: Option<HierPart>;
    if let Some(hpp) = hpp {
        let q: Option<String>;
        if let Some(t) = tb.pop()? {
            if t.is(b'?') {
                q = Some(query(tb)?.to_string());
            } else {
                tb.push(t);
                q = None;
            }
        } else {
            q = None;
        }
        hp = Some(HierPart(hpp, q));
    } else {
        hp = None;
    }
    Ok(hp)
}

#[derive(Debug, PartialEq)]
struct HierPart(HierPartPath, Option<String>);

impl std::fmt::Display for HierPart {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string().as_str())?;
        if let Some(q) = &self.1 {
            fmt.write_char('?')?;
            fmt.write_str(q.to_string().as_str())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum HierPartPath {
    NetPath(NetPath),
    AbsPath(AbsPath),
}

impl std::fmt::Display for HierPartPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HierPartPath::NetPath(o) => fmt.write_str(o.to_string().as_str()),
            HierPartPath::AbsPath(o) => fmt.write_str(o.to_string().as_str()),
        }
    }
}

fn path<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Path>>
where
    T: TokenStream<Char>,
{
    let p: Option<Path>;
    if let Some(ap) = abs_path(tb)? {
        p = Some(Path::AbsPath(ap));
    } else if let Some(op) = opaque_part(tb)? {
        p = Some(Path::OpaquePart(op));
    } else {
        p = None;
    }
    Ok(p)
}

#[derive(Debug, PartialEq)]
enum Path {
    AbsPath(AbsPath),
    OpaquePart(OpaquePart),
}

impl std::fmt::Display for Path {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Path::AbsPath(o) => fmt.write_str(o.to_string().as_str()),
            Path::OpaquePart(o) => fmt.write_str(o.to_string().as_str()),
        }
    }
}

fn rel_path<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<RelPath>>
where
    T: TokenStream<Char>,
{
    let rp: Option<RelPath>;
    if let Some(rs) = rel_segment(tb)? {
        let ap = abs_path(tb)?;
        rp = Some(RelPath(rs, ap));
    } else {
        rp = None;
    }
    Ok(rp)
}

#[derive(Debug, PartialEq)]
struct RelPath(RelSegment, Option<AbsPath>);

impl std::fmt::Display for RelPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string().as_str())?;
        if let Some(ap) = &self.1 {
            fmt.write_str(ap.to_string().as_str())?;
        }
        Ok(())
    }
}

fn net_path<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<NetPath>>
where
    T: TokenStream<Char>,
{
    let np: Option<NetPath>;
    if let Some(t1) = tb.pop()? {
        if t1.is(b'/') {
            if let Some(t2) = tb.pop()? {
                if t2.is(b'/') {
                    if let Some(a) = authority(tb)? {
                        let ap = match abs_path(tb)? {
                            Some(ap) => Some(ap.to_string()),
                            None => None,
                        };
                        np = Some(NetPath {
                            authority: a,
                            abs_path: ap,
                        });
                    } else {
                        tb.push(t2);
                        tb.push(t1);
                        np = None;
                    }
                } else {
                    tb.push(t2);
                    tb.push(t1);
                    np = None;
                }
            } else {
                tb.push(t1);
                np = None;
            }
        } else {
            tb.push(t1);
            np = None;
        }
    } else {
        np = None;
    }
    Ok(np)
}

#[derive(Debug, PartialEq)]
struct NetPath {
    authority: Authority,
    abs_path: Option<String>,
}

impl std::fmt::Display for NetPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_char('/')?;
        fmt.write_char('/')?;
        fmt.write_str(&self.authority.to_string().as_str())?;
        if let Some(ap) = &self.abs_path {
            fmt.write_str(ap.to_string().as_str())?;
        }
        Ok(())
    }
}

fn abs_path<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<AbsPath>>
where
    T: TokenStream<Char>,
{
    let mut ap: Option<AbsPath> = None;
    if let Some(t) = tb.pop()? {
        if t.is(b'/') {
            if let Some(ps) = path_segments(tb)? {
                ap = Some(AbsPath(ps));
            } else {
                tb.push(t);
            }
        } else {
            tb.push(t);
        }
    }
    Ok(ap)
}

#[derive(Debug, PartialEq)]
struct AbsPath(PathSegments);

impl std::fmt::Display for AbsPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_char('/')?;
        fmt.write_str(&self.0.to_string().as_str())?;
        Ok(())
    }
}

fn opaque_part<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<OpaquePart>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    if let Some(t) = tb.pop()? {
        if t.is_uric_no_slash() {
            tokens.push(t);
        } else {
            tb.push(t);
        }
    }
    if !tokens.is_empty() {
        loop {
            if let Some(t) = tb.pop()? {
                if t.is_uric() {
                    tokens.push(t);
                    continue;
                }
                tb.push(t);
            }
            break;
        }
    }
    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(OpaquePart(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct OpaquePart(Vec<Char>);

impl std::fmt::Display for OpaquePart {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn rel_segment<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<RelSegment>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    loop {
        if let Some(t) = tb.pop()? {
            if t.is_unreserved() {
                tokens.push(t);
                continue;
            }
            if t.is_escaped() {
                tokens.push(t);
                continue;
            }
            if let Char::Ascii(b) = t {
                match b {
                    b';' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' => {
                        tokens.push(t);
                        continue;
                    }
                    _ => (),
                };
            }
            tb.push(t);
        }
        break;
    }
    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(RelSegment(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct RelSegment(Vec<Char>);

impl std::fmt::Display for RelSegment {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn scheme<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Scheme>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    if let Some(t) = tb.pop()? {
        if t.is_alpha() {
            tokens.push(t);
        } else {
            tb.push(t);
        }
    }
    if !tokens.is_empty() {
        loop {
            if let Some(t) = tb.pop()? {
                if t.is_alpha() || t.is_digit() || t.is(b'+') || t.is(b'-') || t.is(b'.') {
                    tokens.push(t);
                    continue;
                }
                tb.push(t);
            }
            break;
        }
    }
    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(Scheme(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct Scheme(Vec<Char>);

impl std::fmt::Display for Scheme {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn authority<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Authority>>
where
    T: TokenStream<Char>,
{
    let a: Option<Authority>;
    if let Some(s) = server(tb)? {
        a = Some(Authority::Server(s));
    } else if let Some(r) = reg_name(tb)? {
        a = Some(Authority::RegName(r));
    } else {
        a = None;
    }
    Ok(a)
}

#[derive(Debug, PartialEq)]
enum Authority {
    Server(Server),
    RegName(RegName),
}

impl std::fmt::Display for Authority {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Authority::Server(o) => fmt.write_str(o.to_string().as_str()),
            Authority::RegName(o) => fmt.write_str(o.to_string().as_str()),
        }
    }
}

fn reg_name<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<RegName>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    loop {
        if let Some(t) = tb.pop()? {
            if t.is_unreserved() {
                tokens.push(t);
                continue;
            }
            if t.is_escaped() {
                tokens.push(t);
                continue;
            }
            if let Char::Ascii(b) = t {
                match b {
                    b'$' | b',' | b';' | b':' | b'@' | b'&' | b'=' | b'+' => {
                        tokens.push(t);
                        continue;
                    }
                    _ => (),
                };
            }
            tb.push(t);
        }
        break;
    }
    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(RegName(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct RegName(Vec<Char>);

impl std::fmt::Display for RegName {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn server<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Server>>
where
    T: TokenStream<Char>,
{
    let mut ui = userinfo(tb)?;

    let ui = match tb.pop()? {
        Some(t) => match t.is(b'@') {
            true => Some(ui),
            false => {
                tb.push(t);
                tb.push_tokens(ui.0);
                None
            }
        },
        None => {
            tb.push_tokens(ui.0);
            None
        }
    };

    let s = match hostport(tb)? {
        Some(hp) => {
            let ui = match ui {
                Some(ui) => Some(ui.to_string()),
                None => None,
            };
            Some(Server(ui, hp))
        }
        None => match ui {
            Some(ui) => {
                tb.push(Char::Ascii(b'@'));
                tb.push_tokens(ui.0);
                None
            }
            None => None,
        },
    };

    Ok(s)
}

#[derive(Debug, PartialEq)]
struct Server(Option<String>, Hostport);

impl std::fmt::Display for Server {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(u) = &self.0 {
            fmt.write_str(u.to_string().as_str())?;
            fmt.write_char('@')?;
        }
        fmt.write_str(self.1.to_string().as_str())?;
        Ok(())
    }
}

fn userinfo<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Userinfo>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    loop {
        if let Some(t) = tb.pop()? {
            let is_match = match t {
                Char::Escaped(_) => true,
                Char::Ascii(b) => {
                    t.is_unreserved() || match b {
                        b';' => true,
                        b':' => true,
                        b'&' => true,
                        b'=' => true,
                        b'+' => true,
                        b'$' => true,
                        b',' => true,
                        _ => false,
                    }
                }
            };
            if is_match {
                tokens.push(t);
                continue;
            } else {
                tb.push(t);
            }
        }
        break;
    }
    Ok(Userinfo(tokens))
}

#[derive(Debug, PartialEq)]
struct Userinfo(Vec<Char>);

impl std::fmt::Display for Userinfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn hostport<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Hostport>>
where
    T: TokenStream<Char>,
{
    let mut hn: Option<Hostport> = None;
    if let Some(ho) = host(tb)? {
        let mut po: Option<Port> = None;
        if let Some(c) = tb.pop()? {
            if c.is(b':') {
                po = Some(port(tb)?);
            } else {
                tb.push(c);
            }
        }
        hn = Some(Hostport(ho, po));
    }
    Ok(hn)
}

#[derive(Debug, PartialEq)]
struct Hostport(Host, Option<Port>);

impl std::fmt::Display for Hostport {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.0.to_string().as_str())?;
        if let Some(po) = &self.1 {
            fmt.write_char(':')?;
            fmt.write_str(po.to_string().as_str())?;
        }
        Ok(())
    }
}

fn host<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Host>>
where
    T: TokenStream<Char>,
{
    let host: Option<Host>;
    if let Some(hn) = hostname(tb)? {
        host = Some(Host::Hostname(hn));
    } else if let Some(ia) = ipv4_address(tb)? {
        host = Some(Host::IPv4address(ia));
    } else {
        host = None;
    }
    Ok(host)
}

#[derive(Debug, PartialEq)]
enum Host {
    Hostname(Hostname),
    IPv4address(IPv4address),
}

impl std::fmt::Display for Host {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Host::Hostname(hn) => fmt.write_str(hn.to_string().as_str()),
            Host::IPv4address(ip) => fmt.write_str(ip.to_string().as_str()),
        }
    }
}

fn hostname<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Hostname>>
where
    T: TokenStream<Char>,
{
    let mut domainlabels: Vec<Domainlabel> = Vec::new();
    loop {
        if let Some(dl) = domainlabel(tb)? {
            if let Some(c) = tb.pop()? {
                if c.is(b'.') {
                    domainlabels.push(dl);
                } else {
                    tb.push(c);
                    tb.push_tokens(dl.0);
                    break;
                }
            } else {
                tb.push_tokens(dl.0);
                break;
            }
        } else {
            break;
        }
    }

    let mut tl: Option<Toplabel> = toplabel(tb)?;
    let mut dot: Option<Char> = None;

    if let Some(_) = tl {
        if let Some(c) = tb.pop()? {
            if c.is(b'.') {
                dot = Some(c);
            } else {
                tb.push(c);
            }
        }
    } else if !domainlabels.is_empty() {
        let last_idx = domainlabels.len() - 1;
        let dl = domainlabels.remove(last_idx);

        tb.push(Char::Ascii(b'.'));
        tb.push_tokens(dl.0);

        tl = toplabel(tb)?;

        if tl.is_some() {
            dot = tb.pop()?
        } else {
            for _ in 0..domainlabels.len() {
                tb.push(Char::Ascii(b'.'));

                let last_idx = domainlabels.len() - 1;
                let dl = domainlabels.remove(last_idx);
                tb.push_tokens(dl.0);
            }
        }
    }

    match tl {
        Some(tl) => Ok(Some(Hostname(domainlabels, tl, dot))),
        None => Ok(None),
    }
}

#[derive(Debug, PartialEq)]
struct Hostname(Vec<Domainlabel>, Toplabel, Option<Char>);

impl std::fmt::Display for Hostname {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for dl in &self.0 {
            fmt.write_str(dl.to_string().as_str())?;
            fmt.write_char('.')?;
        }
        fmt.write_str(&self.1.to_string().as_str())?;
        if let Some(_) = &self.2 {
            fmt.write_char('.')?;
        }
        Ok(())
    }
}

fn domainlabel<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Domainlabel>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    if let Some(c) = tb.pop()? {
        if c.is_alphanum() {
            tokens.push(c);
            loop {
                if let Some(c) = tb.pop()? {
                    if c.is_alphanum() || c.is(b'-') {
                        tokens.push(c);
                    } else {
                        tb.push(c);
                        break;
                    }
                } else {
                    break;
                }
            }
            let last_is_alphanum = tokens.last().unwrap().is_alphanum();
            if !last_is_alphanum {
                tb.push_tokens(tokens);
                tokens = Vec::new();
            }
        } else {
            tb.push(c);
        }
    }

    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(Domainlabel(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct Domainlabel(Vec<Char>);

impl std::fmt::Display for Domainlabel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn toplabel<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<Toplabel>>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    if let Some(c) = tb.pop()? {
        if c.is_alpha() {
            tokens.push(c);
            loop {
                if let Some(c) = tb.pop()? {
                    if c.is_alphanum() || c.is(b'-') {
                        tokens.push(c);
                    } else {
                        tb.push(c);
                        break;
                    }
                } else {
                    break;
                }
            }

            let last_is_alphanum = tokens.last().unwrap().is_alphanum();
            if !last_is_alphanum {
                tb.push_tokens(tokens);
                tokens = Vec::new();
            }
        } else {
            tb.push(c);
        }
    }

    let tokens_len = tokens.len();

    match tokens.is_empty() {
        true => Ok(None),
        false => Ok(Some(Toplabel(tokens))),
    }
}

#[derive(Debug, PartialEq)]
struct Toplabel(Vec<Char>);

impl std::fmt::Display for Toplabel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn ipv4_address<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Option<IPv4address>>
where
    T: TokenStream<Char>,
{
    let d1 = digits(tb)?;
    if d1.is_empty() {
        return Ok(None);
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

    let d2 = digits(tb)?;

    if d2.is_empty() {
        tb.push(dot1);
        tb.push_tokens(d1);
        return Ok(None);
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

    let d3 = digits(tb)?;
    if d3.is_empty() {
        tb.push(dot2);
        tb.push_tokens(d2);
        tb.push(dot1);
        tb.push_tokens(d1);
        return Ok(None);
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

    let d4 = digits(tb)?;
    if d4.is_empty() {
        tb.push(dot3);
        tb.push_tokens(d3);
        tb.push(dot2);
        tb.push_tokens(d2);
        tb.push(dot1);
        tb.push_tokens(d1);
        return Ok(None);
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
struct Port(Vec<Char>);

impl std::fmt::Display for Port {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for d in &self.0 {
            fmt.write_str(d.to_string().as_str())?;
        }
        Ok(())
    }
}

fn port<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Port>
where
    T: TokenStream<Char>,
{
    let d = digits(tb)?;
    Ok(Port(d))
}

fn digits<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Vec<Char>>
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
    Ok(digits)
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

fn query<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Query>
where
    T: TokenStream<Char>,
{
    Ok(Query(fragment(tb)?.0))
}

#[derive(Debug, PartialEq)]
struct Query(Vec<Char>);

impl std::fmt::Display for Query {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

fn fragment<T>(tb: &mut TokenBuffer<Char, T>) -> Result<Fragment>
where
    T: TokenStream<Char>,
{
    let mut tokens: Vec<Char> = Vec::new();
    loop {
        if let Some(t) = tb.pop()? {
            if t.is_uric() {
                tokens.push(t);
            } else {
                tb.push(t);
                break;
            }
        } else {
            break;
        }
    }
    Ok(Fragment(tokens))
}

#[derive(Debug, PartialEq)]
struct Fragment(Vec<Char>);

impl std::fmt::Display for Fragment {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for c in &self.0 {
            fmt.write_str(c.to_string().as_str())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_uri() {
        let uri_str = "http://user:pwd@www.sourcepit.org:123/foo/bar.html?query=true#fragment";

        let mut tb = TokenBuffer::from(uri_str.as_bytes());
        assert_eq!(0, tb.buffer.len());

        let u = uri(&mut tb).unwrap();
        assert!(u.is_some());

        let u = u.unwrap();
        assert_eq!(true, u.is_absolute());
        assert_eq!(false, u.is_opaque());
        assert_eq!(false, u.is_relative());

        let scheme = u.scheme();
        assert!(scheme.is_some());
        let scheme = scheme.unwrap();
        assert_eq!("http", scheme.to_string());

        let userinfo = u.userinfo();
        assert!(userinfo.is_some());
        let userinfo = userinfo.unwrap();
        assert_eq!("user:pwd", userinfo);

        let host = u.host();
        assert!(host.is_some());
        let host = host.unwrap();
        assert_eq!("www.sourcepit.org", host.to_string());

        let port = u.port();
        assert!(port.is_some());
        let port = port.unwrap();
        assert_eq!(123, port);

        let path = u.path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!("/foo/bar.html", path);

        let query = u.query();
        assert!(query.is_some());
        let query = query.unwrap();
        assert_eq!("query=true", query);

        let fragment = u.fragment();
        assert!(fragment.is_some());
        let fragment = fragment.unwrap();
        assert_eq!("fragment", fragment);

        assert_eq!(uri_str, u.to_string());
    }

    #[test]
    fn test_opaque_uri() {
        let uri_str = "mailto:a@b.com#fragment";

        let mut tb = TokenBuffer::from(uri_str.as_bytes());
        assert_eq!(0, tb.buffer.len());

        let u = uri(&mut tb).unwrap();
        assert!(u.is_some());

        let u = u.unwrap();
        assert_eq!(true, u.is_absolute());
        assert_eq!(true, u.is_opaque());
        assert_eq!(false, u.is_relative());

        let scheme = u.scheme();
        assert!(scheme.is_some());
        let scheme = scheme.unwrap();
        assert_eq!("mailto", scheme.to_string());

        let opaque_part = u.opaque_part();
        assert!(opaque_part.is_some());
        let opaque_part = opaque_part.unwrap();
        assert_eq!("a@b.com", opaque_part.to_string());

        let userinfo = u.userinfo();
        assert!(userinfo.is_none());

        let host = u.host();
        assert!(host.is_none());

        let path = u.path();
        assert!(path.is_none());

        let query = u.query();
        assert!(query.is_none());

        let fragment = u.fragment();
        assert!(fragment.is_some());
        let fragment = fragment.unwrap();
        assert_eq!("fragment", fragment);

        assert_eq!(uri_str, u.to_string());
    }

    #[test]
    fn test_relative_uri_with_net_path() {
        let uri_str = "//user:pwd@www.sourcepit.org:123/foo/bar.html?query=true#fragment";

        let mut tb = TokenBuffer::from(uri_str.as_bytes());
        assert_eq!(0, tb.buffer.len());

        let u = uri(&mut tb).unwrap();
        assert!(u.is_some());

        let u = u.unwrap();
        assert_eq!(false, u.is_absolute());
        assert_eq!(false, u.is_opaque());
        assert_eq!(true, u.is_relative());

        let scheme = u.scheme();
        assert!(scheme.is_none());

        let userinfo = u.userinfo();
        assert!(userinfo.is_some());
        let userinfo = userinfo.unwrap();
        assert_eq!("user:pwd", userinfo);

        let host = u.host();
        assert!(host.is_some());
        let host = host.unwrap();
        assert_eq!("www.sourcepit.org", host.to_string());

        let port = u.port();
        assert!(port.is_some());
        let port = port.unwrap();
        assert_eq!(123, port);

        let path = u.path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!("/foo/bar.html", path);

        let query = u.query();
        assert!(query.is_some());
        let query = query.unwrap();
        assert_eq!("query=true", query);

        let fragment = u.fragment();
        assert!(fragment.is_some());
        let fragment = fragment.unwrap();
        assert_eq!("fragment", fragment);

        assert_eq!(uri_str, u.to_string());
    }

    #[test]
    fn test_relative_uri_with_abs_path() {
        let uri_str = "/foo/bar.html?query=true#fragment";

        let mut tb = TokenBuffer::from(uri_str.as_bytes());
        assert_eq!(0, tb.buffer.len());

        let u = uri(&mut tb).unwrap();
        assert!(u.is_some());

        let u = u.unwrap();
        assert_eq!(false, u.is_absolute());
        assert_eq!(false, u.is_opaque());
        assert_eq!(true, u.is_relative());

        let scheme = u.scheme();
        assert!(scheme.is_none());

        let userinfo = u.userinfo();
        assert!(userinfo.is_none());

        let host = u.host();
        assert!(host.is_none());

        let path = u.path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!("/foo/bar.html", path);

        let query = u.query();
        assert!(query.is_some());
        let query = query.unwrap();
        assert_eq!("query=true", query);

        let fragment = u.fragment();
        assert!(fragment.is_some());
        let fragment = fragment.unwrap();
        assert_eq!("fragment", fragment);

        assert_eq!(uri_str, u.to_string());
    }

    #[test]
    fn test_relative_uri_with_rel_path() {
        let uri_str = "foo/bar.html?query=true#fragment";

        let mut tb = TokenBuffer::from(uri_str.as_bytes());
        assert_eq!(0, tb.buffer.len());

        let u = uri(&mut tb).unwrap();
        assert!(u.is_some());

        let u = u.unwrap();
        assert_eq!(false, u.is_absolute());
        assert_eq!(false, u.is_opaque());
        assert_eq!(true, u.is_relative());

        let scheme = u.scheme();
        assert!(scheme.is_none());

        let userinfo = u.userinfo();
        assert!(userinfo.is_none());

        let host = u.host();
        assert!(host.is_none());

        let path = u.path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!("foo/bar.html", path);

        let query = u.query();
        assert!(query.is_some());
        let query = query.unwrap();
        assert_eq!("query=true", query);

        let fragment = u.fragment();
        assert!(fragment.is_some());
        let fragment = fragment.unwrap();
        assert_eq!("fragment", fragment);

        assert_eq!(uri_str, u.to_string());
    }

    #[test]
    fn test_server() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let s = server(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, s);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let s = server(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, s.0);
        assert_eq!("foo", s.1.to_string());

        let mut tb = TokenBuffer::from("foo@bar".as_bytes());
        let s = server(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert!(s.0.is_some());
        assert_eq!("foo", s.0.unwrap().to_string());
        assert_eq!("bar", s.1.to_string());

        let mut tb = TokenBuffer::from("foo@".as_bytes());
        let s = server(&mut tb).unwrap();
        assert_eq!(4, tb.buffer.len());
        assert_eq!(None, s);
    }

    #[test]
    fn test_hostport() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let hp = hostport(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, hp);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let hp = hostport(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("foo", hp.0.to_string());
        assert!(hp.1.is_none());
        assert_eq!("foo", hp.to_string());

        let mut tb = TokenBuffer::from("foo:".as_bytes());
        let hp = hostport(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("foo", hp.0.to_string());
        assert!(hp.1.is_some());
        assert_eq!("foo:", hp.to_string());

        let mut tb = TokenBuffer::from("foo:123".as_bytes());
        let hp = hostport(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("foo", hp.0.to_string());
        assert!(hp.1.is_some());
        assert_eq!("foo:123", hp.to_string());
    }

    #[test]
    fn test_host() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let ho = host(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, ho);

        let mut tb = TokenBuffer::from("1.2.3.4".as_bytes());
        let ho = host(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        let is_ip = match ho {
            Host::Hostname(_) => false,
            Host::IPv4address(_) => true,
        };
        assert!(is_ip);

        let mut tb = TokenBuffer::from("1.2.3.f".as_bytes());
        let ho = host(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        let is_hostname = match ho {
            Host::Hostname(_) => true,
            Host::IPv4address(_) => false,
        };
        assert!(is_hostname);
    }

    #[test]
    fn test_hostname() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let hn = hostname(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, hn);

        let mut tb = TokenBuffer::from("1.2.3.4".as_bytes());
        let hn = hostname(&mut tb).unwrap();
        assert_eq!(7, tb.buffer.len());
        assert_eq!(None, hn);

        let mut tb = TokenBuffer::from("foo".as_bytes());
        let hn = hostname(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert!(hn.0.is_empty());
        assert_eq!("foo", hn.1.to_string());
        assert!(hn.2.is_none());
        assert_eq!("foo", hn.to_string());

        let mut tb = TokenBuffer::from("foo.".as_bytes());
        let hn = hostname(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert!(hn.0.is_empty());
        assert_eq!("foo", hn.1.to_string());
        assert!(hn.2.is_some());
        assert_eq!("foo.", hn.to_string());

        let mut tb = TokenBuffer::from("123.foo.".as_bytes());
        let hn = hostname(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(1, hn.0.len());
        assert_eq!("foo", hn.1.to_string());
        assert!(hn.2.is_some());
        assert_eq!("123.foo.", hn.to_string());
    }

    #[test]
    fn test_domainlabel() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let dl = domainlabel(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, dl);

        let mut tb = TokenBuffer::from("1".as_bytes());
        let dl = domainlabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("1", dl.to_string());

        let mut tb = TokenBuffer::from("f-".as_bytes());
        let dl = domainlabel(&mut tb).unwrap();
        assert_eq!(2, tb.buffer.len());
        assert_eq!(None, dl);

        let mut tb = TokenBuffer::from("fo".as_bytes());
        let dl = domainlabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("fo", dl.to_string());

        let mut tb = TokenBuffer::from("f0".as_bytes());
        let dl = domainlabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("f0", dl.to_string());

        let mut tb = TokenBuffer::from("f-0".as_bytes());
        let dl = domainlabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("f-0", dl.to_string());
    }

    #[test]
    fn test_toplabel() {
        let mut tb = TokenBuffer::from("".as_bytes());
        let tl = toplabel(&mut tb).unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!(None, tl);

        let mut tb = TokenBuffer::from("1".as_bytes());
        let tl = toplabel(&mut tb).unwrap();
        assert_eq!(1, tb.buffer.len());
        assert_eq!(None, tl);

        let mut tb = TokenBuffer::from("f-".as_bytes());
        let tl = toplabel(&mut tb).unwrap();
        assert_eq!(2, tb.buffer.len());
        assert_eq!(None, tl);

        let mut tb = TokenBuffer::from("fo".as_bytes());
        let tl = toplabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("fo", tl.to_string());

        let mut tb = TokenBuffer::from("f0".as_bytes());
        let tl = toplabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("f0", tl.to_string());

        let mut tb = TokenBuffer::from("f-0".as_bytes());
        let tl = toplabel(&mut tb).unwrap().unwrap();
        assert_eq!(0, tb.buffer.len());
        assert_eq!("f-0", tl.to_string());
    }

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

        let q = query(&mut tb)?;
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            q.0
        );

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'}'), c);

        let q = query(&mut tb)?;
        assert_eq!(
            vec![Char::Ascii(b'b'), Char::Ascii(b'a'), Char::Ascii(b'r')],
            q.0
        );

        assert_eq!(None, tb.pop()?);

        Ok(())
    }

    #[test]
    fn test_fragment() -> Result<()> {
        let mut tb = TokenBuffer::from("foo}bar".as_bytes());

        let f = fragment(&mut tb)?;
        assert_eq!(
            vec![Char::Ascii(b'f'), Char::Ascii(b'o'), Char::Ascii(b'o')],
            f.0
        );

        let c = tb.pop()?.unwrap();
        assert_eq!(Char::Ascii(b'}'), c);

        let f = fragment(&mut tb)?;
        assert_eq!(
            vec![Char::Ascii(b'b'), Char::Ascii(b'a'), Char::Ascii(b'r')],
            f.0
        );

        assert_eq!(None, tb.pop()?);

        Ok(())
    }
}
