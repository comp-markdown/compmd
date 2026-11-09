use std::sync::atomic::{AtomicU32, Ordering};

use std::path::PathBuf;

use super::*;

pub const HASH: &str = "1df336e75685de8c";

/// A folder of its own under the system's temporary folder, removed when dropped.
pub struct Folder(pub PathBuf);

impl Drop for Folder {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

/// A resource folder with `frames.json` holding `written`, a build named `HASH` with `strict.html` and
/// `chunks/runtime.js`, and a file beside the build.
pub fn resources(written: &str) -> Folder {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let folder = std::env::temp_dir().join(format!("compmd-tauri-{}-{n}", std::process::id()));
    let build = folder.join(HASH);
    fs::create_dir_all(build.join("chunks")).expect("made");
    fs::write(build.join("strict.html"), "<!doctype html>").expect("written");
    fs::write(build.join("chunks").join("runtime.js"), "export {}").expect("written");
    fs::write(folder.join("secret.txt"), "not served").expect("written");
    fs::write(folder.join(FRAMES_JSON), written).expect("written");
    Folder(folder)
}

/// `frames.json` naming `HASH`.
pub fn named() -> String {
    format!("{{\"hash\":\"{HASH}\"}}")
}

#[test]
fn the_build_is_the_folder_frames_json_names() {
    let folder = resources(&named());
    let frames = Frames::read(&folder.0);
    assert_eq!(
        frames.build(),
        Some(&Build {
            hash: HASH.to_owned(),
            files: Files::Folder(folder.0.join(HASH)),
        })
    );
    let custom = format!("compmd://localhost/frames/{HASH}/");
    let http = format!("http://compmd.localhost/frames/{HASH}/");
    assert_eq!(frames.base_over("compmd", false), Some(custom.clone()));
    assert_eq!(frames.base_over("compmd", true), Some(http.clone()));
    assert_eq!(
        frames.base("compmd"),
        Some(if OVER_HTTP { http } else { custom })
    );
}

#[test]
fn the_embedded_build_is_the_one_the_crate_was_built_with() {
    const FILES: &[(&str, &[u8])] = &[("strict.html", b"<!doctype html>")];
    assert_eq!(
        Frames::embedded_as(Some(HASH), FILES).build(),
        Some(&Build {
            hash: HASH.to_owned(),
            files: Files::Embedded(FILES),
        })
    );
    assert_eq!(Frames::embedded_as(None, &[]), Frames::none());
    assert_eq!(
        Frames::embedded(),
        Frames::embedded_as(embedded::HASH, embedded::FILES)
    );
}

#[test]
fn without_a_build_there_is_nothing() {
    for written in [
        "",
        "{}",
        "{\"hash\":\"../secret\"}",
        "{\"hash\":\"1DF336E75685DE8C\"}",
        "{\"hash\":\"1df336e7\"}",
        "{\"hash\":\"1df336e75685de8g\"}",
    ] {
        let frames = Frames::read(&resources(written).0);
        assert_eq!(frames, Frames::none(), "{written}");
        assert_eq!(frames.base("compmd"), None);
    }
    let gone = std::env::temp_dir().join("compmd-tauri-no-such-folder");
    assert_eq!(Frames::read(&gone), Frames::none());
}
