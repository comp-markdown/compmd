//! The processor as a host uses it, one scenario a test: what the host gives it (files, packs, a policy, data, a
//! render context), what it asks the host for, and what a document gets when something is wrong. Each test reads as
//! an example of the public API, a good use or a bad one.

use compmd::context::{RenderContext, parse_time};
use compmd::diagnostic::{Diagnostic, Stage};
use compmd::expand::Options;
use compmd::host::{
    Data, Fetch, Fetched, LoadedRelease, NeededPack, Release, Time, newest_matching,
};
use compmd::policy::{Policy, Trust};
use compmd::processor::{Expanded, LibraryModule, Validated};
use compmd::{Ambient, ParseOptions, Processor, TextEdit};
use semver::Version;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

/// What a host gives the processor; each test changes what it needs.
struct Host {
    files: HashMap<String, String>,
    packs: HashMap<String, Vec<LoadedRelease>>,
    policy: Policy,
    ambient: Ambient,
    parse: ParseOptions,
    allowed_origins: Vec<String>,
    remote: Rc<dyn Fetch>,
    data: Rc<dyn Data>,
    context: RenderContext,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            files: HashMap::new(),
            packs: HashMap::new(),
            policy: Policy::default(),
            ambient: Ambient::default(),
            parse: ParseOptions::default(),
            allowed_origins: Vec::new(),
            remote: Rc::new(()),
            data: Rc::new(()),
            context: RenderContext {
                now: parse_time("2026-11-02T08:50:00Z").expect("a time"),
                tz: "UTC".into(),
                time: Rc::new(Zones),
                inputs: serde_json::Map::new(),
            },
        }
    }
}

impl Host {
    /// `text`, as `a.comp.md`, through stage 3: what `sources` is asked of before loading.
    fn validate(&self, text: &str) -> Processor<Validated> {
        Processor::new("a.comp.md", text)
            .parse(&self.parse)
            .resolve_with(
                self.files.clone(),
                self.packs.clone(),
                &self.ambient,
                &self.policy,
            )
            .validate(&self.allowed_origins)
    }

    /// `text`, as `a.comp.md`, through every stage.
    fn process(&self, text: &str) -> Processor<Expanded> {
        self.validate(text)
            .load(
                self.context.clone(),
                Rc::clone(&self.remote),
                Rc::clone(&self.data),
            )
            .expand(&Options::default())
    }

    /// Adds `release` to the packs.
    fn with(mut self, name: &str, release: LoadedRelease) -> Self {
        self.packs
            .entry(name.to_string())
            .or_default()
            .push(release);
        self
    }
}

/// The time zones the tests use.
struct Zones;

impl Time for Zones {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        match zone {
            "UTC" => Some(0),
            "Asia/Tokyo" => Some(9 * 3600),
            _ => None,
        }
    }
}

/// A release of the pack `name` at `version` whose document is `document`, with `manifest` added to its
/// `package.json`, and an integrity of its own.
fn release(name: &str, version: &str, document: &str, manifest: Value) -> LoadedRelease {
    let mut package = json!({ "name": name, "version": version });
    if let (Some(package), Value::Object(more)) = (package.as_object_mut(), manifest) {
        package.extend(more);
    }
    LoadedRelease {
        release: Release {
            version: Version::parse(version).expect("a version"),
            integrity: Some(format!("sha512-{name}@{version}")),
            broken: None,
        },
        files: HashMap::from([
            ("package.json".to_string(), package.to_string()),
            ("index.comp.md".to_string(), document.to_string()),
        ]),
    }
}

