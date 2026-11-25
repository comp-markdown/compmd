//! The same cases as `@compmd/vite`'s `config.test.ts`, so the two readings agree, and the editor's key.

use serde_json::{Value, json};

use super::*;

/// `value` read as `compmd.config.json`.
fn read(value: &Value) -> Config {
    read_config(value, "compmd.config.json").expect("a configuration")
}

/// The problems `value` has, as `read_config` lists them; none when it reads.
fn problems_of(value: &Value) -> Vec<String> {
    read_config(value, "compmd.config.json")
        .err()
        .map(|error| error.problems)
        .unwrap_or_default()
}

/// The pack listed as `key` in `config`.
fn pack<'c>(config: &'c Config, key: &str) -> &'c ListedPack {
    &config
        .packs
        .iter()
        .find(|(listed, _)| listed == key)
        .expect("listed")
        .1
}

fn range(text: &str) -> VersionReq {
    VersionReq::parse(text).expect("a range")
}

#[test]
fn fills_in_every_default() {
    assert_eq!(read(&json!({})), Config::default());
    let config = Config::default();
    assert_eq!(
        (config.imports, config.fetch, config.origins),
        (Imports::Any, Fetch::Host, Origins::Listed(Vec::new()))
    );
}

#[test]
fn reads_the_origins_documents_may_reach_without_a_question_a_list_of_https_origins_or_every_origin()
 {
    assert_eq!(
        read(&json!({ "origins": ["https://cdn.example.com/", "http://localhost:8080"] })).origins,
        Origins::Listed(vec![
            "https://cdn.example.com".into(),
            "http://localhost:8080".into()
        ])
    );
    assert_eq!(read(&json!({ "origins": "any" })).origins, Origins::Any);
    assert_eq!(
        problems_of(&json!({ "origins": "all" })),
        [
            "origins: a list of origins, such as `[\"https://cdn.example.com\"]`, or `\"any\"` for every origin"
        ]
    );
    assert_eq!(
        problems_of(&json!({ "origins": ["http://cdn.example.com", "https://*.example.com"] })),
        [
            "origins: `http://cdn.example.com` must be `https:` (`http:` only for localhost)",
            "origins: `https://*.example.com` has a wildcard: name each origin",
        ]
    );
}

#[test]
fn reads_the_registries_by_scope_each_url_ending_in_a_slash_and_who_fetches() {
    let config = read(&json!({
        "registries": { "@kit": "https://npm.kit.example/npm", "*": "https://registry.npmjs.org/" },
        "fetch": "direct"
    }));
    assert_eq!(
        config.registries,
        BTreeMap::from([
            ("*".to_owned(), "https://registry.npmjs.org/".to_owned()),
            ("@kit".to_owned(), "https://npm.kit.example/npm/".to_owned()),
        ])
    );
    assert_eq!(config.fetch, Fetch::Direct);
    assert_eq!(
        read(&json!({ "registries": { "*": "http://localhost:4873" } })).registries["*"],
        "http://localhost:4873/"
    );
}

#[test]
fn takes_registries_named_in_full_on_https_with_nothing_secret_in_their_urls() {
    let registry = |url: Value| problems_of(&json!({ "registries": { "*": url } }));
    assert_eq!(
        registry(json!("/compmd/registry/")),
        [
            "registries[\"*\"]: `/compmd/registry/` isn't a URL: a registry is named in full, such as `https://registry.npmjs.org/`"
        ]
    );
    assert_eq!(
        registry(json!("http://npm.kit.example/")),
        [
            "registries[\"*\"]: `http://npm.kit.example/` must be `https:` (`http:` only for localhost)"
        ]
    );
    assert_eq!(
        registry(json!("https://me:token@npm.kit.example/")),
        [
            "registries[\"*\"]: `https://me:token@npm.kit.example/` must have no credentials, query or fragment: a registry's token belongs to the host's proxy"
        ]
    );
    assert_eq!(
        registry(json!("https://npm.kit.example/?token=x")),
        [
            "registries[\"*\"]: `https://npm.kit.example/?token=x` must have no credentials, query or fragment: a registry's token belongs to the host's proxy"
        ]
    );
    assert_eq!(registry(json!(42)), ["registries[\"*\"]: a registry's URL"]);
    assert_eq!(
        problems_of(&json!({ "registries": { "kit": "https://npm.kit.example/" } })),
        ["registries[\"kit\"]: a scope, such as `@kit`, or `*` for every other scope"]
    );
    assert_eq!(
        problems_of(&json!({ "registries": ["https://registry.npmjs.org/"], "fetch": "proxy" })),
        [
            "registries: an object, by scope (`@kit`), with `*` for every other scope",
            "fetch: `host` or `direct`"
        ]
    );
}

