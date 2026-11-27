use std::fs;
use std::path::Path;

use compmd::diagnostic::{Location as Place, Range as CompmdRange};

use super::*;
use crate::encoding::Encoding;
use crate::places::Places;

fn uri_of(path: &Path) -> Uri {
    url::Url::from_file_path(path)
        .expect("a file URL")
        .as_str()
        .parse()
        .expect("a URI")
}

/// `defined` asked from the document `a.comp.md` in `root`, whose text is `text`.
fn located(defined: &Defined, root: &Path, text: &str) -> Option<Location> {
    let (uri, places) = (uri_of(&root.join("a.comp.md")), Places::new(root, []));
    let asked = Asked {
        processor: None,
        uri: &uri,
        plain_markdown_file: false,
        path: &root.join("a.comp.md"),
        index: None,
        links: true,
        text,
        version: 1,
        encoding: Encoding::Utf16,
        places: &places,
        pack_components: &[],
        unregistered: None,
    };
    location(defined, &asked)
}

#[test]
fn a_definition_here_in_a_file_or_in_a_pack_that_isnt_on_the_disk() {
    let root = std::env::temp_dir().join(format!("compmd-lsp-definition-{}", std::process::id()));
    fs::create_dir_all(&root).expect("a folder");
    fs::write(root.join("notes.comp.md"), "Été\n:: note\n").expect("written");
    let on = |line, column| CompmdRange {
        start: Place::new(line, column),
        end: Some(Place::new(line, column + 1)),
    };
    let here = located(
        &Defined {
            file: None,
            range: Some(on(1, 2)),
        },
        &root,
        "ab\n",
    )
    .expect("here");
    assert_eq!(
        (here.uri, here.range.start),
        (
            uri_of(&root.join("a.comp.md")),
            Position {
                line: 0,
                character: 1
            }
        )
    );
    let there = Defined {
        file: Some("notes.comp.md".into()),
        range: Some(on(1, 3)),
    };
    let there = located(&there, &root, "").expect("there");
    assert_eq!(
        (there.uri, there.range.start.character),
        (uri_of(&root.join("notes.comp.md")), 2)
    );
    let whole = located(
        &Defined {
            file: Some("notes.comp.md".into()),
            range: None,
        },
        &root,
        "",
    )
    .expect("the file");
    assert_eq!(whole.range.start, Position::default());
    let core = Defined {
        file: Some("@compmd/core@1.0.0/index.comp.md".into()),
        range: None,
    };
    assert_eq!(located(&core, &root, ""), None);
}
