//! Times an edit against processing the whole document, for the 52 KB body of the WebAssembly bench and the same
//! body ten times (about 500 KB), and prints p50 and p95 of each against the keystroke target (1 ms).
//!
//! Each edit class is an edit and the edit that undoes it, alternately, so the document stays the same:
//!
//! - `keystroke`: a character typed into a paragraph in the middle of the body;
//! - `blank line`: a paragraph in the middle split in two;
//! - `label`: the label of a link definition changed, which parses the whole body again;
//! - `frontmatter`: a character typed into the frontmatter, which runs every stage again.
//!
//! `cargo bench --bench edit`

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::host::{LoadedRelease, Release};
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor, TextEdit};
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

/// Criterion's linear sampling runs at least 5,050 measurement iterations after warm-up, so the last 1,000 recorded
/// iterations are all from the measurement phase.
const SAMPLES: usize = 1_000;

/// The target for a keystroke inside a block of ordinary size.
const KEYSTROKE: Duration = Duration::from_millis(1);

fn fixture(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../wasm/bench/fixtures/notes");
    std::fs::read_to_string(root.join(path)).expect("a fixture")
}

/// The pack the notes use, as the host's registry.
fn packs() -> HashMap<String, Vec<LoadedRelease>> {
    let files = ["index.comp.md", "package.json"]
        .into_iter()
        .map(|file| (file.to_string(), fixture(&format!("preview-pack/{file}"))))
        .collect();
    HashMap::from([(
        "@preview/charts".to_string(),
        vec![LoadedRelease {
            release: Release {
                version: "1.0.0".parse().expect("a version"),
                integrity: None,
            },
            files,
        }],
    )])
}

/// The documents: the notes, and their body ten times, each with a link definition at the end.
fn documents() -> [(&'static str, String); 2] {
    let notes = fixture("notes.comp.md");
    let (frontmatter, body) = notes.split_at(notes.find("\n# ").expect("a heading") + 1);
    let definition = "\n[ref]: https://example.com\n";
    [
        ("52 KB", format!("{notes}{definition}")),
        (
            "500 KB",
            format!("{frontmatter}{}{definition}", body.repeat(10)),
        ),
    ]
}

/// An edit and the edit that undoes it, with the target it has: only an edit inside a block has one.
struct Class {
    name: &'static str,
    edit: TextEdit,
    undo: TextEdit,
    target: Option<Duration>,
}

fn insertion(name: &'static str, target: Option<Duration>, at: usize, text: &str) -> Class {
    Class {
        name,
        target,
        edit: TextEdit {
            start: at,
            end: at,
            text: text.into(),
        },
        undo: TextEdit {
            start: at,
            end: at + text.chars().count(),
            text: String::new(),
        },
    }
}

/// The edit classes for `text`.
fn classes(text: &str) -> [Class; 4] {
    let chars: Vec<char> = text.chars().collect();
    let middle = chars.len() / 2;
    let sentence_end = middle
        + chars[middle..]
            .iter()
            .position(|c| *c == '.')
            .expect("a sentence");
    let label = chars.len() - "ref]: https://example.com\n".chars().count();
    [
        insertion("keystroke", Some(KEYSTROKE), sentence_end, "x"),
        insertion("blank line", Some(KEYSTROKE), sentence_end + 1, "\n\n"),
        insertion("label", None, label, "x"),
        insertion(
            "frontmatter",
            None,
            "---\nmd:\n  version: 1".chars().count(),
            "\n  title: x",
        ),
    ]
}

/// Benchmarks `op` and prints its percentiles, against `target` when it has one. A `flat` benchmark is slow: it
/// takes few samples, and its percentiles come from all of them.
fn measured(
    c: &mut Criterion,
    group: &str,
    name: &str,
    flat: bool,
    target: Option<Duration>,
    mut op: impl FnMut(),
) {
    let mut samples = Vec::new();
    let mut group = c.benchmark_group(group);
    if flat {
        group.sampling_mode(SamplingMode::Flat).sample_size(10);
    }
    group.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let started = Instant::now();
                op();
                let elapsed = started.elapsed();
                samples.push(elapsed);
                total += elapsed;
            }
            total
        });
    });
    group.finish();
    if samples.is_empty() {
        // A filter on the command line left it out.
        return;
    }
    let recent = if flat {
        &samples[..]
    } else {
        &samples[samples.len().saturating_sub(SAMPLES)..]
    };
    let mut sorted = recent.to_vec();
    sorted.sort();
    let at = |share: usize| sorted[(sorted.len() - 1) * share / 100];
    let (p50, p95) = (at(50), at(95));
    println!(
        "{name}: p50 {:.3} ms, p95 {:.3} ms{}",
        p50.as_secs_f64() * 1000.0,
        p95.as_secs_f64() * 1000.0,
        target.map_or_else(String::new, |target| format!(
            ", target {} ms: {}",
            target.as_millis(),
            if p95 <= target { "met" } else { "missed" }
        ))
    );
}

fn edits(c: &mut Criterion) {
    let packs = packs();
    let ambient = Ambient {
        packs: vec!["@preview/charts".into()],
        report: false,
    };
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: &(),
        inputs: serde_json::Map::default(),
    };
    for (size, text) in documents() {
        let files = HashMap::from([("notes.comp.md".to_string(), text.clone())]);
        let process = || {
            Processor::new("notes.comp.md", text.clone())
                .parse(&ParseOptions::default())
                .resolve(&files, &packs, &ambient)
                .validate(&[])
                .load(&context, &(), &())
                .expand(&Options::default())
        };
        measured(c, "whole", size, true, None, || {
            black_box(process());
        });
        for class in classes(&text) {
            let mut processor: Option<Processor<Expanded>> = Some(process());
            let mut undo = false;
            measured(
                c,
                &format!("edit {size}"),
                class.name,
                false,
                class.target,
                || {
                    let edit = if undo { &class.undo } else { &class.edit };
                    undo = !undo;
                    processor = processor.take().map(|processor| processor.edit(edit));
                },
            );
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = edits
}
criterion_main!(benches);
