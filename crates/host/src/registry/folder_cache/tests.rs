use std::sync::atomic::{AtomicU32, Ordering};

use semver::Version;

use super::*;

const NPM: &str = "https://registry.npmjs.org/";
const KIT: &str = "https://npm.kit.example/";

/// A folder of its own under the system's temporary folder, removed when dropped.
struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!("compmd-host-cache-{}-{n}", std::process::id())))
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
        drop(fs::remove_file(&self.0));
    }
}

fn pack(name: &str) -> PackName {
    name.parse().expect("a name")
}

fn version(text: &str) -> VersionInfo {
    VersionInfo {
        version: text.parse().expect("a version"),
        tarball: format!("{NPM}@kit/mermaid/-/mermaid-{text}.tgz"),
        integrity: Some("sha512-abc".into()),
        unpacked_size: Some(10),
    }
}

fn release(integrity: &str) -> KeptRelease {
    KeptRelease {
        pack: pack("@kit/mermaid"),
        version: Version::new(1, 2, 0),
        integrity: integrity.into(),
    }
}

#[test]
fn versions_are_kept_by_registry_fresh_for_their_max_age() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let mermaid = pack("@kit/mermaid");
    assert_eq!(cache.kept_versions(NPM, &mermaid), None);
    cache
        .keep_versions(NPM, &mermaid, vec![version("1.2.0")], 300)
        .expect("kept");
    cache
        .keep_versions(KIT, &mermaid, vec![version("2.0.0")], 0)
        .expect("kept");
    assert_eq!(
        cache.kept_versions(NPM, &mermaid),
        Some(KeptVersions {
            versions: vec![version("1.2.0")],
            fresh: true
        })
    );
    assert_eq!(
        cache.kept_versions(KIT, &mermaid),
        Some(KeptVersions {
            versions: vec![version("2.0.0")],
            fresh: false
        }),
        "kept, but stale at once"
    );
    assert_eq!(cache.kept_versions(NPM, &pack("@kit/charts")), None);
}

#[test]
fn a_scopes_packs_are_those_whose_versions_at_the_registry_are_kept() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    assert_eq!(
        cache.packs_in(NPM, "@kit"),
        Vec::<PackName>::new(),
        "nothing kept"
    );
    for (registry, name) in [
        (NPM, "@kit/timeline"),
        (NPM, "@kit/cards"),
        (KIT, "@kit/maps"),
        (NPM, "@acme/maps"),
    ] {
        cache
            .keep_versions(registry, &pack(name), vec![version("1.0.0")], 0)
            .expect("kept");
    }
    fs::write(temp.0.join("versions/@kit/notes.txt"), "").expect("written");
    assert_eq!(
        cache.packs_in(NPM, "@kit"),
        [pack("@kit/cards"), pack("@kit/timeline")]
    );
    assert_eq!(cache.packs_in(KIT, "@kit"), [pack("@kit/maps")]);
}

#[test]
fn a_release_is_given_back_only_with_the_integrity_it_was_kept_with() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let files = Files {
        text: BTreeMap::from([("index.comp.md".to_owned(), "# Mermaid".to_owned())]),
        binary: BTreeMap::from([("logo.png".to_owned(), vec![0x89, 0xff])]),
    };
    assert_eq!(cache.kept(&release("sha512-abc")), None);
    cache
        .keep(&release("sha512-abc"), files.clone())
        .expect("kept");
    assert_eq!(cache.kept(&release("sha512-abc")), Some(files));
    assert_eq!(
        cache.kept(&release("sha512-other")),
        None,
        "another tarball's integrity"
    );
}

#[test]
fn what_a_store_cant_keep_is_an_error() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let unsafe_path = Files {
        text: BTreeMap::from([("../outside.js".to_owned(), String::new())]),
        binary: BTreeMap::new(),
    };
    assert!(
        cache
            .keep(&release("not an integrity"), Files::default())
            .is_err()
    );
    assert!(cache.keep(&release("sha512-abc"), unsafe_path).is_err());
    fs::write(&temp.0, "a file, not a folder").expect("written");
    assert!(
        cache
            .keep_versions(NPM, &pack("@kit/mermaid"), Vec::new(), 0)
            .is_err()
    );
}