/// A pack's document that exports one `compmd` component, `card`, showing `text`.
fn card_pack(text: &str) -> String {
    format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=card\n---\n{text}\n```\n"
    )
}

/// A pack's document that exports one JavaScript component, `name`, with its signature.
fn js_pack(name: &str) -> String {
    format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```js component={name}\nexport default () => {{}};\n```\n\n```compmd component={name} signature\nparams: {{}}\n```\n"
    )
}

/// A document that imports `names` from each of `froms`, then uses `body`.
fn importing(froms: &[&str], body: &str) -> String {
    let imports: String = froms
        .iter()
        .flat_map(|from| ["    - from: \"", from, "\"\n      names: \"*\"\n"])
        .collect();
    format!("---\nmd:\n  import:\n{imports}---\n\n{body}")
}

/// The diagnostics as `code message`.
fn diagnostics<S>(processor: &Processor<S>) -> Vec<String> {
    processor
        .diagnostics()
        .iter()
        .map(|diagnostic| format!("{} {}", diagnostic.code, diagnostic.message))
        .collect()
}

/// The codes of the diagnostics.
fn codes<S>(processor: &Processor<S>) -> Vec<String> {
    processor
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

/// Every text the expanded tree shows, joined.
fn shown(processor: &Processor<Expanded>) -> String {
    fn collect(value: &Value, out: &mut String) {
        match value {
            Value::Object(node) => {
                if node.get("type") == Some(&json!("text")) {
                    out.push_str(node["value"].as_str().unwrap_or_default());
                }
                for child in node.values() {
                    collect(child, out);
                }
            }
            Value::Array(items) => items.iter().for_each(|item| collect(item, out)),
            _ => {}
        }
    }
    let mut out = String::new();
    collect(
        &serde_json::to_value(processor.expanded()).expect("a tree"),
        &mut out,
    );
    out
}

mod packs {
    use super::*;

    #[test]
    fn a_document_imports_a_pack_the_host_gives_and_uses_its_components() {
        let host = Host::default().with(
            "@acme/kit",
            release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
        );
        let processed = host.process(&importing(&["@acme/kit@1"], ":: card[Card]\n"));
        assert_eq!(diagnostics(&processed), Vec::<String>::new());
        assert_eq!(shown(&processed), "A card.");
    }

    #[test]
    fn a_missing_pack_is_listed_for_the_host_to_get_and_the_document_processed_again_with_it() {
        let text = importing(&["@acme/kit@^1.1"], ":: card[Card]\n");
        let host = Host::default();
        let first = host.validate(&text);
        let needed = first.sources(&host.context).packs;
        assert_eq!(
            needed,
            [NeededPack {
                pack: "@acme/kit".into(),
                range: "^1.1".into(),
                integrity: None,
                library: false
            }]
        );
        // The host's registry has these versions; it takes the one the processor would.
        let versions = ["1.0.0", "1.1.4", "2.0.0"].map(|v| Version::parse(v).expect("a version"));
        let chosen = newest_matching(&needed[0].range, &versions).expect("a version");
        assert_eq!(chosen.to_string(), "1.1.4");
        let host = host.with(
            "@acme/kit",
            release("@acme/kit", "1.1.4", &card_pack("A card."), json!({})),
        );
        let again = host.process(&text);
        assert_eq!(again.sources().packs, Vec::<NeededPack>::new());
        assert_eq!(shown(&again), "A card.");
    }

    #[test]
    fn a_release_the_range_doesnt_take_leaves_the_pack_needed_and_the_import_f026() {
        let host = Host::default().with(
            "@acme/kit",
            release("@acme/kit", "2.0.0", &card_pack("A card."), json!({})),
        );
        let processed = host.process(&importing(&["@acme/kit@1"], ":: card[Card]\n"));
        // A wildcard import that loads nothing may hold any name, so `card` isn't F002 as well.
        assert_eq!(codes(&processed), ["F026"]);
        assert_eq!(processed.sources().packs.len(), 1);
    }

    #[test]
    fn an_import_pinned_to_the_releases_integrity_loads() {
        let host = Host::default().with(
            "@acme/kit",
            release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
        );
        let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-@acme/kit@1.2.0\n      names: [card]\n---\n\n:: card[Card]\n";
        assert_eq!(diagnostics(&host.process(text)), Vec::<String>::new());
    }

    #[test]
    fn an_import_pinned_to_another_integrity_is_f035_and_loads_nothing() {
        let host = Host::default().with(
            "@acme/kit",
            release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
        );
        let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-other\n      names: [card]\n---\n\n:: card[Card]\n";
        assert_eq!(
            diagnostics(&host.process(text))[0],
            "F035 `@acme/kit@1.2.0` doesn't match this `integrity`; the import is dropped"
        );
    }

    #[test]
    fn a_release_the_host_found_broken_is_f035_with_its_reason() {
        let mut broken = release("@acme/kit", "1.2.0", &card_pack("A card."), json!({}));
        broken.release.broken = Some("its tarball doesn't match `dist.integrity`".into());
        let processed = Host::default()
            .with("@acme/kit", broken)
            .process(&importing(&["@acme/kit@1"], ""));
        assert_eq!(codes(&processed), ["F035"]);
        assert!(diagnostics(&processed)[0].contains("its tarball doesn't match `dist.integrity`"));
    }

    #[test]
    fn a_library_pack_a_pack_names_is_listed_then_given_to_its_components() {
        let bars = release(
            "@acme/bars",
            "1.0.0",
            &js_pack("bars"),
            json!({ "peerDependencies": { "@kit/d3": "^7" } }),
        );
        let host = Host::default().with("@acme/bars", bars);
        let text = importing(&["@acme/bars@1"], ":: bars[Bars]\n");
        let first = host.validate(&text);
        assert_eq!(
            first.sources(&host.context).packs,
            [NeededPack {
                pack: "@kit/d3".into(),
                range: "^7".into(),
                integrity: None,
                library: true
            }]
        );
        assert_eq!(codes(&first), ["W121"], "until the host gets it");
        let mut d3 = release(
            "@kit/d3",
            "7.9.0",
            "",
            json!({ "compmd": { "library": "dist/d3.js" } }),
        );
        d3.files.insert(
            "dist/d3.js".into(),
            "export const version = '7.9.0';".into(),
        );
        let again = host.with("@kit/d3", d3).process(&text);
        assert_eq!(diagnostics(&again), Vec::<String>::new());
        let [bars] = again.host_components().try_into().expect("one component");
        let module = LibraryModule {
            pack: "@kit/d3".into(),
            version: Version::new(7, 9, 0),
            path: "dist/d3.js".into(),
            libraries: BTreeMap::new(),
        };
        assert_eq!(
            bars.libraries,
            BTreeMap::from([("@kit/d3".to_string(), module)])
        );
    }
}

mod policy {
    use super::*;

    fn kit() -> Host {
        Host::default().with(
            "@acme/kit",
            release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
        )
    }

    fn with_policy(host: Host, policy: Value) -> Host {
        Host {
            policy: serde_json::from_value(policy).expect("a policy"),
            ..host
        }
    }

    #[test]
    fn under_imports_listed_a_pack_the_configuration_lists_is_imported() {
        let host = with_policy(
            kit(),
            json!({ "imports": "listed", "packs": { "@acme/kit": { "level": "strict" } } }),
        );
        assert_eq!(
            diagnostics(&host.process(&importing(&["@acme/kit@1"], ""))),
            Vec::<String>::new()
        );
    }

    #[test]
    fn under_imports_listed_any_other_pack_is_f026_and_never_asked_for() {
        let host = with_policy(Host::default(), json!({ "imports": "listed" }));
        let validated = host.validate(&importing(&["@acme/kit@1"], ""));
        assert_eq!(
            diagnostics(&validated),
            [
                "F026 this host allows only the packs its configuration lists, and `@acme/kit` isn't one; the import is dropped"
            ]
        );
        assert_eq!(
            validated.sources(&host.context).packs,
            Vec::<NeededPack>::new()
        );
    }

    #[test]
    fn a_pack_the_host_denies_is_f026_with_its_reason() {
        let host = with_policy(
            kit(),
            json!({ "deny": [{ "pack": "@acme/kit", "reason": "draws fake prompts" }] }),
        );
        assert_eq!(
            diagnostics(&host.process(&importing(&["@acme/kit@1"], "")))[0],
            "F026 `@acme/kit@1.2.0` is denied here: draws fake prompts; the import is dropped"
        );
    }

    #[test]
    fn a_denied_range_leaves_the_other_versions_allowed() {
        let host = with_policy(
            kit(),
            json!({ "deny": [{ "pack": "@acme/kit", "version": "<1.2", "reason": "a flaw found in 1.1" }] }),
        );
        assert_eq!(
            diagnostics(&host.process(&importing(&["@acme/kit@1"], ""))),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_pack_the_host_left_out_is_f026_with_why_and_never_asked_for() {
        let host = with_policy(
            Host::default(),
            json!({ "refused": { "@acme/kit": "larger than this host serves" } }),
        );
        let validated = host.validate(&importing(&["@acme/kit@1"], ""));
        assert_eq!(
            diagnostics(&validated),
            [
                "F026 `@acme/kit` isn't available here: larger than this host serves; the import is dropped"
            ]
        );
        assert_eq!(
            validated.sources(&host.context).packs,
            Vec::<NeededPack>::new()
        );
    }

    #[test]
    fn each_packs_components_have_its_trust_level_strict_unless_listed() {
        let host = Host::default()
            .with(
                "@acme/charts",
                release("@acme/charts", "1.0.0", &js_pack("chart"), json!({})),
            )
            .with(
                "@acme/maps",
                release("@acme/maps", "1.0.0", &js_pack("map"), json!({})),
            )
            .with(
                "@acme/stars",
                release("@acme/stars", "1.0.0", &js_pack("stars"), json!({})),
            );
        let host = with_policy(
            host,
            json!({ "packs": {
                "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true },
                "@acme/stars": { "level": "trusted" }
            } }),
        );
        let processed = host.process(&importing(
            &["@acme/charts@1", "@acme/maps@1", "@acme/stars@1"],
            "",
        ));
        let trust: BTreeMap<String, Trust> = processed
            .host_components()
            .into_iter()
            .map(|component| (component.name, component.trust))
            .collect();
        assert_eq!(
            trust,
            BTreeMap::from([
                ("chart".to_string(), Trust::Strict),
                (
                    "map".to_string(),
                    Trust::Permissive {
                        network: vec!["https://tiles.example.com".into()],
                        host: true
                    }
                ),
                ("stars".to_string(), Trust::Trusted),
            ])
        );
    }

    /// A Svelte component, with its signature, and a use of it.
    const BADGE: &str = "```svelte component=badge\n<b>hi</b>\n```\n\n```compmd component=badge signature\nparams: {}\n```\n\n:: badge[Badge]\n";

    #[test]
    fn a_component_on_a_runtime_the_host_runs_is_given_to_the_renderer() {
        let host = with_policy(
            Host::default(),
            json!({ "runtimes": [{ "name": "svelte", "version": "5.57.2" }] }),
        );
        let processed = host.process(BADGE);
        assert_eq!(codes(&processed), Vec::<String>::new());
        assert_eq!(processed.host_components().len(), 1);
    }

    #[test]
    fn a_component_on_a_runtime_the_host_doesnt_run_is_w102_and_left_out() {
        let host = with_policy(
            Host::default(),
            json!({ "runtimes": [{ "name": "react", "version": "19.3.0" }] }),
        );
        let processed = host.process(BADGE);
        assert_eq!(codes(&processed), ["W102"]);
        assert_eq!(processed.host_components(), []);
        assert_eq!(shown(&processed), "Badge", "its fallback");
    }
}

mod what_the_processor_asks_the_host {
    use super::*;

    /// Remote data that records the URLs it is asked for, and answers each with `{ "n": 1 }`.
    #[derive(Default)]
    struct Recording(RefCell<Vec<String>>);

    impl Fetch for Recording {
        fn fetch(&self, url: &str) -> Option<Fetched> {
            self.0.borrow_mut().push(url.to_string());
            Some(Fetched {
                text: "{\"n\": 1}".into(),
                content_type: Some("application/json".into()),
            })
        }
    }

    impl Data for Recording {
        fn data(&self, source: &str) -> Option<Value> {
            self.0.borrow_mut().push(source.to_string());
            Some(json!({ "n": 1 }))
        }
    }

    const DATA: &str = "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: allowed\n    - from: https://b.org/y.json\n      as: elsewhere\n---\n";

    #[test]
    fn remote_data_is_fetched_only_from_the_origins_the_host_allows() {
        let recording = Rc::new(Recording::default());
        let host = Host {
            allowed_origins: vec!["https://a.org".into()],
            remote: Rc::clone(&recording) as Rc<dyn Fetch>,
            ..Host::default()
        };
        let processed = host.process(DATA);
        assert_eq!(*recording.0.borrow(), ["https://a.org/x.json"]);
        assert_eq!(codes(&processed), ["F028"], "`https://b.org` isn't allowed");
    }

    #[test]
    fn a_hosts_data_is_asked_for_by_its_source_without_host() {
        let recording = Rc::new(Recording::default());
        let host = Host {
            data: Rc::clone(&recording) as Rc<dyn Data>,
            ..Host::default()
        };
        let processed = host.process("---\nmd:\n  data:\n    - from: host:tasks?status=open\n      as: tasks\n---\n\n{{md.data.tasks.n}}\n");
        assert_eq!(*recording.0.borrow(), ["tasks?status=open"]);
        assert_eq!(shown(&processed), "1");
    }

    #[test]
    fn sources_lists_what_a_render_needs_before_anything_is_fetched() {
        let host = Host {
            allowed_origins: vec!["https://a.org".into()],
            ..Host::default()
        };
        let needed = host.validate(DATA).sources(&host.context);
        assert_eq!(needed.remote, ["https://a.org/x.json"]);
    }

    #[test]
    fn a_relative_import_is_read_from_the_hosts_files_and_one_that_leaves_the_root_is_f026() {
        let host = Host {
            files: HashMap::from([("parts.comp.md".to_string(), card_pack("A part."))]),
            ..Host::default()
        };
        let text = "---\nmd:\n  import:\n    - from: ./parts.comp.md\n      names: [card]\n    - from: ../outside.comp.md\n      names: \"*\"\n---\n\n:: card[Card]\n";
        let processed = host.process(text);
        assert_eq!(shown(&processed), "A part.");
        assert_eq!(
            diagnostics(&processed),
            ["F026 `../outside.comp.md` leaves its root"]
        );
    }
}

