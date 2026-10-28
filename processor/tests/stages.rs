//! Stages 2 and 3 beyond the conformance set: component files, packs and their files, re-exports, and the values
//! params take, each with the diagnostics it gives, as `(code, line, column, file)`.

mod common;

use common::at;
use compmd::host::{LoadedRelease, Release};
use compmd::{Ambient, ParseOptions, Processor};
use semver::Version;
use std::collections::HashMap;

/// Packs held in memory, by name.
type Packs = HashMap<String, Vec<LoadedRelease>>;

/// Adds a release to packs.
trait With {
    fn with(self, pack: &str, version: &str, files: &[(&str, &str)]) -> Self;
}

impl With for Packs {
    fn with(mut self, pack: &str, version: &str, files: &[(&str, &str)]) -> Self {
        self.entry(pack.to_string())
            .or_default()
            .push(LoadedRelease {
                release: Release {
                    version: Version::parse(version).expect("a version"),
                    integrity: None,
                },
                files: files
                    .iter()
                    .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
                    .collect(),
            });
        self
    }
}

type Found = Vec<(String, usize, usize, Option<String>)>;

/// Resolves and validates `input.comp.md` among `files`, with `packs` and the `ambient` packs.
fn run(files: &[(&str, &str)], packs: &Packs, ambient: &[&str]) -> Found {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    let text = files["input.comp.md"].clone();
    let ambient = Ambient {
        packs: ambient.iter().map(|pack| (*pack).to_string()).collect(),
        report: false,
    };
    let validated = Processor::new("input.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(&files, packs, &ambient)
        .validate(&[]);
    let mut found: Found = validated
        .diagnostics()
        .iter()
        .map(|d| {
            (
                d.code.to_string(),
                d.range.start.line,
                d.range.start.column,
                d.file.clone(),
            )
        })
        .collect();
    found.sort();
    found
}

fn codes(text: &str) -> Vec<(String, usize, usize)> {
    run(&[("input.comp.md", text)], &Packs::default(), &[])
        .into_iter()
        .map(|(code, line, column, _)| (code, line, column))
        .collect()
}

#[test]
fn a_component_file_exports_one_component_named_after_its_stem() {
    let input = "---\nmd:\n  import:\n    - from: ./charts/Chart.jsx\n      names: [Chart, {name: Chart, as: plot}, Other]\n    - from: ./charts/Chart.jsx\n      names: \"*\"\n---\n\n```compmd component=plot signature\nparams:\n  rows: array\n```\n\n:: plot[A plot] {rows=[1]}\n\n:: Chart[A chart]\n";
    let found = run(
        &[
            ("input.comp.md", input),
            ("charts/Chart.jsx", "export default () => null;"),
        ],
        &Packs::default(),
        &[],
    );
    assert_eq!(
        found,
        [("F013".into(), 7, 14, None), ("F021".into(), 5, 47, None),]
    );
}

#[test]
fn imports_that_load_nothing_drop_their_names() {
    let input = "---\nmd:\n  import:\n    - from: ./missing.jsx\n      names: [x]\n    - from: ../outside.jsx\n      names: [y]\n    - from: ../outside.comp.md\n      names: \"*\"\n    - from: \"@acme/bare\"\n      names: [z]\n---\n\n:: x[X]\n\n:: y[Y]\n\n:: z[Z]\n\n:: anything[Any]\n";
    let packs = Packs::default().with("@acme/bare", "1.0.0", &[]);
    let found = run(&[("input.comp.md", input)], &packs, &[]);
    assert_eq!(
        found,
        [
            ("F026".into(), 4, 13, None),
            ("F026".into(), 6, 13, None),
            ("F026".into(), 8, 13, None),
            ("F026".into(), 10, 13, None),
        ]
    );
}

#[test]
fn packs_find_their_document_and_their_own_files() {
    let kit = [
        (
            "package.json",
            "{\"name\": \"@acme/kit\", \"version\": \"2.0.0\"}",
        ),
        (
            "index.comp.md",
            "---\nmd:\n  import:\n    - from: ./parts.comp.md\n      names: [part]\n  export:\n    names: [part, {name: part, as: piece}, {name: part, as: box}, gone]\n---\n",
        ),
        (
            "parts.comp.md",
            "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=part\n---\nPart.\n```\n",
        ),
    ];
    let packs = Packs::default().with("@acme/kit", "2.0.0", &kit);
    let input = "---\nmd:\n  import:\n    - from: \"@acme/kit@2\"\n      names: [part, piece]\n    - from: \"@acme/kit\"\n      names: \"*\"\n---\n\n:: part[P]\n\n:: piece[P]\n";
    let found = run(&[("input.comp.md", input)], &packs, &[]);
    assert_eq!(
        found,
        [(
            "F013".into(),
            7,
            51,
            Some("@acme/kit@2.0.0/index.comp.md".into())
        )]
    );
}

#[test]
fn re_exports_pass_on_what_they_have() {
    let ui = "---\nmd:\n  import:\n    - from: ./nowhere.comp.md\n      names: [lost]\n  export:\n    names: [lost]\n---\n";
    let input =
        "---\nmd:\n  import:\n    - from: ./ui.comp.md\n      names: [lost]\n---\n\n:: lost[L]\n";
    let found = run(
        &[("input.comp.md", input), ("ui.comp.md", ui)],
        &Packs::default(),
        &[],
    );
    assert_eq!(found, [("F026".into(), 4, 13, Some("ui.comp.md".into()))]);
}

#[test]
fn host_components_are_pinned_and_typed_where_they_are_exported() {
    let input = "---\nmd:\n  import:\n    - from: ./ui.comp.md\n      names: [dial]\n    - from: ./Gauge.svelte\n      names: [Gauge]\n  export:\n    names: [dial, clock, plain, Gauge]\n---\n\n```jsx component=clock runtime=react@19\nexport default () => null;\n```\n\n```compmd component=clock signature\nparams: {}\n```\n\n```ts component=plain\nexport default function mount() {}\n```\n\n```compmd component=plain signature\nparams: {}\n```\n";
    let ui = "---\nmd:\n  export:\n    names: [dial]\n---\n\n```svelte component=dial\n<b />\n```\n\n```compmd component=dial signature\nparams: {}\n```\n";
    let found = run(
        &[
            ("input.comp.md", input),
            ("ui.comp.md", ui),
            ("Gauge.svelte", "<b />"),
        ],
        &Packs::default(),
        &[],
    );
    assert_eq!(
        found,
        [
            ("F029".into(), 9, 33, None),
            ("W117".into(), 7, 1, Some("ui.comp.md".into())),
        ]
    );
}

#[test]
fn ambient_packs_are_the_newest_release_and_lowest_priority() {
    let old = [
        (
            "package.json",
            "{\"name\": \"@host/c\", \"version\": \"1.0.0\"}",
        ),
        (
            "index.comp.md",
            "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=old\n---\nOld.\n```\n",
        ),
    ];
    let new = [
        (
            "package.json",
            "{\"name\": \"@host/c\", \"version\": \"1.1.0\"}",
        ),
        (
            "index.comp.md",
            "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=new\n---\nNew.\n```\n\n```compmd component=tag signature\nparams: {}\n```\n",
        ),
    ];
    let packs = Packs::default()
        .with("@host/c", "1.0.0", &old)
        .with("@host/c", "1.1.0", &new);
    let input = ":: new[N]\n\n:: tag[T]\n\n:: old[O]\n";
    let found = run(
        &[("input.comp.md", input)],
        &packs,
        &["@host/c", "@host/none"],
    );
    assert_eq!(found, [("F002".into(), 5, 4, None)]);
}

#[test]
fn definitions_that_share_a_component_are_no_cycle() {
    assert_eq!(
        codes(
            "```compmd component=a\n---\n:: c[C]\n\n:: b[B]\n```\n\n```compmd component=b\n---\n:: c[C]\n```\n\n```compmd component=c\n---\nC.\n```\n\n:: a[A]\n"
        ),
        []
    );
}

#[test]
fn values_are_read_by_their_declared_type() {
    let card = "```compmd component=card\nparams:\n  n: number?\n  on: boolean?\n  day: date?\n  rows: array?\n  meta: object?\n  t: name?\n  tone: info | warn = info\n  color: color?\n  title: string?\n---\nCard.\n```\n\n";
    let good = format!(
        "{card}:: card[C] {{n=2.5 on day=2026-10-21 rows=[1] meta={{\"a\":1}} t=row tone=\"warn\" color=\"#fff\" title=x}}\n\n:: card[C] {{on=false day=\"2026-01-31\" title=\"{{{{md.doc.x}}}}\" n=(1 + 2) tone=(\"i\" + \"nfo\")}}\n"
    );
    assert_eq!(codes(&good), [at("F014", 18, 48)]);
    let bad = format!(
        "{card}:: card[C] {{n=\"2\" on=yes day=2026-02-30 rows={{\"a\":1}} meta=[1] t=a-b tone=[1] color=red-ish title}}\n\n:: card[C] {{t=(x) n=(\"a\")}}\n"
    );
    assert_eq!(
        codes(&bad),
        [
            at("F009", 16, 15),
            at("F009", 16, 22),
            at("F009", 16, 30),
            at("F009", 16, 46),
            at("F009", 16, 59),
            at("F009", 16, 65),
            at("F009", 16, 74),
            at("F009", 16, 84),
            at("F009", 16, 92),
            at("F009", 18, 15),
            at("F024", 18, 22),
        ]
    );
}

#[test]
fn universal_params_take_their_grammar() {
    assert_eq!(
        codes(
            ":: chart[C] {scroll=true fill=maybe class=(\"a\" + 1) lang=\"de\" #x .y dir=sideways padding=[1] md:lang=de border-radius=(1)}\n\n```jsx component=chart\nexport default () => null;\n```\n"
        ),
        [
            at("F008", 1, 94),
            at("F009", 1, 31),
            at("F009", 1, 73),
            at("F009", 1, 90),
            at("F024", 1, 44),
            at("F024", 1, 120),
            at("W105", 3, 1),
        ]
    );
}

#[test]
fn bound_inputs_are_names() {
    assert_eq!(
        codes(
            "---\nmd:\n  inputs:\n    q: string = \"\"\n---\n\n:: text-input[Q] {bind=(q)}\n\n:: text-input[Q] {bind=\"q\"}\n\n```compmd component=c\n---\n:: text-input[Q] {bind=q}\n```\n"
        ),
        [at("F009", 7, 24), at("F009", 9, 24), at("F030", 13, 24)]
    );
}

#[test]
fn components_without_a_signature_check_only_expressions_and_paths() {
    assert_eq!(
        codes(
            "::: gone {a=(1 +) b=\"{{nope}}\"}\nText.\n::: /gone\n\n```jsx component=chart\nexport default () => null;\n```\n\n:: chart[C] {any=(md.doc.x) more=\"{{md.version}}\"}\n"
        ),
        [
            at("F002", 1, 5),
            at("F014", 1, 24),
            at("F024", 1, 17),
            at("F024", 9, 26),
            at("W105", 5, 1)
        ]
    );
}

#[test]
fn fences_take_box_params_on_compmd_only() {
    assert_eq!(
        codes(
            "```compmd component=card {gap=s display=flex width=50% nothing=1 md:gap=huge .c #i}\n---\nCard.\n```\n\n```compmd component=flat {display=contents .c}\n---\nFlat.\n```\n\n:: card[C]\n\n:: flat[F] {lang=de}\n"
        ),
        [at("F008", 1, 56), at("F009", 1, 73), at("W120", 6, 44)]
    );
}

/// `display=contents` at a breakpoint is F009 and takes nothing away: the component keeps its box, for the fence's
/// other params and the caller's.
#[test]
fn display_contents_at_a_breakpoint_keeps_the_box() {
    assert_eq!(
        codes(
            "```compmd component=late {md:display=contents padding=s}\n---\nLate.\n```\n\n:: late[L] {padding=m}\n"
        ),
        [at("F009", 1, 38)]
    );
}

#[test]
fn interpolations_are_checked_in_every_inline_and_block() {
    let text = "*{{a}}* **{{b}}** ~~{{c}}~~ [x]({{d}}) [{{e}}][r] ![i]({{md.doc.f}})\n\n| H |\n| - |\n| {{g}} |\n\n> {{h}}\n\n- {{i}}\n\nNote.[^n]\n\n[^n]: {{j}}\n\n[r]: https://example.com/{{k}}\n\n# {{l}} {{ md.version }}\n";
    let found: Vec<usize> = codes(text)
        .into_iter()
        .map(|(_, line, column)| line * 100 + column)
        .collect();
    assert_eq!(
        found,
        [
            104, 113, 123, 135, 143, 158, 505, 705, 905, 1309, 1528, 1705
        ]
    );
}

#[test]
fn templates_bind_their_element_and_md_loop() {
    assert_eq!(
        codes(
            "---\nrows: [{a: 1}]\n---\n\n::: each {in=(md.doc.rows) as=r join=list}\n- {{r.a}} {{md.loop.first}} {{md.loop.nope}} {{md.loop.index.x}}\n\n:: #before\n{{md.loop}}\n:: #nothing\nx\n::: /each\n\n:: value[V] {of=(string(md.doc.rows[0].a))}\n"
        ),
        [
            at("F006", 10, 1),
            at("F014", 6, 31),
            at("F014", 6, 48),
            at("F014", 9, 3)
        ]
    );
}

#[test]
fn definitions_see_their_params_and_slots_but_not_inputs() {
    assert_eq!(
        codes(
            "---\nmd:\n  inputs:\n    q: string = \"\"\n---\n\n```compmd component=c\nparams:\n  max-width: length?\n  as: name?\nslots: {default: {}, side: {}}\n---\n{{max_width}} {{as}} {{md.slot.side}} {{md.slot}} {{md.input}} {{md.input.q}}\n```\n\n{{md.input}} {{md.data}} {{md.doc}} {{md}}\n"
        ),
        [at("F014", 13, 17), at("F014", 13, 53), at("F014", 13, 66)]
    );
}

#[test]
fn expressions_see_inputs_data_loops_and_params_by_type() {
    let text = "---\nmd:\n  inputs:\n    rate: number = 1\n  data:\n    - from: ./sales.csv\n      as: sales\n      schema: {amount: number}\n    - from: ./team.yaml\n      as: team\n---\n\n::: each {in=(md.data.sales) as=s}\n:: value[V] {of=(string(s.amount * md.input.rate + double(md.loop.index)))}\n\n:: value[V] {of=(s.nope)}\n::: /each\n\n::: each {in=(md.data.team) as=p}\n:: value[V] {of=(p.anything)}\n::: /each\n\n```compmd component=c\nparams:\n  n: number\n---\n:: value[V] {of=(string(n + 1))}\n\n:: value[V] {of=(md.input.rate)}\n```\n";
    assert_eq!(codes(text), [at("F024", 16, 20), at("F024", 29, 21)]);
}

#[test]
fn every_common_param_has_its_values() {
    assert_eq!(
        codes(
            "::: box {padding-inline-start=s order=first align-self=center justify-self=end sticky=bottom show=sm hide=lg aspect-ratio=none grid-area=side height=auto}\nText.\n::: /box\n\n::: box {order=middle align-self=left}\nText.\n::: /box\n"
        ),
        [at("F009", 5, 16), at("F009", 5, 34)]
    );
}

#[test]
fn a_pack_without_exports_exports_nothing() {
    let kit = [
        (
            "package.json",
            "{\"name\": \"@acme/kit\", \"version\": \"1.0.0\"}",
        ),
        ("index.comp.md", "Nothing here.\n"),
    ];
    let packs = Packs::default().with("@acme/kit", "1.0.0", &kit);
    let input = "---\nmd:\n  import:\n    - from: \"@acme/kit\"\n      names: [chip]\n---\n";
    let found = run(&[("input.comp.md", input)], &packs, &[]);
    assert_eq!(found, [("F021".into(), 5, 15, None)]);
}

#[test]
fn an_empty_registry_has_no_packs() {
    let files: HashMap<String, String> = HashMap::new();
    let ambient = Ambient {
        packs: vec!["@host/c".into()],
        report: true,
    };
    let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: [chip]\n---\n";
    let resolved = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(&files, &(), &ambient);
    let codes: Vec<String> = resolved
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(codes, ["F026"]);
}

#[test]
fn breakpoints_are_for_layout_params() {
    assert_eq!(
        codes(
            "```jsx component=chart runtime=react@19\nexport default () => null;\n```\n\n```compmd component=chart signature\nparams:\n  size: number?\n```\n\n:: chart[C] {md:size=2 sm:width=50%}\n\n:: if[I] {md:when=(true)}\n"
        ),
        [at("F008", 10, 14), at("F008", 12, 11)]
    );
}
