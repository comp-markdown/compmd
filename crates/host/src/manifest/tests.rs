use super::*;

const HASH: &str = "1df336e75685de8c";

fn release(text: &str) -> ReleaseId {
    text.parse().expect("a release")
}

#[test]
fn a_manifest_with_everything() {
    let text = format!(
        r#"{{
          "format": 1,
          "frames": {{"hash": "{HASH}"}},
          "source": "compmd://localhost/",
          "packs": [{{
            "pack": "@kit/mermaid",
            "version": "1.2.0",
            "integrity": "sha512-abc",
            "trust": "permissive",
            "modules": {{"dist/mermaid.js": "sha384-def"}}
          }}],
          "libraries": [{{"name": "d3", "version": "7.9.0"}}],
          "policy": {{"imports": "any"}},
          "ambient": ["@app/components"]
        }}"#
    );
    let manifest = Manifest::parse(&text).expect("a manifest");
    assert_eq!(manifest.frames.hash.as_str(), HASH);
    assert_eq!(manifest.scheme(), Some("compmd"));
    let pack = manifest
        .pack(&release("@kit/mermaid@1.2.0"))
        .expect("the pack");
    assert_eq!(
        pack.integrity,
        Some("sha512-abc".parse().expect("an integrity"))
    );
    assert_eq!(pack.trust, Trust::Permissive);
    let module: FilePath = "dist/mermaid.js".parse().expect("a path");
    assert_eq!(pack.modules[&module].to_string(), "sha384-def");
    assert_eq!(manifest.pack(&release("@kit/mermaid@1.3.0")), None);
    assert_eq!(manifest.libraries[0].version, Version::new(7, 9, 0));
    assert_eq!(
        manifest.ambient,
        ["@app/components".parse::<PackName>().expect("a name")]
    );
    assert!(manifest.policy.is_some());
    let written = serde_json::to_string(&manifest).expect("written");
    assert_eq!(Manifest::parse(&written), Ok(manifest));
}

#[test]
fn the_least_a_manifest_says() {
    let manifest = Manifest::parse(&format!(
        r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}}}"#
    ))
    .expect("a manifest");
    assert_eq!(manifest.packs, []);
    assert_eq!(manifest.scheme(), None);
    let pack: BundledPack =
        serde_json::from_str(r#"{"pack": "@kit/x", "version": "1.0.0"}"#).expect("a pack");
    assert_eq!(pack.trust, Trust::Strict);
}

#[test]
fn an_http_source_names_no_scheme() {
    for source in [
        "https://frames.example/",
        "http://compmd.localhost/",
        "no scheme",
    ] {
        let manifest = Manifest {
            source: Some(source.to_owned()),
            ..Manifest::parse(&format!(
                r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}}}"#
            ))
            .expect("a manifest")
        };
        assert_eq!(manifest.scheme(), None, "{source}");
    }
}

#[test]
fn a_manifest_that_cant_be_used_says_why() {
    let read = |text: String| Manifest::parse(&text);
    assert!(matches!(
        read("not json".to_owned()),
        Err(ManifestError::Json(_))
    ));
    assert_eq!(
        read(r#"{"format": 2, "frames": {"hash": "x"}}"#.to_owned()),
        Err(ManifestError::UnknownFormat {
            found: 2,
            known: FORMAT
        })
    );
    assert!(matches!(
        read(r#"{"format": 1}"#.to_owned()),
        Err(ManifestError::Json(_))
    ));
    for (what, packs) in [
        ("a hash", r#""frames": {"hash": "../secret"}"#.to_owned()),
        (
            "a name",
            format!(
                r#""frames": {{"hash": "{HASH}"}}, "packs": [{{"pack": "../x", "version": "1.0.0"}}]"#
            ),
        ),
        (
            "a path",
            format!(
                r#""frames": {{"hash": "{HASH}"}}, "packs": [{{"pack": "@kit/x", "version": "1.0.0", "modules": {{"../a.js": "sha384-a"}}}}]"#
            ),
        ),
        (
            "an integrity",
            format!(
                r#""frames": {{"hash": "{HASH}"}}, "packs": [{{"pack": "@kit/x", "version": "1.0.0", "integrity": "md5-a"}}]"#
            ),
        ),
    ] {
        let manifest = read(format!(r#"{{"format": 1, {packs}}}"#));
        assert!(matches!(manifest, Err(ManifestError::Json(_))), "{what}");
    }
}

#[test]
fn every_error_says_what_to_do() {
    let unknown = ManifestError::UnknownFormat { found: 2, known: 1 }.to_string();
    assert!(unknown.contains("upgrade compmd-host"), "{unknown}");
    assert!(
        ManifestError::Json("x".to_owned())
            .to_string()
            .contains("can't be read")
    );
    assert!(
        ManifestError::Hash("x".to_owned())
            .to_string()
            .contains("16 lowercase hex")
    );
}
