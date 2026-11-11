use super::*;

const HASH: &str = "1df336e75685de8c";

#[test]
fn a_manifest_with_everything() {
    let text = format!(
        r#"{{
          "format": 1,
          "frames": {{"hash": "{HASH}"}},
          "source": "compmd://localhost/",
          "packs": [{{
            "name": "@kit/mermaid",
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
    assert_eq!(manifest.frames.hash, HASH);
    assert_eq!(manifest.scheme(), Some("compmd"));
    let pack = manifest
        .pack("@kit/mermaid", &Version::new(1, 2, 0))
        .expect("the pack");
    assert_eq!(pack.integrity.as_deref(), Some("sha512-abc"));
    assert_eq!(pack.trust, Trust::Permissive);
    assert_eq!(pack.modules["dist/mermaid.js"], "sha384-def");
    assert_eq!(manifest.pack("@kit/mermaid", &Version::new(1, 3, 0)), None);
    assert_eq!(manifest.libraries[0].name, "d3");
    assert_eq!(manifest.ambient, ["@app/components"]);
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
        serde_json::from_str(r#"{"name": "@kit/x", "version": "1.0.0"}"#).expect("a pack");
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
    assert!(matches!(
        Manifest::parse("not json"),
        Err(ManifestError::Json(_))
    ));
    assert_eq!(
        Manifest::parse(r#"{"format": 2, "frames": {"hash": "x"}}"#),
        Err(ManifestError::UnknownFormat {
            found: 2,
            known: FORMAT
        })
    );
    assert!(matches!(
        Manifest::parse(r#"{"format": 1}"#),
        Err(ManifestError::Json(_))
    ));
    assert_eq!(
        Manifest::parse(r#"{"format": 1, "frames": {"hash": "../secret"}}"#),
        Err(ManifestError::Hash("../secret".to_owned()))
    );
    for hash in ["1DF336E75685DE8C", "1df336e7", "1df336e75685de8g"] {
        assert!(matches!(
            Manifest::parse(&format!(
                r#"{{"format": 1, "frames": {{"hash": "{hash}"}}}}"#
            )),
            Err(ManifestError::Hash(_))
        ));
    }
    assert_eq!(
        Manifest::parse(&format!(
            r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}, "packs": [{{"name": "../x", "version": "1.0.0"}}]}}"#
        )),
        Err(ManifestError::PackName("../x".to_owned()))
    );
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
    assert!(
        ManifestError::PackName("x".to_owned())
            .to_string()
            .contains("pack's name")
    );
}
