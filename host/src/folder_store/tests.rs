use std::sync::atomic::{AtomicU32, Ordering};

use super::*;

/// A folder of its own under the system's temporary folder, removed when dropped.
struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!("compmd-host-store-{}-{n}", std::process::id())))
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

#[test]
fn a_kept_release_is_read_back() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    let version = Version::new(1, 2, 0);
    store
        .keep(
            "@kit/mermaid",
            &version,
            "sha512-abc",
            [
                ("package.json", b"{}".to_vec()),
                ("dist/mermaid.js", b"export {}".to_vec()),
            ],
        )
        .expect("kept");
    assert_eq!(
        store.releases("@kit/mermaid"),
        [StoredRelease {
            version: version.clone(),
            integrity: "sha512-abc".to_owned()
        }]
    );
    assert_eq!(
        store.file("@kit/mermaid", &version, "dist/mermaid.js"),
        Some(b"export {}".to_vec())
    );
    assert_eq!(store.file("@kit/mermaid", &version, "dist"), None);
    assert_eq!(store.file("@kit/mermaid", &version, "missing.js"), None);
    assert_eq!(
        store.file("@kit/mermaid", &Version::new(1, 3, 0), "package.json"),
        None
    );
}

#[test]
fn keeping_again_replaces_the_release() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    let version = Version::new(1, 0, 0);
    store
        .keep(
            "@kit/x",
            &version,
            "sha512-one",
            [("old.js", b"1".to_vec())],
        )
        .expect("kept");
    store
        .keep(
            "@kit/x",
            &version,
            "sha512-two",
            [("new.js", b"2".to_vec())],
        )
        .expect("kept");
    assert_eq!(store.file("@kit/x", &version, "old.js"), None);
    assert_eq!(
        store.file("@kit/x", &version, "new.js"),
        Some(b"2".to_vec())
    );
    assert_eq!(store.releases("@kit/x")[0].integrity, "sha512-two");
}

#[test]
fn only_its_own_releases_are_listed() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    store
        .keep(
            "@kit/x",
            &Version::new(1, 0, 0),
            "sha512-a",
            [("a.js", b"1".to_vec())],
        )
        .expect("kept");
    store
        .keep(
            "@kit/xy",
            &Version::new(2, 0, 0),
            "sha512-b",
            [("a.js", b"1".to_vec())],
        )
        .expect("kept");
    fs::create_dir_all(temp.0.join("@kit").join("x@not-a-version")).expect("made");
    fs::create_dir_all(temp.0.join("@kit").join("x@3.0.0")).expect("made, without release.json");
    let versions: Vec<Version> = store
        .releases("@kit/x")
        .into_iter()
        .map(|release| release.version)
        .collect();
    assert_eq!(versions, [Version::new(1, 0, 0)]);
    assert_eq!(store.releases("@kit/none"), []);
    assert_eq!(store.releases("@other/x"), []);
}

#[test]
fn nothing_outside_a_release_is_kept_or_read() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    let version = Version::new(1, 0, 0);
    for path in [
        "../escape.js",
        "a/../../escape.js",
        "./a.js",
        "a//b.js",
        "",
        "/abs.js",
    ] {
        let error = store
            .keep("@kit/x", &version, "sha512-a", [(path, b"1".to_vec())])
            .expect_err(path);
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{path}");
    }
    let error = store
        .keep("../x", &version, "sha512-a", [("a.js", b"1".to_vec())])
        .expect_err("a bad name");
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    store
        .keep("@kit/x", &version, "sha512-a", [("a.js", b"1".to_vec())])
        .expect("kept");
    assert_eq!(store.file("@kit/x", &version, "../release.json"), None);
    assert_eq!(store.file("../x", &version, "a.js"), None);
    assert_eq!(store.releases("../x"), []);
}
