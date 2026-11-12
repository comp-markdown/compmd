use std::fs;

use super::*;

/// A fresh folder named `name` in the system's temporary folder, with `files` by `/`-separated path.
fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("compmd-tauri-build-{name}"));
    let _ = fs::remove_dir_all(&root);
    for (path, text) in files {
        let file = root.join(path);
        fs::create_dir_all(file.parent().expect("a folder")).expect("the folder is made");
        fs::write(file, text).expect("the file is written");
    }
    root
}

const MANIFEST_TEXT: &str =
    r#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}, "source": "compmd://localhost/"}"#;

#[test]
fn a_bundle_embeds_its_manifest_and_what_it_serves_sorted() {
    let root = folder(
        "full",
        &[
            ("manifest.json", MANIFEST_TEXT),
            ("frames/1df336e75685de8c/strict.html", "<!doctype html>"),
            ("frames/1df336e75685de8c/chunks/a.js", ""),
            ("packs/@kit/bars@1.0.0/package.json", "{}"),
            ("_headers", "for static hosts only"),
        ],
    );
    let code = embedding(&root).expect("a bundle");
    let paths: Vec<&str> = code
        .lines()
        .filter_map(|line| line.trim().strip_prefix("(\"")?.split('"').next())
        .collect();
    assert_eq!(
        paths,
        [
            "frames/1df336e75685de8c/chunks/a.js",
            "frames/1df336e75685de8c/strict.html",
            "manifest.json",
            "packs/@kit/bars@1.0.0/package.json",
        ]
    );
    assert!(
        code.starts_with("::core::option::Option::Some(&[\n"),
        "{code}"
    );
    assert!(code.contains("include_bytes!("), "{code}");
}

#[test]
fn the_build_script_gets_the_code_and_what_to_tell_cargo() {
    let root = folder("embed", &[("manifest.json", MANIFEST_TEXT)]);
    let out = folder("embed-out", &[("keep", "")]);
    let said = embed(&root, &out);
    assert_eq!(said, [format!("cargo:rerun-if-changed={}", root.display())]);
    let code = fs::read_to_string(out.join(EMBEDDED)).expect("written");
    assert!(code.contains("\"manifest.json\""), "{code}");
    // No bundle: none, with a warning.
    let empty = folder("embed-empty", &[]);
    let said = embed(&empty, &out);
    assert!(
        said[1].starts_with("cargo:warning=tauri-plugin-compmd: no bundle in "),
        "{said:?}"
    );
    assert_eq!(
        fs::read_to_string(out.join(EMBEDDED)).expect("written"),
        NONE
    );
}

#[test]
#[should_panic(expected = "manifest is format 2")]
fn a_bundle_that_cant_be_used_stops_the_build() {
    let root = folder("embed-format", &[("manifest.json", r#"{"format": 2}"#)]);
    let out = folder("embed-format-out", &[("keep", "")]);
    drop(embed(&root, &out));
}

#[test]
#[should_panic(expected = "can't be written into OUT_DIR")]
fn an_out_dir_that_cant_be_written_stops_the_build() {
    let root = folder("embed-unwritable", &[("manifest.json", MANIFEST_TEXT)]);
    drop(embed(&root, &root.join("missing")));
}

#[test]
#[should_panic(expected = "runs in the app's build script")]
fn bundle_runs_in_a_build_script_only() {
    // Cargo sets OUT_DIR for a build script, not for this test.
    bundle("compmd");
}

#[test]
fn bundle_embeds_the_folder_under_the_apps_manifest_into_out_dir() {
    let app = folder("app", &[("compmd/manifest.json", MANIFEST_TEXT)]);
    let out = folder("app-out", &[("keep", "")]);
    let var = |name: &str| match name {
        "CARGO_MANIFEST_DIR" => Some(app.clone().into_os_string()),
        "OUT_DIR" => Some(out.clone().into_os_string()),
        _ => None,
    };
    bundle_with(Path::new("compmd"), var);
    let code = fs::read_to_string(out.join(EMBEDDED)).expect("written");
    assert!(
        code.contains(&app.join("compmd").join(MANIFEST).display().to_string()),
        "{code}"
    );
}

#[cfg(unix)]
#[test]
fn a_folder_that_cant_be_read_is_refused() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = folder(
        "unreadable",
        &[("manifest.json", MANIFEST_TEXT), ("packs/x", "")],
    );
    let packs = root.join("packs");
    fs::set_permissions(&packs, fs::Permissions::from_mode(0o000)).expect("set");
    let result = embedding(&root);
    fs::set_permissions(&packs, fs::Permissions::from_mode(0o755)).expect("set");
    assert!(matches!(result, Err(BuildError::Unreadable { folder, .. }) if folder == packs));
}

#[test]
fn a_folder_without_a_bundle_is_none() {
    let root = folder("none", &[]);
    assert!(matches!(embedding(&root), Err(BuildError::NoBundle(found)) if found == root));
}

#[test]
fn a_manifest_of_another_format_says_which_plugin_reads_it() {
    let root = folder("format", &[("manifest.json", r#"{"format": 2}"#)]);
    let error = embedding(&root).expect_err("refused");
    assert!(matches!(error, BuildError::Format { found: 2, known: 1 }));
    assert!(
        error.to_string().contains(env!("CARGO_PKG_VERSION")),
        "{error}"
    );
}

#[test]
fn a_manifest_that_isnt_one_is_refused() {
    let root = folder("broken", &[("manifest.json", "{")]);
    assert!(matches!(embedding(&root), Err(BuildError::Manifest(_))));
}

#[test]
fn a_bundle_built_for_http_or_a_bad_scheme_is_refused() {
    let http = folder(
        "http",
        &[(
            "manifest.json",
            r#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#,
        )],
    );
    assert!(matches!(embedding(&http), Err(BuildError::NoScheme)));
    let bad = folder(
        "scheme",
        &[(
            "manifest.json",
            r#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}, "source": "My_Frames://localhost/"}"#,
        )],
    );
    let error = embedding(&bad).expect_err("refused");
    assert!(
        matches!(&error, BuildError::Scheme(scheme) if scheme == "My_Frames"),
        "{error}"
    );
}
