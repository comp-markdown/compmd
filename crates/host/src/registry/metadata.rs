//! npm's protocol, as far as the rules read it ([resolving registry packs], decision 7): a pack's versions from its
//! abbreviated metadata, and where a version's tarball is, on its registry's origin only.
//!
//! [resolving registry packs]: https://github.com/comp-markdown/compmd/blob/main/docs/resolving-registry-packs.md

use compmd_pack::PackName;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// The abbreviated form of npm's metadata, which lists every version with its tarball and integrity.
pub const ABBREVIATED: &str = "application/vnd.npm.install-v1+json";

/// A version of a pack, as a registry's metadata lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    /// The version.
    pub version: Version,
    /// Where its tarball is, in full.
    pub tarball: String,
    /// The integrity the registry publishes for the tarball, `sha512-…`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    /// Its unpacked size in bytes, which the host's size cap is checked against before anything is downloaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unpacked_size: Option<u64>,
}

/// The URL of `pack`'s metadata at the registry `registry` (a URL ending in `/`): its name with its `/` escaped.
pub(crate) fn metadata_url(registry: &str, pack: &PackName) -> String {
    format!("{registry}{}", pack.as_str().replacen('/', "%2f", 1))
}

/// The versions `body`, a registry's abbreviated metadata, lists, each with a tarball, its URL resolved against the
/// registry's.
///
/// # Errors
///
/// When `body` isn't JSON.
pub(crate) fn versions_of(body: &str, registry: &str) -> Result<Vec<VersionInfo>, String> {
    let metadata: Value = serde_json::from_str(body)
        .map_err(|error| format!("{registry} gave metadata that isn't JSON: {error}"))?;
    let base = Url::parse(registry).ok();
    let versions = metadata["versions"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(version, entry)| {
            let dist = &entry["dist"];
            let tarball = dist["tarball"].as_str()?;
            let tarball = base
                .as_ref()
                .and_then(|base| base.join(tarball).ok())
                .map(String::from)?;
            Some(VersionInfo {
                version: Version::parse(version).ok()?,
                tarball,
                integrity: dist["integrity"].as_str().map(str::to_owned),
                unpacked_size: dist["unpackedSize"].as_u64(),
            })
        });
    Ok(versions.collect())
}

/// Why `tarball` may not be fetched for the registry `registry`, if so: a tarball is fetched only from its
/// registry's origin, so a host's headers for it (a private registry's token) go nowhere else.
pub(crate) fn off_origin(tarball: &str, registry: &str) -> Option<String> {
    let origin = |url: &str| {
        Url::parse(url)
            .ok()
            .map(|url| url.origin().ascii_serialization())
    };
    let registry_origin = origin(registry).unwrap_or_default();
    (origin(tarball).as_deref() != Some(registry_origin.as_str()))
        .then(|| format!("{tarball} isn't on its registry's origin, {registry_origin}: a tarball is fetched only from there"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NPM: &str = "https://registry.npmjs.org/";

    #[test]
    fn versions_have_their_tarball_integrity_and_size() {
        let body = r#"{"name": "@kit/mermaid", "versions": {
            "1.0.0": {"dist": {"tarball": "https://registry.npmjs.org/@kit/mermaid/-/mermaid-1.0.0.tgz", "integrity": "sha512-a", "unpackedSize": 1200}},
            "1.1.0": {"dist": {"tarball": "/@kit/mermaid/-/mermaid-1.1.0.tgz"}},
            "1.2.0": {"dist": {}},
            "not semver": {"dist": {"tarball": "x.tgz"}}
        }}"#;
        let versions = versions_of(body, NPM).expect("versions");
        assert_eq!(
            versions,
            [
                VersionInfo {
                    version: Version::new(1, 0, 0),
                    tarball: "https://registry.npmjs.org/@kit/mermaid/-/mermaid-1.0.0.tgz".into(),
                    integrity: Some("sha512-a".into()),
                    unpacked_size: Some(1200),
                },
                VersionInfo {
                    version: Version::new(1, 1, 0),
                    tarball: "https://registry.npmjs.org/@kit/mermaid/-/mermaid-1.1.0.tgz".into(),
                    integrity: None,
                    unpacked_size: None,
                },
            ]
        );
        assert_eq!(versions_of("{}", NPM), Ok(Vec::new()));
        assert!(versions_of("{", NPM).is_err_and(|error| {
            error.starts_with("https://registry.npmjs.org/ gave metadata that isn't JSON")
        }));
        assert_eq!(
            versions_of(
                r#"{"versions": {"1.0.0": {"dist": {"tarball": "x.tgz"}}}}"#,
                "not a url"
            ),
            Ok(Vec::new())
        );
    }

    #[test]
    fn a_packs_metadata_is_at_its_escaped_name() {
        let pack: PackName = "@kit/mermaid".parse().expect("a name");
        assert_eq!(
            metadata_url(NPM, &pack),
            "https://registry.npmjs.org/@kit%2fmermaid"
        );
    }

    #[test]
    fn a_tarball_is_fetched_only_from_its_registrys_origin() {
        assert_eq!(
            off_origin("https://registry.npmjs.org/@kit/x/-/x-1.0.0.tgz", NPM),
            None
        );
        assert_eq!(
            off_origin("https://evil.example/x.tgz", NPM),
            Some("https://evil.example/x.tgz isn't on its registry's origin, https://registry.npmjs.org: a tarball is fetched only from there".into())
        );
        assert!(off_origin("not a url", NPM).is_some());
    }
}
