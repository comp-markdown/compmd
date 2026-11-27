use std::time::{Duration, Instant};

use crossbeam_channel::unbounded;

use super::*;
use crate::links::document_links::LinkKind;

/// A folder of its own for the test `name`, with `files` (a path in it and its text).
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("compmd-lsp-index-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for (path, text) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a folder")).expect("its folder");
        fs::write(path, text).expect("written");
    }
    root
}

fn relative(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    let mut names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .expect("under the root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    names.sort();
    names
}

#[test]
fn the_markdown_files_skip_packages_hidden_folders_ignored_ones_and_large_ones() {
    let large = "x".repeat(usize::try_from(LARGEST_FILE).expect("a size") + 1);
    let root = project(
        "walk",
        &[
            ("notes.md", "# Notes\n"),
            ("posts/harbour.comp.md", "# Harbour\n"),
            ("posts/README.MD", "# Read me\n"),
            ("data.csv", "a,b\n"),
            ("node_modules/kit/README.md", "# Kit\n"),
            (".cache/old.md", "# Old\n"),
            (".gitignore", "drafts/\n"),
            ("drafts/later.md", "# Later\n"),
            ("large.md", &large),
        ],
    );
    let (paths, stopped) = markdown_files(&root, MOST_FILES);
    assert_eq!(
        relative(&root, &paths),
        ["notes.md", "posts/README.MD", "posts/harbour.comp.md"]
    );
    assert!(!stopped);
    let (first, stopped) = markdown_files(&root, 2);
    assert_eq!((first.len(), stopped), (2, true), "stopped at its limit");
}

#[test]
fn a_file_read_alone_gives_its_headings_and_links() {
    let file = IndexedFile::read(
        Path::new("/site/notes.md"),
        "# Harbour notes\n\nSee [tides](tides.md#high).\n",
        false,
    );
    assert_eq!(file.headings[0].anchor, "harbour-notes");
    assert_eq!(
        file.links[0].kind,
        LinkKind::Inline {
            url: "tides.md#high".into()
        }
    );
    let compmd = IndexedFile::read(
        Path::new("/site/notes.comp.md"),
        "::: box\n## Boxed\n::: /box\n",
        false,
    );
    assert_eq!(
        compmd.headings[0].anchor, "boxed",
        "a `.comp.md` file is CompMD"
    );
    let wiki = IndexedFile::read(Path::new("/site/notes.md"), "[[Tides]]\n", true);
    assert_eq!(
        wiki.links[0].kind,
        LinkKind::Wiki {
            target: "Tides".into()
        },
        "a `.md` file's wiki links when they are on"
    );
}

#[test]
fn the_index_reads_in_the_background_and_keeps_an_open_documents_entry() {
    let root = project("background", &[("a.md", "# A\n"), ("b.md", "# B\n")]);
    let index = ProjectIndex::new(root.clone());
    assert_eq!(index.root(), root);
    assert!(!index.is_read());
    let open = IndexedFile::read(&root.join("a.md"), "# Edited\n", false);
    index.set(root.join("a.md"), open.clone());
    let (log, _) = unbounded();
    index.read_in_background(log);
    let started = Instant::now();
    while !index.is_read() && started.elapsed() < Duration::from_secs(20) {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        index.files().get(&root.join("a.md")),
        Some(&open),
        "the open document's"
    );
    assert_eq!(index.files()[&root.join("b.md")].headings[0].anchor, "b");
    fs::write(root.join("b.md"), "# Bee\n").expect("written");
    index.read_again(&root.join("b.md"));
    assert_eq!(index.files()[&root.join("b.md")].headings[0].anchor, "bee");
    fs::remove_file(root.join("b.md")).expect("removed");
    index.read_again(&root.join("b.md"));
    assert!(!index.files().contains_key(&root.join("b.md")), "gone");
}

/// Waits until `done` holds of `index`, for at most 20 seconds.
fn wait_for(index: &ProjectIndex, done: impl Fn(&BTreeMap<PathBuf, IndexedFile>) -> bool) {
    let started = Instant::now();
    while !done(&index.files()) && started.elapsed() < Duration::from_secs(20) {
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn wiki_links_switched_on_in_markdown_files_read_them_again_but_the_open_ones() {
    let root = project("wiki", &[("a.md", "[[B]]\n"), ("b.md", "[[A]]\n")]);
    let index = ProjectIndex::new(root.clone());
    assert_eq!(index.wiki_links(), WikiLinks::default());
    let (log, _) = unbounded();
    index.read_in_background(log.clone());
    wait_for(&index, |files| files.len() == 2);
    assert_eq!(index.files()[&root.join("b.md")].links, [], "brackets");
    let on = WikiLinks {
        in_markdown_files: true,
        followed: true,
    };
    assert!(index.set_wiki_links(on), "parsed otherwise");
    assert!(
        !index.set_wiki_links(WikiLinks {
            followed: false,
            ..on
        }),
        "followed or not, parsed the same"
    );
    let open = OpenTexts::default();
    open.set(root.join("a.md"), "[[B]]\n".into());
    index.read_again_in_background(open, log);
    wait_for(&index, |files| !files[&root.join("b.md")].links.is_empty());
    assert!(index.wiki_links().in_markdown_files);
    assert!(
        index.files()[&root.join("a.md")].links.is_empty(),
        "the open file is its worker's"
    );
    assert_eq!(index.files()[&root.join("b.md")].links.len(), 1);
}
