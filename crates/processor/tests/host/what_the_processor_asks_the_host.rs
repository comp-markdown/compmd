//! What the processor asks of a host's own `Fetch`, `Data` and `Files`, and what it never asks.

use super::*;
use compmd::host::{Fetched, Files};
use std::cell::RefCell;

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
fn data_the_host_is_still_getting_is_pending_without_f026_and_said_so() {
    let pending = |sources: &[&str]| WithPending {
        held: (),
        pending: sources.iter().map(|source| (*source).to_string()).collect(),
    };
    let host = Host {
        allowed_origins: vec!["https://a.org".into()],
        remote: Rc::new(pending(&["https://a.org/x.json"])),
        data: Rc::new(pending(&["tasks"])),
        ..Host::default()
    };
    let processed = host.process(
        "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: sales\n    - from: host:tasks\n      as: tasks\n    - from: host:notes\n      as: notes\n---\n",
    );
    assert_eq!(
        codes(&processed),
        ["F026"],
        "only `host:notes`, which isn't pending"
    );
    let said = processed.pending();
    assert_eq!(said.remote, ["https://a.org/x.json"]);
    assert_eq!(said.host, ["tasks"]);
}

#[test]
fn a_hosts_data_is_asked_for_by_its_source_without_host() {
    let recording = Rc::new(Recording::default());
    let host = Host {
        data: Rc::clone(&recording) as Rc<dyn Data>,
        ..Host::default()
    };
    let processed = host.process(
        "---\nmd:\n  data:\n    - from: host:tasks?status=open\n      as: tasks\n---\n\n{{md.data.tasks.n}}\n",
    );
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

impl Files for Recording {
    fn file(&self, path: &str) -> Option<String> {
        self.0.borrow_mut().push(path.to_string());
        Some("n\n1\n".into())
    }
}

/// `text`, as the document at `path`, through every stage, with files that record what they are asked for.
fn processed_at(path: &str, text: &str) -> (Processor<Expanded>, Vec<String>) {
    let files = Rc::new(Recording::default());
    let processed = Processor::new(path, text)
        .parse(&ParseOptions::default())
        .resolve(Rc::clone(&files), (), &Ambient::default())
        .validate(&[])
        .load(Host::default().context, Rc::new(()), Rc::new(()))
        .expand(&Options::default());
    let asked = files.0.borrow().clone();
    (processed, asked)
}

#[test]
fn a_path_with_a_windows_name_is_f026_and_never_asked_for() {
    let text = "---\nmd:\n  import:\n    - from: \"./..\\\\..\\\\secrets.comp.md\"\n      names: \"*\"\n    - from: \"./c:/x.jsx\"\n      names: \"*\"\n  data:\n    - from: \"./a\\u0000.csv\"\n      as: nul\n    - from: ./ok.csv\n      as: ok\n---\n";
    let (processed, asked) = processed_at("docs/a.comp.md", text);
    assert_eq!(asked, ["docs/ok.csv"], "only the plain path is read");
    assert_eq!(
        diagnostics(&processed),
        [
            "F026 `./..\\..\\secrets.comp.md` isn't a file's path: a name has `\\`, `:` or NUL",
            "F026 `./c:/x.jsx` isn't a file's path: a name has `\\`, `:` or NUL",
            "F026 `./a\0.csv` isn't a file's path: a name has `\\`, `:` or NUL; `md.data.nul` is null",
        ]
    );
}

#[test]
fn a_document_whose_path_isn_t_one_in_its_root_reads_no_relative_file() {
    let text = "---\nmd:\n  data:\n    - from: ./ok.csv\n      as: ok\n---\n\nStill shown.\n";
    for path in ["./a.comp.md", "/docs/a.comp.md", "C:\\docs\\a.comp.md", ""] {
        let (processed, asked) = processed_at(path, text);
        assert_eq!(asked, Vec::<String>::new(), "{path:?}");
        assert_eq!(
            diagnostics(&processed),
            [format!(
                "F026 `./ok.csv` can't be resolved: the document's path, `{path}`, isn't a `/`-separated path in its root; `md.data.ok` is null"
            )],
            "{path:?}"
        );
        assert_eq!(shown(&processed), "Still shown.");
    }
}
