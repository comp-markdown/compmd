use std::fs;

use semver::Version;

use super::*;

pub const HASH: &str = "1df336e75685de8c";

/// A manifest naming `HASH`, holding `@kit/mermaid` 1.2.0.
pub fn manifest() -> Vec<u8> {
    format!(
        r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}, "source": "compmd://localhost/",
            "packs": [{{"name": "@kit/mermaid", "version": "1.2.0", "integrity": "sha512-abc"}}]}}"#
    )
    .into_bytes()
}

/// A bundle with `strict.html`, `chunks/runtime.js`, the Mermaid pack's `package.json` and module, and a file
/// outside both.
pub fn bundle() -> Bundle {
    Bundle::from_files([
        (MANIFEST.to_owned(), manifest()),
        (
            format!("frames/{HASH}/strict.html"),
            b"<!doctype html>".to_vec(),
        ),
        (
            format!("frames/{HASH}/chunks/runtime.js"),
            b"export {}".to_vec(),
        ),
        (
            "packs/@kit/mermaid@1.2.0/package.json".to_owned(),
            br#"{"name": "@kit/mermaid"}"#.to_vec(),
        ),
        (
            "packs/@kit/mermaid@1.2.0/dist/mermaid.js".to_owned(),
            b"export default 1".to_vec(),
        ),
        (
            "packs/@kit/other@1.0.0/index.js".to_owned(),
            b"not in the manifest".to_vec(),
        ),
        ("secret.txt".to_owned(), b"not served".to_vec()),
    ])
    .expect("a bundle")
}

fn mermaid(path: &str) -> Place {
    Place {
        pack: "@kit/mermaid".to_owned(),
        version: Version::new(1, 2, 0),
        path: path.to_owned(),
    }
}

#[test]
fn a_bundle_gives_its_frames_and_its_packs_files() {
    let bundle = bundle();
    assert_eq!(bundle.manifest().frames.hash, HASH);
    assert_eq!(
        bundle.frame_file("strict.html"),
        Some(b"<!doctype html>".to_vec())
    );
    assert_eq!(
        bundle.frame_file("chunks/runtime.js"),
        Some(b"export {}".to_vec())
    );
    assert_eq!(bundle.frame_file("missing.js"), None);
    assert_eq!(
        bundle.pack_file(&mermaid("dist/mermaid.js")),
        Some(b"export default 1".to_vec())
    );
    assert_eq!(bundle.pack_file(&mermaid("missing.js")), None);
    let other = Place {
        pack: "@kit/other".to_owned(),
        version: Version::new(1, 0, 0),
        path: "index.js".to_owned(),
    };
    assert_eq!(
        bundle.pack_file(&other),
        None,
        "a pack the manifest doesn't name"
    );
}

#[test]
fn the_frames_base_is_on_the_origin_it_is_given() {
    let bundle = bundle();
    let base = format!("compmd://localhost/frames/{HASH}/");
    assert_eq!(bundle.frames_base("compmd://localhost"), base);
    assert_eq!(bundle.frames_base("compmd://localhost/"), base);
    assert_eq!(
        bundle.frames_base("http://compmd.localhost"),
        format!("http://compmd.localhost/frames/{HASH}/")
    );
}

#[test]
fn an_embedded_bundle_and_one_in_a_folder() {
    static FILES: &[(&str, &[u8])] = &[
        ("frames/1df336e75685de8c/strict.html", b"<!doctype html>"),
        (
            "manifest.json",
            br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#,
        ),
    ];
    let embedded = Bundle::embedded(FILES).expect("a bundle");
    assert_eq!(
        embedded.frame_file("strict.html"),
        Some(b"<!doctype html>".to_vec())
    );

    let folder = std::env::temp_dir().join(format!("compmd-host-bundle-{}", std::process::id()));
    fs::create_dir_all(folder.join("frames").join(HASH)).expect("made");
    fs::write(folder.join(MANIFEST), manifest()).expect("written");
    fs::write(
        folder.join("frames").join(HASH).join("strict.html"),
        "<!doctype html>",
    )
    .expect("written");
    let read = Bundle::read(&folder).expect("a bundle");
    assert_eq!(
        read.frame_file("strict.html"),
        Some(b"<!doctype html>".to_vec())
    );
    drop(fs::remove_dir_all(folder));
}

#[test]
fn a_bundle_without_a_manifest_or_with_a_bad_one_says_so() {
    let none: [(String, Vec<u8>); 0] = [];
    assert_eq!(Bundle::from_files(none), Err(BundleError::NoManifest));
    assert!(matches!(
        Bundle::from_files([(MANIFEST, b"{}".to_vec())]),
        Err(BundleError::Manifest(_))
    ));
    assert!(BundleError::NoManifest.to_string().contains("@compmd/vite"));
    let gone = std::env::temp_dir().join("compmd-host-no-such-bundle");
    assert_eq!(Bundle::read(&gone), Err(BundleError::NoManifest));
}