mod edits_and_reloads {
    use super::*;

    #[test]
    fn an_edit_to_the_frontmatter_expands_the_whole_document_again() {
        let text = "---\ntitle: One\n---\n\n# {{md.doc.title}}\n\nText.\n";
        let processed = Host::default().process(text);
        let start = text.find("One").expect("the title");
        let edited = processed.edit(&TextEdit {
            start,
            end: start + 3,
            text: "Two".into(),
        });
        let splice = edited.splice();
        assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 2, 2));
        assert_eq!(shown(&edited), "TwoText.");
    }

    #[test]
    fn an_edit_that_breaks_the_frontmatter_is_f033_and_the_fix_clears_it() {
        let text = "---\ntitle: One\n---\n\nText.\n";
        let processed = Host::default().process(text);
        let start = text.find("One").expect("the title");
        let broken = processed.edit(&TextEdit {
            start,
            end: start,
            text: "&anchor ".into(),
        });
        assert_eq!(codes(&broken), ["F033"]);
        let fixed = broken.edit(&TextEdit {
            start,
            end: start + "&anchor ".len(),
            text: String::new(),
        });
        assert_eq!(codes(&fixed), Vec::<String>::new());
    }

    #[test]
    fn after_an_edit_adds_a_data_import_sources_lists_it_and_a_reload_fills_it_in() {
        let host = Host {
            allowed_origins: vec!["https://a.org".into()],
            ..Host::default()
        };
        let processed = host.process("Text.\n");
        let frontmatter = "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: x\n---\n\n{{md.data.x.n}}\n\n";
        let edited = processed.edit(&TextEdit {
            start: 0,
            end: 0,
            text: frontmatter.into(),
        });
        assert_eq!(edited.sources().remote, ["https://a.org/x.json"]);
        // Not fetched yet: the import is F026, and its interpolation, of `null`, F024.
        assert_eq!(codes(&edited), ["F026", "F024"]);
        let fetched = HashMap::from([(
            "https://a.org/x.json".to_string(),
            Fetched {
                text: "{\"n\": 7}".into(),
                content_type: Some("application/json".into()),
            },
        )]);
        let reloaded = edited.reload(host.context.clone(), fetched, ());
        assert_eq!(codes(&reloaded), Vec::<String>::new());
        assert_eq!(shown(&reloaded), "7Text.");
    }
}

