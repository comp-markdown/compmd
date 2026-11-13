# 20. The Rust crates

The processor is a Rust crate, `compmd`; `@compmd/processor` is that crate compiled to WebAssembly. In Rust, it processes documents outside a browser: in a build, a server, a native app, or CI. This chapter shows its stages, what a host gives it, its diagnostics, and a program that checks a folder's documents and fails on an error. Then the crates beside it: `compmd-host`, what a host in Rust serves its readers, and the Rust crate `compmd-pack`, the names packs and their files are known by. [The crate's README](../../processor/README.md) and its API documentation go further.

## Getting it

The crate is [`compmd` on crates.io](https://crates.io/crates/compmd), with `serde_json` beside it for the trees as JSON:

```sh
cargo add compmd serde_json
```

Its parser, a fork of markdown-rs, comes with it as the crate `compmd-markdown`.

**Versions.** The crates share their version with the npm packages, and each release publishes them all ([RELEASING.md](../../RELEASING.md)): this chapter is written for 0.7.0, as the rest of the guide is. Its source is the folder `processor/` of [the repository](https://github.com/comp-markdown/compmd), and its API documentation is on [docs.rs](https://docs.rs/compmd).

## The stages

A `Processor` goes through the spec's [stages](https://compmd.hakanserce.com/spec/#11-processing-model) as states: each stage takes the processor and gives it back in its next state, so they run only in order, and each state gives what its stage made and the diagnostics so far. Each stage takes only what it uses. From the crate's documentation:

```rust
use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor};
use std::collections::HashMap;
use std::rc::Rc;

let text = "---\nteam: [Ana, Kenji]\n---\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n";
let files: HashMap<String, String> = HashMap::new();
let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
let processor = Processor::new("team.comp.md", text)
    .parse(&ParseOptions::default())
    .resolve(files, (), &Ambient::default())
    .validate(&[])
    .load(context, (), ())
    .expand(&Options::default());
assert!(processor.diagnostics().is_empty());
let json = serde_json::to_value(processor.expanded()).unwrap();
assert_eq!(json["children"][0]["children"][0]["value"], "Ana, Kenji");
```

| Call | Stage | Takes |
|---|---|---|
| `Processor::new(path, text)` | | the document's path under its root folder, and its text |
| `.parse(&ParseOptions)` | 1, parse | `ParseOptions { metadata }`: the document's metadata, when a host keeps it apart from the text |
| `.resolve(files, packs, &Ambient)` | 2, resolve | the root folder's files, the packs, and the ambient packs (`Ambient { packs, report }`, `report` for W113) |
| `.resolve_with(files, packs, &Ambient, &Policy)` | 2, resolve | the same, with the host's policy for packs |
| `.validate(&allowed_origins)` | 3, validate | the origins remote data may come from, `https://api.example.com` |
| `.load(context, fetch, data)` | 4a, load | the render context, remote data, `host:` data |
| `.expand(&expand::Options)` | 4b, evaluate | the options; `origins: true` puts an origin on every expanded node, and `mode` is `Mode::Lenient` (the default) or `Mode::Strict` |

`RenderContext` is the render: `now` in nanoseconds since 1970 (`compmd::context::parse_time` gives it from RFC 3339 text), the processor's time zone `tz`, the host's time zones `time`, and the values given for the document's `inputs`, as JSON by name.

Each state gives what its stage made: `tree()` after parsing (the parse tree, spec §12.1), `inputs()` after loading, and after expanding `expanded()` (the expanded tree), `host_components()` (each host-language component with its code, for a renderer), `failed()` (whether strict mode failed), `cache_keys()` (each data import's final URL or path, which a host keys its cached copies by), `frontmatter()`, `components()` and `hint_at(offset)` for editors. Every tree serializes, with serde, to the spec's JSON. From the expanded state, `reload(context, fetch, data)` renders again for other inputs or data without the first three stages, and `edit(&TextEdit)` takes an edit of the text and redoes only the blocks it touches, with `splice()` saying what changed. [The README](../../processor/README.md) lists them all.

## What the host gives

The processor does no I/O. What it reads comes through the traits of `compmd::host`, each stage taking only those it uses:

| Trait | Gives | In memory | None |
|---|---|---|---|
| `Files` | `file(path)`: a file under the document's root folder | `HashMap<String, String>` | `()` |
| `Packs` | `releases(pack)` and `pack_file(pack, version, path)` | `HashMap<String, Vec<LoadedRelease>>` | `()` |
| `Fetch` | `fetch(url)`: what the host fetched for a remote URL, a `Fetched { text, content_type }` | `HashMap<String, Fetched>` | `()`: every URL fails |
| `Data` | `data(source)`: a `host:` source's data, as JSON | `HashMap<String, serde_json::Value>` | `()` |
| `Time` | `offset(zone, at)`: a time zone's offset from UTC, in seconds | | `()`: only `UTC` |

The crate carries no time zone data; a host that gives `md.tz` other zones implements `Time` with its own. Each trait is also implemented for an `Rc` of it, so a host that keeps using its files gives the processor a clone. A host of its own implements a trait directly, as this pack store of one pack does, from the crate's documentation:

```rust
use compmd::host::{Packs, Release};
use semver::Version;

/// One pack, `@acme/kit@1.0.0`, with its manifest alone.
struct Kit;

impl Packs for Kit {
    fn releases(&self, pack: &str) -> Vec<Release> {
        let kit = Release { version: Version::new(1, 0, 0), integrity: None, broken: None };
        if pack == "@acme/kit" { vec![kit] } else { Vec::new() }
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        let found = pack == "@acme/kit" && *version == Version::new(1, 0, 0) && path == "package.json";
        found.then(|| r#"{"name": "@acme/kit", "version": "1.0.0"}"#.to_string())
    }
}
```

A `Release` is a version, the `integrity` its registry published (an import's pinned `integrity` must equal it, F035 otherwise), and `broken`, why its files can't be used. A `LoadedRelease` is a release with its files, and reads from the same JSON as `@compmd/processor`'s `PackRelease`, so a host can keep its packs in one format for both.

**The policy.** `compmd::policy::Policy` is the request's `policy`, and reads from the same JSON, which `policyOf` gives from a configuration ([chapter 18](18-other-hosts.md)). From the crate's documentation:

```rust
use compmd::policy::{Imports, Policy, Trust};

let policy: Policy = serde_json::from_str(r#"{
    "packs": { "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"] } },
    "imports": "listed",
    "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }],
    "refused": { "@huge/pack": "larger than this host serves" },
    "runtimes": [{ "name": "react", "version": "19.3.0" }, { "name": "svelte", "version": "5.57.2" }],
    "libraries": [{ "name": "d3", "version": "7.9.0" }]
}"#).unwrap();
assert_eq!(policy.imports, Imports::Listed);
assert_eq!(policy.trust("@acme/charts"), Trust::Strict);
```

**What to fetch first.** The processor fetches nothing. After validating, `sources(&context)` says what a render needs that the host hasn't given: the remote URLs, those of origins it doesn't allow (`disallowed`, for a host that asks its reader), the `host:` sources, and the packs no release matches (each `NeededPack` with its range and pinned integrity), for a host that fetches them, then loads; `compmd::host::newest_matching(range, versions)` picks a version as the processor will.

## Diagnostics

`processor.diagnostics()` gives every stage's so far, in stage order, each a `compmd::diagnostic::Diagnostic`:

- `code`, a `Code` (`Code::F004`, written `F004`), whose `severity()` is `Severity::Error` for an F code and `Severity::Warning` for a W code;
- `message`, for people;
- `file`, the document it is in when it isn't the processed one; `range`, its `start` and maybe `end` (`Location { line, column }`, from 1, columns in code points); `related` places; `stage`, a `Stage`; and `fix`, an edit, when the message suggests one.

A `Diagnostic` serializes to the spec's [machine form](https://compmd.hakanserce.com/spec/#105-machine-format), the JSON `@compmd/processor` gives. Every code is in [the diagnostics' appendix](22-the-diagnostics.md). Two things differ from WebAssembly: the crate takes host metadata (`ParseOptions::metadata`), so it reports W115, and its `Code` has no W104, which no package reports yet.

## Checking documents in CI

The spec's [strict mode](https://compmd.hakanserce.com/spec/#101-modes) fails on any F code: for CI and publishing. It reports the same diagnostics and makes the same tree as lenient mode; `failed()` says whether it failed. A program that checks every document among the files it is given, prints each diagnostic, and fails on an error:

```rust
use compmd::context::RenderContext;
use compmd::diagnostic::Mode;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor};
use std::collections::HashMap;
use std::process::ExitCode;
use std::rc::Rc;

/// The diagnostics of the document at `path`, each as `file:line:column: code message`, and whether it failed.
fn check(path: &str, files: &HashMap<String, String>) -> (Vec<String>, bool) {
    let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    let strict = Options { mode: Mode::Strict, ..Options::default() };
    let processor = Processor::new(path, files[path].as_str())
        .parse(&ParseOptions::default())
        .resolve(files.clone(), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&strict);
    let lines = processor
        .diagnostics()
        .iter()
        .map(|d| format!("{}:{}:{}: {} {}", d.file.as_deref().unwrap_or(path), d.range.start.line, d.range.start.column, d.code, d.message))
        .collect();
    (lines, processor.failed())
}

fn main() -> ExitCode {
    // Every file of the folder by its path: the documents, and the files they import and read.
    let files: HashMap<String, String> = std::env::args()
        .skip(1)
        .filter_map(|path| Some((path.clone(), std::fs::read_to_string(&path).ok()?)))
        .collect();
    let mut failed = false;
    for path in files.keys().filter(|path| path.ends_with(".comp.md")) {
        let (lines, failed_here) = check(path, &files);
        lines.iter().for_each(|line| eprintln!("{line}"));
        failed |= failed_here;
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
```

Run it from the documents' root folder with their paths as `git ls-files` prints them, relative to it, so imports and data files resolve: `cargo run --release -- $(git ls-files docs)`. The render time is fixed (1970, in UTC) so the check gives the same result every run; a document whose content depends on the date, through `md.today()`, gets a fixed one.

What it leaves to you:

- **Packs.** It gives none, so every pack import is F026. Give the packs your documents use as a `HashMap<String, Vec<LoadedRelease>>`, read from their folders or from JSON, and your app's policy with `resolve_with`.
- **Remote and `host:` data.** None is given, so a remote import loads its `fallback` or is F026, and a URL's origin is F028 unless you list it in `validate`. A check that should reach the network fetches what `sources` lists first.
- **Warnings** are printed and pass. Fail on them too, if your documents should have none.

The same check runs in Node with `@compmd/processor`'s `process` and `mode: "strict"`, which gives the same diagnostics and `failed`; the Vite plugin checks a listed pack's documents that way when an app builds ([chapter 21](21-the-configurations-reference.md#what-the-plugin-checks)).

## What the crate doesn't do

It implements stages 1 to 4b, and is a "compmd/1 processor with remote data and origins" in the spec's [conformance](https://compmd.hakanserce.com/spec/#131-what-conforms) terms. It doesn't run host-language components: it gives their code (`host_components`) to a renderer that does, such as `@compmd/sandbox` in a browser, and W118, found only by running one, is a renderer's. It doesn't analyse a component's code against its signature fence (W104). And it renders no HTML, but for what a plain CommonMark renderer shows, `compmd::plain::html`.

## A host in Rust: `compmd-host`

A host in Rust that shows documents serves its readers' browsers or webviews what [other hosts](18-other-hosts.md) serve from Node: the bundle, with the serving contract. [`compmd-host`](https://crates.io/crates/compmd-host) has it, for any platform, so an adapter for one (a desktop framework's plugin, a server's route) is a few lines:

```sh
cargo add compmd-host --features processor
```

- **`Bundle`** reads the bundle `@compmd/vite` or `buildBundle` wrote: from a folder (`Bundle::read`), embedded in a binary, or made in memory for a test (`Bundle::from_files`). Its `Manifest` is the `manifest.json` the build wrote; a manifest of a format the crate doesn't know is refused, saying what to upgrade.
- **`Store`** is where a host keeps packs it gets after the build, by their place, with their integrity. `FolderStore` keeps them in a folder, such as the app's data folder; `NoStore` holds none.
- **`respond(&bundle, &store, &request)`** is the serving contract, over the `http` crate's types: it answers `GET /frames/<hash>/<path>` of the bundle's hash and `GET /packs/<name>@<version>/<path>` of a pack the bundle or the store holds, with the contract's headers, and refuses anything else (the manifest too) with a `404` that says why, readable from any origin, or a `405` for another method.
- **`HostPacks`** (the `processor` feature) is `compmd::host::Packs` over the bundle and the store, the bundle's packs first, so the processor resolves a document's imports from exactly the files the frames load. From the crate's documentation:

```rust
let packs = HostPacks::new(Arc::new(bundle), Arc::new(NoStore));
let resolved = Processor::new("a.comp.md", "# A\n").parse(&ParseOptions::default()).resolve((), packs, &Ambient::default());
```

`tauri-plugin-compmd` is the adapter for Tauri, on `compmd-host` ([a Tauri app](19-a-tauri-app.md)).

## Names, checked once: the crate `compmd-pack`

The Rust crate [`compmd-pack`](https://crates.io/crates/compmd-pack) has the names packs and their files are known by (spec §6.4), each checked when it is made, so `compmd` and `compmd-host` mean one thing by each: `PackName` (`@scope/name`, as npm names scoped packages), `ReleaseId` (a pack at one version), `FilePath` (a file's path in a pack, which can't leave it), `Place` (a release and a file's path in it, `@kit/mermaid@1.2.0/dist/mermaid.js`) and `Integrity`. The processor re-exports it as `compmd::pack`, so a host rarely adds it itself. In JSON each is what the spec writes: a name, a path and an integrity are strings, and a place is `{"pack", "version", "path"}`. It isn't `compmd-pack`, the command of [`@compmd/pack`](17-testing-and-publishing.md) that builds and tests a pack, which only shares its name.

Next: the appendices, starting with [the configuration's reference](21-the-configurations-reference.md).
