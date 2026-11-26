use std::collections::BTreeMap;
use std::time::Duration;

use super::*;
use crate::registry::{LocalRegistry, ServedPack, tarball};

const BASE: &str = "compmd://localhost/registry/";

fn notes() -> ServedPack {
    ServedPack::new("@kit/notes").version("1.0.0", &[("package.json", "{}")])
}

fn proxy(registry: &LocalRegistry, max_pack_size: u64) -> Proxy {
    let mut options = Options::new(BTreeMap::from([(
        "@kit".to_owned(),
        registry.url().to_owned(),
    )]));
    options.max_pack_size = max_pack_size;
    Proxy::new(options, Client::new(Duration::from_secs(5)))
}

fn body(response: &Response<Vec<u8>>) -> Value {
    serde_json::from_slice(response.body()).expect("JSON")
}

#[test]
fn metadata_passes_on_with_the_proxys_tarballs() {
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let answer = proxy(&registry, 1000).answer("@kit%2fnotes", BASE);
    assert_eq!(answer.status(), StatusCode::OK);
    assert_eq!(answer.headers()[header::CONTENT_TYPE], ABBREVIATED);
    assert_eq!(
        answer.headers()[header::CACHE_CONTROL],
        "public, max-age=300"
    );
    let dist = &body(&answer)["versions"]["1.0.0"]["dist"];
    assert_eq!(
        dist["tarball"],
        "compmd://localhost/registry/-/tarball/@kit/notes/1.0.0"
    );
    assert!(
        dist["integrity"]
            .as_str()
            .is_some_and(|integrity| integrity.starts_with("sha512-"))
    );
    assert_eq!(registry.asked(), ["/@kit%2fnotes"]);
}

#[test]
fn a_tarball_passes_on_as_its_bytes_up_to_the_size_cap() {
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let answer = proxy(&registry, 100_000).answer("-/tarball/@kit/notes/1.0.0", BASE);
    assert_eq!(answer.status(), StatusCode::OK);
    let files = BTreeMap::from([("package.json".to_owned(), b"{}".to_vec())]);
    assert_eq!(answer.body(), &tarball(&files));
    assert_eq!(
        answer.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    let capped = proxy(&registry, 10).answer("-/tarball/@kit/notes/1.0.0", BASE);
    assert_eq!(capped.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn what_no_registry_has_is_not_found_and_a_failing_registry_is_a_bad_gateway() {
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let proxy = proxy(&registry, 1000);
    for path in [
        "@kit%2fnone",
        "-/tarball/@kit/none/1.0.0",
        "-/tarball/@kit/notes/9.9.9",
        "@other%2fnotes",
        "not-a-pack",
        "%ff",
        "-/tarball/x",
    ] {
        assert_eq!(
            proxy.answer(path, BASE).status(),
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    let gone = {
        let registry = LocalRegistry::serve(&[]).expect("served");
        self::proxy(&registry, 1000)
    };
    assert_eq!(
        gone.answer("@kit%2fnotes", BASE).status(),
        StatusCode::BAD_GATEWAY
    );
}
