//! A browser's requests to a preview, for the tests: plain HTTP over `127.0.0.1`, with the `Host` a test chooses.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;

/// The page's port and the path under its token, from a preview's address `url`.
pub(crate) fn page_of(url: &str) -> (u16, String) {
    let rest = url.strip_prefix("http://127.0.0.1:").expect("on 127.0.0.1");
    let (port, path) = rest.split_once('/').expect("a path");
    (port.parse().expect("a port"), format!("/{path}"))
}

/// A connection to `port`, its answers waited for no longer than a load takes.
fn connected(port: u16) -> TcpStream {
    let stream = TcpStream::connect(("127.0.0.1", port)).expect("connected");
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .expect("a timeout");
    stream
}

/// What a request of `method` and `path` on `port`, with `host` as its `Host` (none when `None`), was answered: its
/// status, its head and its body.
pub(crate) fn ask(
    port: u16,
    method: &str,
    path: &str,
    host: Option<&str>,
) -> (u16, String, String) {
    let mut stream = connected(port);
    let host = host
        .map(|host| format!("Host: {host}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\n{host}Connection: close\r\n\r\n"
    )
    .expect("asked");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("an answer");
    // A tarball isn't text: what isn't is replaced, as the tests only read text bodies.
    let answer = String::from_utf8_lossy(&bytes);
    let (head, body) = answer.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .expect("a status");
    (status, head.to_owned(), body.to_owned())
}

/// A `GET` of `path` on `port`, as the browser that opened it asks.
pub(crate) fn get(port: u16, path: &str) -> (u16, String, String) {
    ask(port, "GET", path, Some(&format!("127.0.0.1:{port}")))
}

/// The events stream of the page at `token` on `port`, its head read.
pub(crate) fn listen(port: u16, token: &str) -> BufReader<TcpStream> {
    let mut stream = connected(port);
    write!(
        stream,
        "GET {token}events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"
    )
    .expect("asked");
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).expect("a status");
    assert_eq!(line, "HTTP/1.1 200 OK\r\n");
    while line != "\r\n" {
        line.clear();
        reader.read_line(&mut line).expect("a header");
    }
    reader
}

/// The next event on `events`: its name and its data.
pub(crate) fn next_event(events: &mut BufReader<TcpStream>) -> (String, Value) {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        events.read_line(&mut line).expect("an event's line");
        if line == "\n" {
            break;
        }
        lines.push(line.trim_end().to_owned());
    }
    let field = |name: &str| {
        lines
            .iter()
            .find_map(|line| line.strip_prefix(name))
            .unwrap_or_default()
            .to_owned()
    };
    (
        field("event: "),
        serde_json::from_str(&field("data: ")).expect("JSON"),
    )
}
