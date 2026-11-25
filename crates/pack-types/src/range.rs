//! A version or range after a pack's name in an import's `from` (spec §6.1), as the processor and every host read it.

use semver::{Version, VersionReq};

/// The range `text` is, and whether it is one exact version: `1.4.2` is exactly that release, unlike Cargo's
/// requirements, where it would be `^1.4.2`. `None` when it isn't a range.
///
/// ```
/// use compmd_pack::range;
///
/// assert_eq!(range("1.4.2").map(|(_, exact)| exact), Some(true));
/// assert_eq!(range("^1.4").map(|(_, exact)| exact), Some(false));
/// assert_eq!(range("latest!"), None);
/// ```
#[must_use]
pub fn range(text: &str) -> Option<(VersionReq, bool)> {
    if let Ok(version) = Version::parse(text) {
        let exact = VersionReq::parse(&format!("={version}")).ok()?;
        return Some((exact, true));
    }
    VersionReq::parse(text).ok().map(|range| (range, false))
}

/// The newest of `versions` that `range` takes, as the processor reads the version or range after a pack's name in
/// an import's `from` (spec §6.1): `1.4.2` is exactly that version, `1` and `^1.4` ranges, `*` any. `None` when none
/// matches, or `range` isn't one. A host that fetches the packs a document needs picks a version with this, so the
/// processor takes the release it was given.
///
/// ```
/// use compmd_pack::newest_matching;
/// use semver::Version;
///
/// let versions = ["1.4.2", "1.5.0", "2.0.0"].map(|v| Version::parse(v).unwrap());
/// assert_eq!(newest_matching("1", &versions), Some(&versions[1]));
/// assert_eq!(newest_matching("1.4.2", &versions), Some(&versions[0]));
/// assert_eq!(newest_matching("^3", &versions), None);
/// ```
#[must_use]
pub fn newest_matching<'v>(range_text: &str, versions: &'v [Version]) -> Option<&'v Version> {
    let (range, _) = range(range_text)?;
    versions
        .iter()
        .filter(|version| range.matches(version))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_exact_version_is_only_itself_and_a_range_its_matches() {
        let versions: Vec<Version> = ["1.4.2", "1.4.3", "1.5.0"]
            .iter()
            .map(|v| Version::parse(v).expect("a version"))
            .collect();
        assert_eq!(newest_matching("1.4.2", &versions), Some(&versions[0]));
        assert_eq!(newest_matching("~1.4", &versions), Some(&versions[1]));
        assert_eq!(newest_matching("*", &versions), Some(&versions[2]));
        assert_eq!(newest_matching("not a range", &versions), None);
        assert_eq!(newest_matching("2", &versions), None);
    }
}
