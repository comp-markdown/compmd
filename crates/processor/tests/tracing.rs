//! The processor's `tracing` spans, as a host's subscriber sees them: one for each stage, named by its `stage` field,
//! with an event of the diagnostics it found. The processor installs no subscriber; this test's records what it gets.

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor};
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Metadata, Subscriber};

/// A subscriber that writes down each span's `stage` and each event's `diagnostics`.
#[derive(Clone, Default)]
struct Recording(Arc<Mutex<Vec<String>>>);

/// What a span or an event says, as `key=value`.
struct Fields<'a>(&'a mut Vec<String>);

impl Visit for Fields<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() != "message" {
            self.0.push(format!("{}={value:?}", field.name()));
        }
    }
}

impl Recording {
    fn push(&self, line: String) {
        self.0.lock().expect("not poisoned").push(line);
    }
}

impl Subscriber for Recording {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        let mut fields = Vec::new();
        span.record(&mut Fields(&mut fields));
        self.push(format!(
            "span {} {}",
            span.metadata().name(),
            fields.join(" ")
        ));
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut fields = Vec::new();
        event.record(&mut Fields(&mut fields));
        self.push(format!("event {}", fields.join(" ")));
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

#[test]
fn each_stage_is_a_span_with_the_diagnostics_it_found() {
    let recording = Recording::default();
    tracing::subscriber::with_default(recording.clone(), || {
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::new(),
        };
        let _expanded = Processor::new("a.comp.md", "Hello {{nobody}}.\n")
            .parse(&ParseOptions::default())
            .resolve((), (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&Options::default());
    });
    let lines = recording.0.lock().expect("not poisoned").clone();
    let stages: Vec<&str> = lines
        .iter()
        .filter_map(|line| line.strip_prefix("span compmd stage="))
        .collect();
    assert_eq!(
        stages,
        [
            "\"parse\"",
            "\"resolve\"",
            "\"validate\"",
            "\"load\"",
            "\"expand\""
        ]
    );
    // `{{nobody}}` is F014, found while validating.
    let events: Vec<&String> = lines
        .iter()
        .filter(|line| line.starts_with("event "))
        .collect();
    assert_eq!(
        events,
        [
            "event diagnostics=0",
            "event diagnostics=0",
            "event diagnostics=1",
            "event diagnostics=0",
            "event diagnostics=0"
        ]
    );
}
