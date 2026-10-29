//! Parsing after an edit (incremental processing): the units of blocks the edit touches, and one unit either side,
//! are parsed again on their own, placed where they are; the units after them keep their blocks and move by the
//! edit's change in lines and offsets. The result is the tree, the diagnostics and the units a parse of the whole
//! new text gives.
//!
//! A **unit** is a run of the document's top-level blocks that parses the same on its own: units are separated by a
//! blank line where no container is open, and a continuation line stays with the component line before it.

use super::blocks::Marks;
use super::source::START;
use super::{Converter, Labels, ParseOptions, Parsed, Source, frontmatter, parse};
use crate::diagnostic::{Diagnostic, Range};
use crate::tree::{Move, Node, Point, Root, Shift};
use std::collections::BTreeSet;

/// An edit of a document's text: the code points `start..end` of the text the processor last saw became `text`.
///
/// Positions count code points from the start of the text, as the trees' `offset`s do, after a leading byte order
/// mark (spec §10.5). A position past the end of the text is the end.
///
/// ```
/// use compmd::TextEdit;
///
/// // In `Hello world`, select `world` and type `there`.
/// let edit = TextEdit { start: 6, end: 11, text: "there".into() };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextEdit {
    /// Where the replaced text starts.
    pub start: usize,
    /// Where the replaced text ends: `start` for an insertion.
    pub end: usize,
    /// What replaces it: empty for a deletion.
    pub text: String,
}

/// A unit of a document's top-level blocks.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Unit {
    /// Where its first block starts.
    start: Point,
    /// Where its last block ends.
    end: Point,
    /// How many of the tree's top-level blocks it holds.
    nodes: usize,
    /// How many parse diagnostics converting it gave.
    diagnostics: usize,
    /// The labels its blocks define.
    labels: Labels,
}

impl Unit {
    /// The units `marks` noted while converting the blocks `children`, after which there were `diagnostics`.
    pub(super) fn from_marks(marks: &Marks, children: &[Node], diagnostics: usize) -> Vec<Self> {
        let starts = &marks.starts;
        starts
            .iter()
            .enumerate()
            .map(|(index, mark)| {
                let next = starts.get(index + 1);
                let nodes = next.map_or(children.len(), |next| next.nodes);
                Self {
                    start: mark.start,
                    end: next.map_or(marks.end, |next| next.after),
                    nodes: nodes - mark.nodes,
                    diagnostics: next.map_or(diagnostics, |next| next.diagnostics)
                        - mark.diagnostics,
                    labels: Labels::of(&children[mark.nodes..nodes]),
                }
            })
            .collect()
    }

    fn shift(&mut self, by: Move) {
        self.start.shift(by);
        self.end.shift(by);
    }
}

impl Labels {
    /// The labels `nodes` define, at any depth, outside `compmd` definition bodies, which resolve their own.
    fn of(nodes: &[Node]) -> Self {
        let mut labels = Self::default();
        labels.collect_tree(nodes);
        labels
    }

    fn collect_tree(&mut self, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Definition(definition) => self.links.push(definition.identifier.clone()),
                Node::FootnoteDefinition(definition) => {
                    self.footnotes.push(definition.identifier.clone());
                }
                Node::ComponentContainer(container) => {
                    for slot in &container.slots {
                        self.collect_tree(&slot.children);
                    }
                }
                _ => {}
            }
            if !matches!(node, Node::ComponentDefinition(_)) {
                self.collect_tree(node.children());
            }
        }
    }

    fn extend(&mut self, other: &Self) {
        self.links.extend_from_slice(&other.links);
        self.footnotes.extend_from_slice(&other.footnotes);
    }

    /// The labels of `units`.
    fn of_units(units: &[Unit]) -> Self {
        let mut labels = Self::default();
        for unit in units {
            labels.extend(&unit.labels);
        }
        labels
    }

    /// Whether both define the same labels, whatever their order and repeats.
    fn same(&self, other: &Self) -> bool {
        same_set(&self.links, &other.links) && same_set(&self.footnotes, &other.footnotes)
    }
}

fn same_set(one: &[String], other: &[String]) -> bool {
    one.iter().collect::<BTreeSet<_>>() == other.iter().collect::<BTreeSet<_>>()
}