mod the_render_context {
    use super::*;

    #[test]
    fn md_today_is_the_date_where_the_document_is_read() {
        let text = "Today is :value[the date]{of=(string(md.today()))}.\n";
        let today = |host: &Host| {
            shown(&host.process(text))
                .trim_start_matches("Today is ")
                .trim_end_matches('.')
                .to_string()
        };
        assert_eq!(today(&Host::default()), "2026-11-02");
        // At 20:30 UTC it is still the 2nd in UTC, and already the 3rd in Tokyo.
        let evening = |tz: &str| Host {
            context: RenderContext {
                tz: tz.into(),
                now: parse_time("2026-11-02T20:30:00Z").expect("a time"),
                ..Host::default().context
            },
            ..Host::default()
        };
        assert_eq!(today(&evening("UTC")), "2026-11-02");
        assert_eq!(today(&evening("Asia/Tokyo")), "2026-11-03");
    }

    #[test]
    fn an_inputs_value_from_the_host_is_used_and_one_of_the_wrong_type_isnt() {
        let text = "---\nmd:\n  inputs:\n    region: eu | us = eu\n---\n\n{{md.input.region}}\n";
        let given = |value: Value| Host {
            context: RenderContext {
                inputs: serde_json::Map::from_iter([("region".to_string(), value)]),
                ..Host::default().context
            },
            ..Host::default()
        };
        assert_eq!(shown(&given(json!("us")).process(text)), "us");
        assert_eq!(shown(&given(json!(42)).process(text)), "eu", "its default");
    }
}

