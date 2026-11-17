//! The serving contract's cases (`fixtures/respond/`), which `@compmd/host` runs too.

use std::collections::HashMap;
use std::path::PathBuf;

use compmd_pack::PackName;
use http::{Request, Response};
use serde::Deserialize;

use super::*;
use crate::store::{NoStore, StoredRelease};

/// The contract's cases, as `fixtures/respond/cases.json` has them.
#[derive(Deserialize)]
struct Contract {
    store: HashMap<String, String>,
    headers: HashMap<String, HashMap<String, Option<String>>>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    method: Option<String>,
    path: String,
    #[serde(default)]
    store: bool,
    status: u16,
    answer: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    body: String,
}

/// A store holding the files the cases name, by place.
struct Fixed(HashMap<String, String>);

impl Store for Fixed {
    fn releases(&self, _pack: &PackName) -> Vec<StoredRelease> {
        Vec::new()
    }

    fn file(&self, place: &Place) -> Option<Vec<u8>> {
        self.0
            .get(&place.to_string())
            .map(|body| body.clone().into_bytes())
    }
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/respond")
}

fn header<'a>(response: &'a Response<Vec<u8>>, name: &str) -> Option<&'a str> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

#[test]
fn every_case_of_the_contract_gets_its_answer() {
    let cases: Contract =
        serde_json::from_slice(&std::fs::read(fixtures().join("cases.json")).expect("the cases"))
            .expect("cases.json");
    let bundle = Bundle::read(&fixtures().join("bundle")).expect("the bundle");
    let store = Fixed(cases.store.clone());
    for case in &cases.cases {
        let request = Request::builder()
            .method(case.method.as_deref().unwrap_or("GET"))
            .uri(&case.path)
            .body(())
            .expect("a request");
        let response = if case.store {
            respond(&bundle, &store, &request)
        } else {
            respond(&bundle, &NoStore, &request)
        };
        let label = format!("{} {}", request.method(), case.path);
        assert_eq!(response.status().as_u16(), case.status, "{label}");
        assert_eq!(
            String::from_utf8_lossy(response.body()),
            case.body,
            "{label}"
        );
        if let Some(kind) = &case.kind {
            assert_eq!(
                header(&response, "content-type"),
                Some(kind.as_str()),
                "{label}"
            );
        }
        for (name, value) in &cases.headers[&case.answer] {
            assert_eq!(header(&response, name), value.as_deref(), "{label}: {name}");
        }
    }
}

#[test]
fn the_cases_store_lists_no_releases() {
    assert_eq!(
        Fixed(HashMap::new()).releases(&"@kit/charts".parse().expect("a name")),
        []
    );
}

/// Paths that reach `respond` as written only over `http` types (a URL parser resolves `.` parts before a Fetch API
/// handler sees them): refused.
#[test]
fn a_path_with_a_dot_part_or_without_its_leading_slash_is_refused() {
    let bundle = Bundle::read(&fixtures().join("bundle")).expect("the bundle");
    for uri in [
        "compmd://localhost",
        "/frames/1df336e75685de8c/./strict.html",
    ] {
        let request = Request::get(uri).body(()).expect("a request");
        assert_eq!(
            respond(&bundle, &NoStore, &request).status().as_u16(),
            404,
            "{uri}"
        );
    }
}