#[test]
fn reads_each_kind_of_pack_with_its_trust() {
    let config = read(&json!({
        "$schema": "./node_modules/@compmd/vite/compmd.config.schema.json",
        "packs": {
            "@kit/mermaid": {},
            "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com/", "http://localhost:8080"], "version": "^3.0.0" },
            "@acme/reviews": { "trust": "permissive", "host": true },
            "./components": { "trust": "trusted", "ambient": true }
        },
        "imports": "listed",
        "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }, { "pack": "@games/kit", "reason": "not for school" }],
        "maxPackSize": "5 MB",
        "libraries": ["d3", { "name": "chart.js", "imports": ["chart.js", "chart.js/auto"] }]
    }));
    let keys: Vec<&str> = config.packs.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "@kit/mermaid",
            "@acme/maps",
            "@acme/reviews",
            "./components"
        ],
        "in the order written"
    );
    assert_eq!(*pack(&config, "@kit/mermaid"), ListedPack::default());
    assert_eq!(
        *pack(&config, "@acme/maps"),
        ListedPack {
            trust: Trust::Permissive {
                network: vec![
                    "https://tiles.example.com".into(),
                    "http://localhost:8080".into()
                ],
                host: false,
            },
            ambient: false,
            version: Some(range("^3.0.0")),
        }
    );
    assert_eq!(
        pack(&config, "@acme/reviews").trust,
        Trust::Permissive {
            network: Vec::new(),
            host: true
        }
    );
    assert_eq!(
        *pack(&config, "./components"),
        ListedPack {
            trust: Trust::Trusted,
            ambient: true,
            version: None,
        }
    );
    assert_eq!(config.imports, Imports::Listed);
    let denied: Vec<(String, Option<VersionReq>, String)> = config
        .deny
        .iter()
        .map(|deny| {
            (
                deny.pack.to_string(),
                deny.version.clone(),
                deny.reason.clone(),
            )
        })
        .collect();
    assert_eq!(
        denied,
        [
            (
                "@bad/pack".to_owned(),
                Some(range("<2")),
                "a flaw found in 1.x".to_owned()
            ),
            ("@games/kit".to_owned(), None, "not for school".to_owned()),
        ]
    );
    assert_eq!(config.max_pack_size, Some(5e6));
    assert_eq!(
        config.libraries,
        [
            Library {
                name: "d3".into(),
                imports: vec!["d3".into()]
            },
            Library {
                name: "chart.js".into(),
                imports: vec!["chart.js".into(), "chart.js/auto".into()]
            },
        ]
    );
}

#[test]
fn takes_a_size_in_bytes_or_with_a_unit() {
    let size = |value: Value| read(&json!({ "maxPackSize": value })).max_pack_size;
    assert_eq!(size(json!(1200)), Some(1200.0));
    assert_eq!(size(json!("800 KB")), Some(800_000.0));
    assert_eq!(size(json!("1.5gb")), Some(1_500_000_000.0));
    assert_eq!(size(json!("12B")), Some(12.0));
}

#[test]
fn refuses_trust_it_cant_give_and_says_why() {
    assert_eq!(
        problems_of(&json!({ "packs": { "@acme/app": { "trust": "trusted" } } })),
        [
            "packs[\"@acme/app\"].trust: only a folder in the app may be trusted, never a registry pack; a registry pack that needs more is `permissive`"
        ]
    );
    assert_eq!(
        problems_of(
            &json!({ "packs": { "@acme/maps": { "network": ["https://tiles.example.com"], "host": true } } })
        ),
        [
            "packs[\"@acme/maps\"].network: only a permissive pack reaches the network",
            "packs[\"@acme/maps\"].host: only a permissive pack is given `host`",
        ]
    );
    assert_eq!(
        problems_of(&json!({ "packs": { "@acme/maps": { "trust": "open" } } })),
        ["packs[\"@acme/maps\"].trust: `strict`, `permissive` or `trusted`"]
    );
}

#[test]
fn takes_origins_only_for_the_network() {
    let network = |origin: &str| {
        problems_of(
            &json!({ "packs": { "@acme/maps": { "trust": "permissive", "network": [origin] } } }),
        )
    };
    assert_eq!(
        network("http://tiles.example.com"),
        [
            "packs[\"@acme/maps\"].network: `http://tiles.example.com` must be `https:` (`http:` only for localhost)"
        ]
    );
    assert_eq!(
        network("https://*.example.com"),
        ["packs[\"@acme/maps\"].network: `https://*.example.com` has a wildcard: name each origin"]
    );
    assert_eq!(
        network("https://tiles.example.com/v2"),
        [
            "packs[\"@acme/maps\"].network: `https://tiles.example.com/v2` must be an origin alone, without a path"
        ]
    );
    assert_eq!(
        network("tiles"),
        ["packs[\"@acme/maps\"].network: `tiles` isn't a URL"]
    );
    assert_eq!(
        problems_of(
            &json!({ "packs": { "@acme/maps": { "trust": "permissive", "network": "https://a.org" } } })
        ),
        ["packs[\"@acme/maps\"].network: a list of origins"]
    );
}

