use std::sync::atomic::{AtomicU32, Ordering};

use semver::Version;

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
        drop(fs::remove_file(&self.0));
    }
}

fn release(text: &str) -> ReleaseId {
    text.parse().expect("a release")
}

fn integrity(text: &str) -> Integrity {
    text.parse().expect("an integrity")
}

fn place(text: &str) -> Place {
    text.parse().expect("a place")
}

fn files(paths: &[(&str, &[u8])]) -> Vec<(FilePath, Vec<u8>)> {
    paths
        .iter()
        .map(|(path, bytes)| (path.parse().expect("a path"), bytes.to_vec()))
        .collect()
}

#[test]
fn a_kept_release_is_read_back() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    let mermaid = release("@kit/mermaid@1.2.0");
    store
        .keep(
            &mermaid,
            &integrity("sha512-abc"),
            files(&[("package.json", b"{}"), ("dist/mermaid.js", b"export {}")]),
        )
        .expect("kept");
    assert_eq!(
        store.releases(&mermaid.pack),
        [StoredRelease {
            release: mermaid.clone(),
            integrity: integrity("sha512-abc")
        }]
    );
    assert_eq!(
        store.file(&place("@kit/mermaid@1.2.0/dist/mermaid.js")),
        Some(b"export {}".to_vec())
    );
    assert_eq!(store.file(&place("@kit/mermaid@1.2.0/dist")), None);
    assert_eq!(store.file(&place("@kit/mermaid@1.2.0/missing.js")), None);
    assert_eq!(store.file(&place("@kit/mermaid@1.3.0/package.json")), None);
}

#[test]
fn keeping_again_replaces_the_release() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    let x = release("@kit/x@1.0.0");
    store
        .keep(&x, &integrity("sha512-one"), files(&[("old.js", b"1")]))
        .expect("kept");
    store
        .keep(&x, &integrity("sha512-two"), files(&[("new.js", b"2")]))
        .expect("kept");
    assert_eq!(store.file(&place("@kit/x@1.0.0/old.js")), None);
    assert_eq!(
        store.file(&place("@kit/x@1.0.0/new.js")),
        Some(b"2".to_vec())
    );
    assert_eq!(
        store.releases(&x.pack)[0].integrity,
        integrity("sha512-two")
    );
}

#[test]
fn only_its_own_releases_are_listed() {
    let temp = Temp::new();
    let store = FolderStore::new(&temp.0);
    store
        .keep(
            &release("@kit/x@1.0.0"),
            &integrity("sha512-a"),
            files(&[("a.js", b"1")]),
        )
        .expect("kept");
    store
        .keep(
            &release("@kit/xy@2.0.0"),
            &integrity("sha512-b"),
            files(&[("a.js", b"1")]),
        )
        .expect("kept");
    fs::create_dir_all(temp.0.join("@kit").join("x@not-a-version")).expect("made");
    fs::create_dir_all(temp.0.join("@kit").join("x@3.0.0")).expect("made, without release.json");
    let x: PackName = "@kit/x".parse().expect("a name");
    let versions: Vec<Version> = store
        .releases(&x)
        .into_iter()
        .map(|kept| kept.release.version)
        .collect();
    assert_eq!(versions, [Version::new(1, 0, 0)]);
    assert_eq!(store.releases(&"@kit/none".parse().expect("a name")), []);
    assert_eq!(store.releases(&"@other/x".parse().expect("a name")), []);
}

#[test]
fn a_store_that_cant_be_written_says_so() {
    let temp = Temp::new();
    fs::write(&temp.0, "a file, not a folder").expect("written");
    let store = FolderStore::new(temp.0.join("store"));
    let error = store
        .keep(
            &release("@kit/x@1.0.0"),
            &integrity("sha512-a"),
            files(&[("a.js", b"1")]),
        )
        .expect_err("a file is in the way");
    assert_ne!(error.kind(), io::ErrorKind::InvalidInput);
}
