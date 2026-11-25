//! The binary, as an editor starts it: LSP over its standard input and output, framed by `Content-Length`.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

/// `message` framed as LSP frames it.
fn framed(message: &Value) -> Vec<u8> {
    let body = message.to_string();
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

/// The next message on `reader`.
fn read(reader: &mut impl BufRead) -> Value {
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("a header");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            length = value.parse().expect("a length");
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).expect("a body");
    serde_json::from_slice(&body).expect("JSON")
}

#[test]
fn the_binary_speaks_lsp_over_stdio_and_exits_when_asked() {
    let mut server = Command::new(env!("CARGO_BIN_EXE_compmd-lsp"))
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the server");
    let mut input = server.stdin.take().expect("its input");
    let mut output = BufReader::new(server.stdout.take().expect("its output"));
    input.write_all(&framed(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }))).expect("sent");
    let answer = read(&mut output);
    assert_eq!(answer["result"]["serverInfo"]["name"], "compmd-lsp");
    assert_eq!(
        answer["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    input
        .write_all(&framed(
            &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
        ))
        .expect("sent");
    input
        .write_all(&framed(
            &json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown" }),
        ))
        .expect("sent");
    assert_eq!(read(&mut output)["id"], 2);
    input
        .write_all(&framed(&json!({ "jsonrpc": "2.0", "method": "exit" })))
        .expect("sent");
    drop(input);
    assert!(server.wait().expect("its exit").success());
}

#[test]
fn the_binary_says_its_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_compmd-lsp"))
        .arg("--version")
        .output()
        .expect("the server");
    assert!(String::from_utf8_lossy(&output.stdout).contains(env!("CARGO_PKG_VERSION")));
}
