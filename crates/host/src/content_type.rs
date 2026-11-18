//! A served file's type, by its extension: what a bundle's files are.

use std::path::Path;

/// The type of the file at `path`, by its extension; `application/octet-stream` for any other.
#[must_use]
pub fn content_type(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_file_has_its_type() {
        for (name, kind) in [
            ("a.html", "text/html; charset=utf-8"),
            ("a.js", "text/javascript; charset=utf-8"),
            ("a.mjs", "text/javascript; charset=utf-8"),
            ("a.css", "text/css; charset=utf-8"),
            ("a.json", "application/json"),
            ("a.js.map", "application/json"),
            ("a.wasm", "application/wasm"),
            ("a.svg", "image/svg+xml"),
            ("a.png", "image/png"),
            ("a.woff2", "font/woff2"),
            ("a.woff", "font/woff"),
            ("a.ttf", "font/ttf"),
            ("a", "application/octet-stream"),
        ] {
            assert_eq!(content_type(name), kind, "{name}");
        }
    }
}