/// What parsing a slice of the units gave.
#[derive(Debug)]
enum Attempt {
    /// The parse of the new text.
    Done(Box<Parsed>),
    /// The slice's last unit didn't come out as before: what follows it may parse differently, so the slice grows.
    Grow,
    /// The edit changed which labels the document defines, so every unit parses again.
    Whole,
}

/// The parse of `text` after `edit`, from `previous`, the parse of `text`; and the new text.
pub(crate) fn reparse(
    previous: &Parsed,
    text: &str,
    edit: &TextEdit,
    options: &ParseOptions,
) -> (String, Parsed) {
    let (new_text, bom, edit) = apply(text, edit);
    let parsed = Slicer {
        previous,
        old: &text[bom..],
        new: &new_text[bom..],
        edit,
    }
    .parse(None)
    .unwrap_or_else(|| parse(&new_text, options));
    (new_text, parsed)
}

/// `text` after `edit`, the length of its byte order mark, and the edit in code points of the text after it.
fn apply(text: &str, edit: &TextEdit) -> (String, usize, Edit) {
    let bom = if text.starts_with('\u{feff}') {
        "\u{feff}"
    } else {
        ""
    };
    let body = &text[bom.len()..];
    let end = byte_index(body, edit.end);
    let start = byte_index(body, edit.start).min(end);
    let new_text = [bom, &body[..start], &edit.text, &body[end..]].concat();
    let replaced = body[start..end].chars().count();
    let before = body[..start].chars().count();
    let edit = Edit {
        start: before,
        end: before + replaced,
        offset: edit.text.chars().count().cast_signed() - replaced.cast_signed(),
    };
    (new_text, bom.len(), edit)
}

/// The edit, in code points of the old text, with how many code points it adds.
#[derive(Clone, Copy)]
struct Edit {
    start: usize,
    end: usize,
    offset: isize,
}

/// A slice an edit tried, for explaining a parse: the units it parsed again, and what came of it.
#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "the tests read it to explain a parse")
)]
struct Tried {
    first: usize,
    last: usize,
    outcome: &'static str,
}

/// Parses an edit's slice of the units again.
struct Slicer<'a> {
    previous: &'a Parsed,
    old: &'a str,
    new: &'a str,
    edit: Edit,
}

