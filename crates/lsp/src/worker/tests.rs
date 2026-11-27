use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crossbeam_channel::unbounded;
use lsp_types::{NumberOrString, Position, Range};

use super::*;

/// A new, empty folder for one test's project.
fn project() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "compmd-lsp-worker-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a folder");
    dir
}

/// A worker for `text`, the document `a.comp.md` in `root`: its job queue and what it posts.
fn worker(root: &Path, text: &str) -> (Sender<Job>, Receiver<Output>) {
    let path = root.join("a.comp.md");
    let uri: Uri = url::Url::from_file_path(&path)
        .expect("a file URL")
        .as_str()
        .parse()
        .expect("a URI");
    let document = Document {
        uri,
        path,
        root: root.to_owned(),
        text: text.to_owned(),
        version: 1,
    };
    let (jobs, received) = unbounded();
    let (output, posted) = unbounded();
    let (sender, registry) = (
        jobs.clone(),
        RegistryPacks::start(root.join("cache"), "http://127.0.0.1:9/"),
    );
    thread::spawn(move || {
        let queue = Queue {
            jobs: &received,
            sender,
        };
        run(
            document,
            Encoding::Utf16,
            OpenTexts::default(),
            queue,
            output,
            registry,
            || None,
        );
    });
    (jobs, posted)
}

/// The codes of the next diagnostics posted, and their version.
fn codes(posted: &Receiver<Output>) -> (Option<i32>, Vec<String>) {
    loop {
        match posted.recv().expect("posted") {
            Output::Diagnostics {
                version,
                diagnostics,
                ..
            } => {
                let codes = diagnostics
                    .into_iter()
                    .map(|diagnostic| match diagnostic.code {
                        Some(NumberOrString::String(code)) => code,
                        _ => diagnostic.message,
                    })
                    .collect();
                return (version, codes);
            }
            Output::Log(_)
            | Output::Response(_)
            | Output::Question { .. }
            | Output::Preview { .. }
            | Output::DrawingMoved
            | Output::ValuesChanged => {}
        }
    }
}

/// A change of the line `line` from `from` to `to` (UTF-16 columns) into `text`.
fn change(line: u32, from: u32, to: u32, text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position {
                line,
                character: from,
            },
            end: Position {
                line,
                character: to,
            },
        }),
        range_length: None,
        text: text.into(),
    }
}

#[test]
fn a_document_is_processed_when_it_opens_and_at_each_edit() {
    let root = project();
    let (jobs, posted) = worker(&root, "# Notes\n\n:: nowhere[Here]\n");
    assert_eq!(codes(&posted), (Some(1), vec!["F002".to_owned()]));
    jobs.send(Job::Change {
        version: 2,
        changes: vec![change(2, 3, 10, "box")],
    })
    .expect("sent");
    assert_eq!(
        codes(&posted),
        (Some(2), Vec::new()),
        "`:: box[Here]` is core's"
    );
    jobs.send(Job::Change {
        version: 3,
        changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: ":: gone[X]\n".into(),
        }],
    })
    .expect("sent");
    assert_eq!(
        codes(&posted),
        (Some(3), vec!["F002".to_owned()]),
        "the whole text"
    );
    jobs.send(Job::Close).expect("sent");
    assert_eq!(codes(&posted), (None, Vec::new()), "closing clears them");
}

#[test]
fn the_projects_configuration_and_packs_hold_and_a_reload_reads_them_again() {
    let root = project();
    fs::write(root.join("compmd.config.json"), r#"{"fetch": "proxy"}"#).expect("written");
    let (jobs, posted) = worker(&root, "# Notes\n");
    let (_, wrong) = codes(&posted);
    assert_eq!(
        wrong,
        ["compmd.config.json:\n- fetch: `host` or `direct`"],
        "a configuration that's wrong is a diagnostic"
    );
    jobs.send(Job::Change {
        version: 2,
        changes: vec![change(0, 2, 7, "Ideas")],
    })
    .expect("sent");
    assert_eq!(
        codes(&posted).1.len(),
        1,
        "an edit with no processor processes the whole text, and finds it again"
    );
    fs::create_dir_all(root.join("components")).expect("a folder");
    fs::write(
        root.join("components/package.json"),
        r#"{"name": "@site/components", "version": "1.0.0"}"#,
    )
    .expect("written");
    fs::write(root.join("components/index.comp.md"), "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n")
        .expect("written");
    fs::write(
        root.join("compmd.config.json"),
        r#"{"packs": {"./components": {"ambient": true}}}"#,
    )
    .expect("written");
    jobs.send(Job::Change {
        version: 3,
        changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: ":: note[N]\n".into(),
        }],
    })
    .expect("sent");
    assert_eq!(
        codes(&posted),
        (Some(3), Vec::new()),
        "the ambient folder pack's `note`"
    );
    fs::write(root.join("compmd.config.json"), "{}").expect("written");
    jobs.send(Job::Reload).expect("sent");
    assert_eq!(
        codes(&posted),
        (Some(3), vec!["F002".to_owned()]),
        "without the pack again"
    );
    jobs.send(Job::Close).expect("sent");
}

#[test]
fn a_document_is_named_by_its_path_in_its_root() {
    assert_eq!(
        relative(Path::new("/site/posts/a.comp.md"), Path::new("/site")),
        "posts/a.comp.md"
    );
    assert_eq!(
        relative(Path::new("/elsewhere/a.comp.md"), Path::new("/site")),
        "a.comp.md"
    );
    assert_eq!(relative(Path::new("/"), Path::new("/site")), "");
    assert!(now() > 0);
}
