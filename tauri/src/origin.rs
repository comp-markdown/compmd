//! Where the bundle is served, and how the page learns it (docs/tauri-plugin.md, decision 5).

/// The origin of the scheme `scheme`: Tauri serves custom schemes as `<scheme>://localhost`, and over http as
/// `http://<scheme>.localhost` on Windows and Android.
pub fn origin_of(scheme: &str) -> String {
    origin_on(scheme, cfg!(any(windows, target_os = "android")))
}

/// The origin of the scheme `scheme`, served over http or not.
pub fn origin_on(scheme: &str, over_http: bool) -> String {
    if over_http {
        format!("http://{scheme}.localhost")
    } else {
        format!("{scheme}://localhost")
    }
}

/// The script that gives the app's pages the origin, as `createSandbox` reads it. Tauri runs it in the main frame only,
/// before the page's own scripts.
pub fn init_script(origin: &str) -> String {
    let origin = serde_json::Value::String(origin.to_owned());
    format!("globalThis.__COMPMD__ = Object.freeze({{ origin: {origin} }});")
}

/// Whether `name` is a scheme's name (RFC 3986): a letter, then letters, digits, `+`, `-` or `.`; lowercase, as
/// webviews compare them.
pub fn is_scheme(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.')
        })
}