impl Slicer<'_> {
    /// The parse of the new text, or `None` when the whole text has to be parsed again: the edit may touch the
    /// frontmatter, or the labels the document defines. With `tried`, notes each slice it tries.
    fn parse(&self, mut tried: Option<&mut Vec<Tried>>) -> Option<Parsed> {
        let units = &self.previous.units;
        let last = units.len().checked_sub(1)?;
        let frontmatter = matches!(self.previous.root.children.first(), Some(Node::Yaml(_)));
        if !frontmatter && frontmatter::find(self.new).is_some() {
            return None;
        }
        let Edit { start, end, .. } = self.edit;
        let touched_first = units
            .iter()
            .position(|unit| unit.end.offset >= start)
            .unwrap_or(last);
        let touched_last = units
            .iter()
            .rposition(|unit| unit.start.offset <= end)
            .unwrap_or(0);
        let low = touched_first.min(touched_last);
        let high = touched_first.max(touched_last);
        let first = low.saturating_sub(1);
        if frontmatter && first == 0 {
            return None;
        }
        let mut after = 1;
        loop {
            let last_unit = (high + after).min(last);
            let attempt = self.attempt(first, last_unit);
            if let Some(tried) = tried.as_deref_mut() {
                tried.push(Tried {
                    first,
                    last: last_unit,
                    outcome: match attempt {
                        Attempt::Done(_) => "done",
                        Attempt::Grow => "grow",
                        Attempt::Whole => "whole",
                    },
                });
            }
            match attempt {
                Attempt::Done(parsed) => return Some(*parsed),
                Attempt::Whole => return None,
                Attempt::Grow => after *= 2,
            }
        }
    }

    /// Parses units `first..=last` again, from the end of the unit before them to the start of the unit after them.
    fn attempt(&self, first: usize, last: usize) -> Attempt {
        let previous = self.previous;
        let units = &previous.units;
        let to_end = last + 1 == units.len();
        let from = if first == 0 {
            START
        } else {
            units[first - 1].end
        };
        let old_to = if to_end {
            self.old.chars().count()
        } else {
            units[last + 1].start.offset
        };
        let new_to = old_to.saturating_add_signed(self.edit.offset);
        let old_slice = slice(self.old, from.offset, old_to);
        let new_slice = slice(self.new, from.offset, new_to);
        let by = Move {
            lines: line_endings(new_slice).cast_signed() - line_endings(old_slice).cast_signed(),
            offset: self.edit.offset,
        };
        let mut outside = Labels::of_units(&units[..first]);
        outside.extend(&Labels::of_units(&units[last + 1..]));
        let mut diagnostics = Vec::new();
        let (root, new_units) = Converter {
            source: Source::new(new_slice, from),
            syntax: previous.syntax,
            labels: &outside,
            frontmatter: None,
            diagnostics: &mut diagnostics,
        }
        .units(first == 0);
        if !Labels::of_units(&new_units).same(&Labels::of_units(&units[first..=last])) {
            return Attempt::Whole;
        }
        let nodes = |units: &[Unit]| units.iter().map(|unit| unit.nodes).sum::<usize>();
        let found = |units: &[Unit]| units.iter().map(|unit| unit.diagnostics).sum::<usize>();
        let head = previous.diagnostics.len() - found(units);
        let kept_nodes = (nodes(&units[..first]), nodes(&units[..=last]));
        let kept_diagnostics = (head + found(&units[..first]), head + found(&units[..=last]));
        let mut later_nodes = previous.root.children[kept_nodes.1..].to_vec();
        let mut later_diagnostics = previous.diagnostics[kept_diagnostics.1..].to_vec();
        if !to_end {
            let old_last = &units[last];
            let mut unit = old_last.clone();
            unit.shift(by);
            let mut old_nodes =
                previous.root.children[kept_nodes.1 - old_last.nodes..kept_nodes.1].to_vec();
            old_nodes.shift(by);
            let mut old_diagnostics = previous.diagnostics
                [kept_diagnostics.1 - old_last.diagnostics..kept_diagnostics.1]
                .to_vec();
            shift_diagnostics(&mut old_diagnostics, by);
            let same = new_units.last().is_some_and(|new_last| {
                let nodes_from = root.children.len() - new_last.nodes;
                let diagnostics_from = diagnostics.len() - new_last.diagnostics;
                *new_last == unit
                    && root.children[nodes_from..] == old_nodes[..]
                    && diagnostics[diagnostics_from..] == old_diagnostics[..]
            });
            if !same {
                return Attempt::Grow;
            }
            later_nodes.shift(by);
            shift_diagnostics(&mut later_diagnostics, by);
        }
        let position = previous.root.position.map(|mut position| {
            match root.position {
                Some(slice) if to_end => position.end = slice.end,
                _ => position.end.shift(by),
            }
            position
        });
        let mut later_units = units[last + 1..].to_vec();
        for unit in &mut later_units {
            unit.shift(by);
        }
        let mut parsed = Parsed {
            root: Root {
                children: previous.root.children[..kept_nodes.0].to_vec(),
                position,
            },
            settings: previous.settings.clone(),
            diagnostics: previous.diagnostics[..kept_diagnostics.0].to_vec(),
            units: units[..first].to_vec(),
            syntax: previous.syntax,
        };
        parsed.root.children.extend(root.children);
        parsed.root.children.extend(later_nodes);
        parsed.diagnostics.extend(diagnostics);
        parsed.diagnostics.extend(later_diagnostics);
        parsed.units.extend(new_units);
        parsed.units.extend(later_units);
        Attempt::Done(Box::new(parsed))
    }
}

/// Moves the places of parse diagnostics in the processed document.
fn shift_diagnostics(diagnostics: &mut [Diagnostic], by: Move) {
    let shift = |range: &mut Range| {
        range.start.line = range.start.line.saturating_add_signed(by.lines);
        if let Some(end) = &mut range.end {
            end.line = end.line.saturating_add_signed(by.lines);
        }
    };
    for diagnostic in diagnostics {
        shift(&mut diagnostic.range);
        for related in diagnostic
            .related
            .iter_mut()
            .filter(|related| related.file.is_none())
        {
            shift(&mut related.range);
        }
    }
}

/// The byte index of code point `offset` of `text`, or its length past the end.
fn byte_index(text: &str, offset: usize) -> usize {
    text.char_indices()
        .nth(offset)
        .map_or(text.len(), |(index, _)| index)
}

