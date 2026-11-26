//! A host's `registry/` proxy in Rust ([resolving registry packs], decision 9; [the Tauri plugin], decision 11), as
//! `@compmd/registry`'s `registryProxy` is in JavaScript: it passes a pack's abbreviated metadata and its tarballs on
//! from the host's registries, so a registry never learns who reads which document. It answers nothing else, passes
//! on no credentials and keeps nothing; the page checks what it passes on, as it checks a registry's own answers.
//!
//! [resolving registry packs]: https://github.com/comp-markdown/compmd/blob/main/docs/resolving-registry-packs.md
//! [the Tauri plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/tauri-plugin.md

use compmd_pack::PackName;
use http::{Response, StatusCode, header};
use serde_json::Value;

use super::client::Client;
use super::metadata::{ABBREVIATED, answered_versions};
use super::options::Options;
use super::step::Answer;

/// Where a version's tarball is under the proxy's base.
const TARBALL: &str = "-/tarball/";

/// The proxy of a host's registries.
#[derive(Clone, Debug)]
pub struct Proxy {
    options: Options,
    client: Client,
}

/// An answer of `status` with `text`.
fn text(status: StatusCode, text: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(text.as_bytes().to_vec())
        .unwrap_or_default()
}

/// `path` with its `%XX` escapes decoded; `None` when it doesn't decode to UTF-8.
fn decoded(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let escape = (bytes[at] == b'%')
            .then(|| path.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        if let Some(byte) = escape {
            out.push(byte);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// What a request under the proxy's base asks for.
enum Asked {
    Metadata(PackName),
    Tarball(PackName, String),
}

/// What `path`, under the proxy's base, asks for: `<name>`, or `-/tarball/<name>/<version>`.
fn asked(path: &str) -> Option<Asked> {
    let path = decoded(path)?;
    let Some(tarball) = path.strip_prefix(TARBALL) else {
        return path.parse().ok().map(Asked::Metadata);
    };
    let (name, version) = tarball.rsplit_once('/')?;
    Some(Asked::Tarball(name.parse().ok()?, version.to_owned()))
}

impl Proxy {
    /// The proxy of `options`' registries, its tarballs no larger than its size cap, fetched with `client`.
    #[must_use]
    pub fn new(options: Options, client: Client) -> Self {
        Self { options, client }
    }

    /// The answer to a `GET` of `path`, what follows the proxy's base, which is `base` in full (ending in `/`): a
    /// pack's metadata, its tarballs' URLs rewritten to the proxy's own, or a tarball's bytes. A pack no registry is
    /// configured for, or one the registry hasn't, is `404`; a registry that fails, `502`.
    #[must_use]
    pub fn answer(&self, path: &str, base: &str) -> Response<Vec<u8>> {
        let not_found = || text(StatusCode::NOT_FOUND, "Not found");
        let Some(asked) = asked(path) else {
            return not_found();
        };
        let pack = match &asked {
            Asked::Metadata(pack) | Asked::Tarball(pack, _) => pack,
        };
        let Some(registry) = self.options.registry_of(pack) else {
            return not_found();
        };
        let answered = match &asked {
            Asked::Metadata(pack) => self.metadata(registry, pack, base),
            Asked::Tarball(pack, version) => self.tarball(registry, pack, version),
        };
        answered.unwrap_or_else(|| text(StatusCode::BAD_GATEWAY, "Bad gateway"))
    }

    /// `pack`'s metadata from `registry`, each tarball's URL the proxy's own; `None` when the registry fails.
    fn metadata(&self, registry: &str, pack: &PackName, base: &str) -> Option<Response<Vec<u8>>> {
        let metadata = self.client.metadata(registry, pack).ok()?;
        if metadata.status == 404 {
            return Some(text(StatusCode::NOT_FOUND, "Not found"));
        }
        if !(200..300).contains(&metadata.status) {
            return None;
        }
        let mut packument: Value = serde_json::from_str(&metadata.body).ok()?;
        if let Some(versions) = packument["versions"].as_object_mut() {
            for (version, entry) in versions {
                if let Some(dist) = entry.get_mut("dist").and_then(Value::as_object_mut) {
                    let tarball = format!("{base}{TARBALL}{pack}/{version}");
                    dist.insert("tarball".to_owned(), Value::String(tarball));
                }
            }
        }
        Response::builder()
            .header(header::CONTENT_TYPE, ABBREVIATED)
            .header(
                header::CACHE_CONTROL,
                format!("public, max-age={}", metadata.max_age),
            )
            .body(packument.to_string().into_bytes())
            .ok()
    }

    /// The tarball of `pack` at `version`, from where `registry`'s metadata says it is; `None` when it fails.
    fn tarball(&self, registry: &str, pack: &PackName, version: &str) -> Option<Response<Vec<u8>>> {
        let metadata = self.client.metadata(registry, pack).ok()?;
        let versions = answered_versions(registry, pack, metadata.status, &metadata.body).ok()?;
        let Some(info) = versions
            .iter()
            .find(|info| info.version.to_string() == version)
        else {
            return Some(text(StatusCode::NOT_FOUND, "Not found"));
        };
        let Answer::Tarball { bytes } =
            self.client
                .tarball(registry, info, self.options.max_pack_size)
        else {
            return None;
        };
        // A version's tarball never changes.
        Response::builder()
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(bytes)
            .ok()
    }
}

#[cfg(test)]
mod tests;
