//! A file or a folder renamed, on a project on the disk.

use std::path::Path;

use lsp_types::{TextEdit, WorkspaceEdit};

use super::file_renames;
use crate::encoding::Encoding;
use crate::features::testing::project_folder;
use crate::files::OpenTexts;
use crate::project_index::{IndexedFile, ProjectIndex};

/// A project of `files` written to the disk and indexed.
fn project(
    name: &str,
    files: &[(&str, &str)],
) -> (std::path::PathBuf, std::sync::Arc<ProjectIndex>) {
    let root = project_folder(name);
    let _ = std::fs::remove_dir_all(&root);
    let index = ProjectIndex::new(root.clone());
    for (file, content) in files {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().expect("a folder")).expect("its folder");
        std::fs::write(&path, content).expect("written");
        index.set(path.clone(), IndexedFile::read(&path, content, false));
    }
    (root, index)
}

/// An edit as `(line, from, to, new text)`.
type Edited = (u32, u32, u32, String);

/// Each file's edits, by its path in `root`, the files in order.
fn edits_of(root: &Path, edit: &WorkspaceEdit) -> Vec<(String, Vec<Edited>)> {
    let root = url::Url::from_file_path(root).expect("a URL").to_string();
    let mut files: Vec<(String, Vec<Edited>)> = edit
        .changes
        .as_ref()
        .expect("changes")
        .iter()
        .map(|(uri, edits)| {
            let name = uri.as_str().strip_prefix(&root).unwrap_or(uri.as_str());
            let mut edits: Vec<(u32, u32, u32, String)> = edits
                .iter()
                .map(|TextEdit { range, new_text }| {
                    (
                        range.start.line,
                        range.start.character,
                        range.end.character,
                        new_text.clone(),
                    )
                })
                .collect();
            edits.sort();
            (name.trim_start_matches('/').to_owned(), edits)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn the_links_to_a_moved_file_and_its_own_relative_links_are_written_again() {
    let (root, index) = project(
        "rename-file",
        &[
            (
                "tides.md",
                "[Notes](./notes/harbour.md#top) and [from the root](/notes/harbour.md).\n",
            ),
            (
                "notes/harbour.md",
                "# Top\n\n[Tides](../tides.md), [log](log.md#x), [site](https://example.com) and [top](#top).\n",
            ),
            ("notes/log.md", "[back](harbour.md)\n\n[h]: ./harbour.md\n"),
        ],
    );
    let renames = [(
        root.join("notes/harbour.md"),
        root.join("posts/2026/harbour.md"),
    )];
    let edit = file_renames(
        &renames,
        [index.as_ref()],
        &OpenTexts::default(),
        Encoding::Utf16,
    );
    assert_eq!(
        edits_of(&root, &edit),
        [
            (
                "notes/harbour.md".into(),
                vec![
                    (2, 8, 19, "../../tides.md".into()),
                    (2, 28, 34, "../../notes/log.md".into()),
                ]
            ),
            (
                "notes/log.md".into(),
                vec![
                    (0, 7, 17, "../posts/2026/harbour.md".into()),
                    (2, 5, 17, "../posts/2026/harbour.md".into())
                ]
            ),
            (
                "tides.md".into(),
                vec![
                    (0, 8, 26, "./posts/2026/harbour.md".into()),
                    (0, 52, 69, "/posts/2026/harbour.md".into())
                ]
            ),
        ]
    );
}

#[test]
fn a_wiki_link_is_given_its_files_new_name_and_a_move_alone_leaves_it() {
    let wiki = "---\nmd:\n  syntax: [wiki-links]\n---\n";
    let index_text = format!("{wiki}[[Harbour#Top|the top]] [[notes/harbour]] [[Log]]\n");
    let (root, index) = project(
        "rename-wiki-file",
        &[
            ("index.md", &index_text),
            ("notes/harbour.comp.md", "# Top\n"),
            ("notes/log.md", "A log.\n"),
        ],
    );
    let renames = [
        (
            root.join("notes/harbour.comp.md"),
            root.join("notes/port.comp.md"),
        ),
        (root.join("notes/log.md"), root.join("archive/log.md")),
    ];
    let edit = file_renames(
        &renames,
        [index.as_ref()],
        &OpenTexts::default(),
        Encoding::Utf16,
    );
    assert_eq!(
        edits_of(&root, &edit),
        [(
            "index.md".into(),
            vec![(4, 2, 9, "port".into()), (4, 26, 39, "notes/port".into())]
        )]
    );
}

#[test]
fn a_folder_moved_moves_the_files_in_it() {
    let (root, index) = project(
        "rename-folder",
        &[
            ("index.md", "[Log](notes/log.md)\n"),
            ("notes/log.md", "[Index](../index.md) and [me](log.md)\n"),
        ],
    );
    std::fs::write(root.join("gone.md"), "[Log](notes/log.md)\n").expect("written");
    let gone = root.join("gone.md");
    index.set(
        gone.clone(),
        IndexedFile::read(&gone, "[Log](notes/log.md)\n", false),
    );
    std::fs::remove_file(&gone).expect("removed");
    let renames = [(root.join("notes"), root.join("archive/notes"))];
    let edit = file_renames(
        &renames,
        [index.as_ref()],
        &OpenTexts::default(),
        Encoding::Utf16,
    );
    assert_eq!(
        edits_of(&root, &edit),
        [
            (
                "index.md".into(),
                vec![(0, 6, 18, "archive/notes/log.md".into())]
            ),
            (
                "notes/log.md".into(),
                vec![(0, 8, 19, "../../index.md".into())]
            ),
        ],
        "a link within the folder stays as it is, and a file no longer on the disk is left alone"
    );
}