/// Code points `from..to` of `text`.
fn slice(text: &str, from: usize, to: usize) -> &str {
    &text[byte_index(text, from)..byte_index(text, to)]
}

/// How many line endings `text` has: LF, CR or CRLF.
fn line_endings(text: &str) -> usize {
    text.matches('\n').count() + text.matches('\r').count() - text.matches("\r\n").count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
    use std::path::Path;

    /// A small, seeded random number generator (xorshift), so a failure repeats.
    struct Random(u64);

    impl Random {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            usize::try_from(self.0 % bound as u64).unwrap_or(0)
        }

        fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
            items[self.below(items.len())]
        }
    }

    /// What edits type and paste: characters with a meaning in Markdown or CompMD, and whole blocks.
    const TYPED: &[&str] = &[
        "a", " ", "\n", "\n\n", "#", "-", "*", ":", "::", "[", "]", "`", ">", "|", "{", "}", "^",
        "$", "=", "+", "!", "\t", "\r\n", "é", "😀", "  ", "1.",
    ];
    const PASTED: &[&str] = &[
        "\n\n::: card {title=\"x\"}\nBody.\n::: /card\n\n",
        "::: card\n",
        "::: /card\n",
        "\n```\n",
        "```compmd component=note\n---\nSee [l] and :badge[b].\n```\n",
        "\n[l]: /x\n",
        "\n[^1]: A note.\n",
        "See [l] and[^1].",
        ":: chart[Weekly [l]] {rows=(r)}\n",
        "::: + gap=s\n",
        ":: #foot\n",
        "\n- a\n- b\n\n  c\n",
        "> q\n> [!NOTE]\n",
        "\n---\n",
        "| a | b |\n|---|---|\n| 1 | 2 |\n",
        "$$\nx\n$$\n",
        "<div>\n:: x\n</div>\n",
        "{{md.doc.title}}",
        "Title\n===\n",
        "    code\n",
    ];

    fn random_edit(text: &str, random: &mut Random) -> TextEdit {
        let length = text.chars().count();
        let start = random.below(length + 1);
        let (end, text) = match random.below(6) {
            0 | 1 => (start, random.pick(TYPED).to_string()),
            2 => (start, random.pick(PASTED).to_string()),
            3 => ((start + 1 + random.below(3)).min(length), String::new()),
            4 => (
                (start + random.below(40)).min(length),
                random.pick(TYPED).to_string(),
            ),
            _ => (
                (start + random.below(200)).min(length),
                random.pick(PASTED).to_string(),
            ),
        };
        TextEdit { start, end, text }
    }

    /// The documents of the conformance set and the examples.
    fn documents() -> Vec<(String, String)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut found = Vec::new();
        let mut folders = vec![root.join("conformance"), root.join("examples")];
        while let Some(folder) = folders.pop() {
            let entries = std::fs::read_dir(&folder).expect("a folder");
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    folders.push(path);
                } else if path.to_string_lossy().ends_with(".comp.md") {
                    let text = std::fs::read_to_string(&path).expect("a document");
                    found.push((path.display().to_string(), text));
                }
            }
        }
        found.sort();
        found
    }

    /// The units of `parsed`, each with its text from `text`.
    fn units_of(parsed: &Parsed, text: &str) -> String {
        let mut written = String::new();
        for (index, unit) in parsed.units.iter().enumerate() {
            let _ = writeln!(
                written,
                "  {index}: {}..{} {:?}, {} blocks, {} diagnostics",
                unit.start.offset,
                unit.end.offset,
                slice(text, unit.start.offset, unit.end.offset),
                unit.nodes,
                unit.diagnostics
            );
        }
        written
    }

    /// How `edit` of `text`, parsed as `previous`, parses incrementally: the units before, each slice it tried and
    /// what came of it, and the units a parse of the whole new text gives.
    fn explain(previous: &Parsed, text: &str, edit: &TextEdit, options: &ParseOptions) -> String {
        let (new_text, bom, applied) = apply(text, edit);
        let mut tried = Vec::new();
        let outcome = Slicer {
            previous,
            old: &text[bom..],
            new: &new_text[bom..],
            edit: applied,
        }
        .parse(Some(&mut tried));
        let mut slices = String::new();
        for tried in &tried {
            let _ = writeln!(
                slices,
                "  units {}..={}: {}",
                tried.first, tried.last, tried.outcome
            );
        }
        format!(
            "the units before:\n{}the slices tried:\n{slices}{}\nthe units of the whole new text:\n{}",
            units_of(previous, text),
            if outcome.is_none() {
                "  parsed the whole text\n"
            } else {
                ""
            },
            units_of(&parse(&new_text, options), &new_text),
        )
    }

    /// After every edit, the parse after an edit equals the parse of the whole new text: the tree, the diagnostics
    /// and the units.
    #[test]
    fn an_edit_parses_as_the_whole_new_text_does() {
        let options = ParseOptions::default();
        let documents = documents();
        assert!(documents.len() > 100, "the conformance set");
        for (index, (path, text)) in documents.into_iter().enumerate() {
            let mut random = Random(0x9e37_79b9_7f4a_7c15 ^ (index as u64 + 1));
            let mut text = text;
            let mut parsed = parse(&text, &options);
            for _ in 0..40 {
                let edit = random_edit(&text, &mut random);
                let (new_text, incremental) = reparse(&parsed, &text, &edit, &options);
                let whole = parse(&new_text, &options);
                let first_difference = incremental
                    .root
                    .children
                    .iter()
                    .zip(&whole.root.children)
                    .position(|(one, other)| one != other);
                let context = || {
                    format!(
                        "{path}\nthe text {text:?}\nthe edit {edit:?}\nthe new text {new_text:?}\n{}the first block that differs: {first_difference:?}, {:#?} and {:#?}",
                        explain(&parsed, &text, &edit, &options),
                        first_difference.map(|index| &incremental.root.children[index]),
                        first_difference.map(|index| &whole.root.children[index]),
                    )
                };
                assert!(incremental.root == whole.root, "the tree: {}", context());
                assert!(
                    incremental.diagnostics == whole.diagnostics,
                    "the diagnostics {:#?} and {:#?}: {}",
                    incremental.diagnostics,
                    whole.diagnostics,
                    context()
                );
                assert!(
                    incremental.units == whole.units,
                    "the units {:#?} and {:#?}: {}",
                    incremental.units,
                    whole.units,
                    context()
                );
                text = new_text;
                parsed = incremental;
            }
        }
    }

    #[test]
    fn an_explanation_gives_the_units_and_the_slices_tried() {
        let options = ParseOptions::default();
        let text = "One.\n\nTwo.\n\n```\nThree.\n```\n\nFour.\n";
        let parsed = parse(text, &options);
        let typed = |start, text: &str| TextEdit {
            start,
            end: start,
            text: text.into(),
        };
        let explained = explain(&parsed, text, &typed(7, "!"), &options);
        assert!(
            explained.contains("  1: 6..10 \"Two.\", 1 blocks, 0 diagnostics\n"),
            "{explained}"
        );
        assert!(explained.contains("  units 0..=2: done\n"), "{explained}");
        let opened = explain(&parsed, text, &typed(6, "```\n"), &options);
        assert!(opened.contains("  units 0..=2: grow\n"), "{opened}");
        let defined = explain(&parsed, text, &typed(6, "[x]: /y\n\n"), &options);
        assert!(defined.contains("  units 0..=2: whole\n"), "{defined}");
        assert!(defined.contains("  parsed the whole text\n"), "{defined}");
    }

    #[test]
    fn diagnostics_move_with_their_ends_and_related_places() {
        let place = |line| Range {
            start: crate::diagnostic::Location::new(line, 2),
            end: Some(crate::diagnostic::Location::new(line + 1, 1)),
        };
        let mut diagnostic = Diagnostic::new(
            crate::diagnostic::Code::F004,
            crate::diagnostic::Location::new(3, 2),
            "x",
        );
        diagnostic.range = place(3);
        diagnostic.related = vec![
            crate::diagnostic::Related {
                file: None,
                range: place(5),
                message: "here".into(),
            },
            crate::diagnostic::Related {
                file: Some("other.comp.md".into()),
                range: place(5),
                message: "there".into(),
            },
        ];
        let mut diagnostics = [diagnostic];
        shift_diagnostics(
            &mut diagnostics,
            Move {
                lines: 2,
                offset: 9,
            },
        );
        assert_eq!(diagnostics[0].range, place(5));
        assert_eq!(diagnostics[0].related[0].range, place(7));
        assert_eq!(
            diagnostics[0].related[1].range,
            place(5),
            "another file's place stays"
        );
    }
}