mod diagnostics_and_their_stages {
    use super::*;

    #[test]
    fn each_diagnostic_says_which_stage_found_it() {
        // One mistake for each stage. The last only evaluating finds: the data is loaded after checking.
        let text = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: missing\n    - from: ./items.json\n      as: items\n---\n\n:: box[B] {title=\"open\n\n:: nope[N]\n\n::: box {border=wiggly}\nX\n::: /box\n\nThe sixth: :value[none]{of=(md.data.items[5])}.\n";
        let host = Host {
            files: HashMap::from([("items.json".to_string(), "[1]".to_string())]),
            ..Host::default()
        };
        let processed = host.process(text);
        let stages: BTreeMap<String, Stage> = processed
            .diagnostics()
            .iter()
            .map(|diagnostic: &Diagnostic| (diagnostic.code.to_string(), diagnostic.stage))
            .collect();
        assert_eq!(
            stages,
            BTreeMap::from([
                ("F011".to_string(), Stage::Parse),
                ("F002".to_string(), Stage::Resolve),
                ("F009".to_string(), Stage::Validate),
                ("F026".to_string(), Stage::Load),
                ("F024".to_string(), Stage::Evaluate),
            ])
        );
    }

    #[test]
    fn a_hosts_metadata_is_a_document_without_frontmatter_s_frontmatter() {
        let host = Host {
            parse: ParseOptions {
                metadata: Some(json!({ "title": "From the host" })),
            },
            ..Host::default()
        };
        let processed = host.process("{{md.doc.title}}\n");
        assert_eq!(diagnostics(&processed), Vec::<String>::new());
        assert_eq!(shown(&processed), "From the host");
    }

    #[test]
    fn a_hosts_metadata_gives_every_md_key_a_frontmatter_would_imports_too() {
        let host = Host {
            parse: ParseOptions {
                metadata: Some(
                    json!({ "md": { "import": [{ "from": "@acme/kit@1", "names": "*" }] } }),
                ),
            },
            ..Host::default().with(
                "@acme/kit",
                release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
            )
        };
        let processed = host.process(":: card[Card]\n");
        assert_eq!(diagnostics(&processed), Vec::<String>::new());
        assert_eq!(shown(&processed), "A card.");
    }

    #[test]
    fn what_is_wrong_in_a_hosts_metadata_is_reported_at_line_1_since_it_isnt_in_the_text() {
        let host = Host {
            parse: ParseOptions {
                metadata: Some(json!({ "md": { "imports": [] } })),
            },
            ..Host::default()
        };
        let processed = host.process("Text.\n");
        let [unknown] = processed.diagnostics() else {
            panic!("one diagnostic")
        };
        assert_eq!(
            (
                unknown.code.to_string(),
                unknown.range.start.line,
                unknown.range.start.column
            ),
            ("W111".to_string(), 1, 1)
        );
        assert!(unknown.message.contains("did you mean `import`?"));
    }

    #[test]
    fn a_hosts_metadata_for_a_document_with_frontmatter_is_ignored_with_w115() {
        let host = Host {
            parse: ParseOptions {
                metadata: Some(json!({ "title": "From the host" })),
            },
            ..Host::default()
        };
        let processed = host.process("---\ntitle: Its own\n---\n\n{{md.doc.title}}\n");
        assert_eq!(codes(&processed), ["W115"]);
        assert_eq!(shown(&processed), "Its own");
    }

    #[test]
    fn a_name_from_an_ambient_pack_is_used_silently_unless_the_host_asks_for_w113() {
        let ambient = release("@host/c", "1.0.0", &card_pack("A card."), json!({}));
        let host = |report: bool| Host {
            ambient: Ambient {
                packs: vec!["@host/c".into()],
                report,
            },
            ..Host::default().with("@host/c", ambient.clone())
        };
        assert_eq!(
            codes(&host(false).process(":: card[Card]\n")),
            Vec::<String>::new()
        );
        assert_eq!(codes(&host(true).process(":: card[Card]\n")), ["W113"]);
    }
}
