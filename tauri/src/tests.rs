use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

use super::*;
use crate::frames::tests::{HASH, named, resources};

/// An app on Tauri's mock runtime with the plugin set up by `builder`.
fn app(builder: Builder) -> tauri::App<MockRuntime> {
    mock_builder()
        .plugin(builder.build())
        .build(mock_context(noop_assets()))
        .expect("the app builds")
}

#[test]
fn the_app_gets_the_frames_base_from_the_plugin_and_its_command() {
    let folder = resources(&named());
    let app = app(Builder::new().scheme("frames").frames(&folder.0));
    let base = app.compmd().base().expect("a build");
    assert!(base.ends_with(&format!("/frames/{HASH}/")), "{base}");
    assert!(base.contains("frames"), "{base}");
    assert_eq!(app.compmd().scheme, "frames");
    assert_eq!(command::base(app.handle().clone()), Some(base));
    // What the scheme answers, as a webview asks it.
    let request =
        tauri::http::Request::get(format!("frames://localhost/frames/{HASH}/strict.html"))
            .body(Vec::new())
            .expect("a request");
    let page = answer(app.handle(), &request);
    assert_eq!(page.status(), tauri::http::StatusCode::OK);
    assert_eq!(page.body(), b"<!doctype html>");
}

#[test]
fn an_app_built_without_its_frames_gives_no_base() {
    let missing = std::env::temp_dir().join("compmd-tauri-no-frames");
    let app = app(Builder::default().frames(missing));
    assert_eq!(app.compmd().base(), None);
    assert_eq!(app.compmd().scheme, SCHEME);
}

#[test]
fn the_frames_are_under_the_apps_resources_unless_absolute() {
    let resources = std::env::temp_dir().join("app-resources");
    assert_eq!(
        folder_of(Path::new("compmd"), Some(resources.clone())),
        Some(resources.join("compmd"))
    );
    assert_eq!(folder_of(Path::new("compmd"), None), None);
    let absolute = std::env::temp_dir().join("frames");
    assert_eq!(folder_of(&absolute, None), Some(absolute.clone()));
}

#[test]
fn a_relative_folder_is_read_from_the_apps_resources() {
    // The mock runtime's resources aren't on disk, so there is no build there.
    let app = app(Builder::new());
    assert_eq!(app.compmd().frames, Frames::none());
}

#[test]
fn a_schemes_name_is_lowercase_letters_digits_and_plus_minus_dot() {
    for good in ["compmd", "my-frames", "app+frames", "a.b", "x1"] {
        assert!(is_scheme(good), "{good}");
    }
    for bad in ["", "1app", "App", "my frames", "a/b", "a:b", "-a"] {
        assert!(!is_scheme(bad), "{bad}");
    }
}

#[test]
#[should_panic(expected = "isn't a scheme's name")]
fn a_bad_scheme_is_a_mistake_in_the_app() {
    drop(Builder::new().scheme("Not A Scheme").build::<MockRuntime>());
}