#[test]
fn lists_every_problem_at_once() {
    assert_eq!(
        problems_of(&json!({
            "pack": {},
            "packs": { "acme-maps": { "trsut": "permissive", "ambient": "yes", "version": "latest!" }, "@a/b": "strict" },
            "imports": "some",
            "deny": [{ "pack": "bad", "version": "~~", "reason": "x" }, { "pack": "@a/c" }],
            "maxPackSize": "large",
            "libraries": ["D3!", { "name": "d3" }]
        })),
        [
            "pack: not a key of the configuration; it takes $schema, packs, imports, deny, maxPackSize, libraries, registries, fetch, origins, editor",
            "packs[\"acme-maps\"]: a pack is a scoped name (`@scope/pack`) or a folder path (`./components`)",
            "packs[\"acme-maps\"].trsut: not an option; a pack takes trust, network, host, ambient, version",
            "packs[\"acme-maps\"].ambient: true or false",
            "packs[\"acme-maps\"].version: a semver range, such as `^1.2.0`",
            "packs[\"@a/b\"]: its options are an object, `{}` for none",
            "imports: `any` or `listed`",
            "deny[0].pack: a scoped name, `@scope/pack`",
            "deny[0].version: a semver range, such as `<2`",
            "deny[1]: `{pack, reason}`, and `version` for some versions only",
            "maxPackSize: bytes, or a size such as `\"5 MB\"`",
            "libraries[0]: an npm package name, such as `\"d3\"`, or `{ \"name\": \"d3\", \"imports\": [\"d3\"] }`",
            "libraries[1]: an npm package name, such as `\"d3\"`, or `{ \"name\": \"d3\", \"imports\": [\"d3\"] }`",
        ]
    );
}

#[test]
fn names_a_pack_as_the_processor_and_the_registry_do() {
    assert_eq!(
        problems_of(
            &json!({ "packs": { "@-x/y": {} }, "deny": [{ "pack": "@~kit/old", "reason": "it is old" }] })
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        problems_of(
            &json!({ "packs": { "@_x/y": {} }, "deny": [{ "pack": format!("@kit/{}", "a".repeat(210)), "reason": "too long" }] })
        ),
        [
            "packs[\"@_x/y\"]: a pack is a scoped name (`@scope/pack`) or a folder path (`./components`)",
            "deny[0].pack: a scoped name, `@scope/pack`",
        ]
    );
}

#[test]
fn refuses_what_isnt_the_shape_it_takes() {
    assert_eq!(problems_of(&json!([])), ["the configuration is an object"]);
    assert_eq!(
        problems_of(&json!({ "packs": [], "deny": {}, "libraries": "d3", "maxPackSize": 0 })),
        [
            "packs: an object, by pack name or folder path",
            "deny: a list of `{pack, version?, reason}`",
            "maxPackSize: bytes, or a size such as `\"5 MB\"`",
            "libraries: a list of npm package names, or `{name, imports}`",
        ]
    );
    assert_eq!(
        problems_of(
            &json!({ "packs": { "@acme/maps": { "trust": "permissive", "host": "yes" } } })
        ),
        ["packs[\"@acme/maps\"].host: true or false"]
    );
}

#[test]
fn names_where_the_configuration_came_from() {
    let error = read_config(&json!({ "imports": "some" }), "package.json (`compmd`)")
        .expect_err("a problem");
    assert_eq!(
        error.to_string(),
        "package.json (`compmd`):\n- imports: `any` or `listed`"
    );
}

#[test]
fn reads_the_scopes_the_editor_suggests() {
    assert_eq!(
        read(&json!({ "editor": { "suggest": ["@acme", "@kit"] } })).editor,
        Editor {
            suggest: vec!["@acme".into(), "@kit".into()]
        }
    );
    assert_eq!(read(&json!({ "editor": {} })).editor, Editor::default());
    assert_eq!(
        problems_of(&json!({ "editor": { "suggest": ["acme", "@ok"], "offer": true } })),
        [
            "editor.offer: not a key of `editor`; it takes suggest",
            "editor.suggest: `acme` isn't a scope, such as `@acme`"
        ]
    );
    assert_eq!(
        problems_of(&json!({ "editor": { "suggest": "@acme" } })),
        ["editor.suggest: a list of scopes, such as `[\"@acme\"]`"]
    );
    assert_eq!(
        problems_of(&json!({ "editor": ["@acme"] })),
        ["editor: an object, such as `{ \"suggest\": [\"@acme\"] }`"]
    );
}

#[test]
fn tells_a_folder_path_from_a_pack_name() {
    assert!(is_folder("./components"));
    assert!(is_folder("../shared/kit"));
    assert!(!is_folder("@acme/app"));
}
