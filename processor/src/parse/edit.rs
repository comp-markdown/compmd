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
use crate::diagnostic::Diagnostic;
use crate::lines::{LineStarts, Place};
use crate::tree::{Move, Moved, Node, Point, Root, Shift};
use std::collections::BTreeSet;
use std::ops;
use std::rc::Rc;

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
enum Attempt {
    /// The slice, to splice into the previous parse.
    Done(Box<Slice>),
    /// The slice's last unit didn't come out as before: what follows it may parse differently, so the slice grows.
    Grow,
    /// The edit changed which labels the document defines, so every unit parses again.
    Whole,
}

/// What an edit changed in the parse of a document, for the stages after parsing.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Change {
    /// The frontmatter, or a component definition or signature fence of the document: the later stages run on the
    /// whole document again.
    Whole,
    /// The top-level blocks of some units changed, and the blocks after them moved.
    Blocks(Spliced),
}

/// The top-level blocks an edit changed, those of whole units. The blocks of the old tree before them are the new
/// tree's, as they were; those after them are the new tree's after `inserted`, moved.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Spliced {
    /// The blocks of the old tree that changed.
    pub(crate) blocks: ops::Range<usize>,
    /// How many blocks of the new tree replace them.
    pub(crate) inserted: usize,
    /// How the blocks after them moved.
    pub(crate) moved: Moved,
}

/// The parse of `text` after `edit`, from `previous`, the parse of `text`; the new text; and what changed. `lines`, the
/// line starts of `text`, become those of the new text.
pub(crate) fn reparse(
    previous: Parsed,
    text: &str,
    lines: &mut LineStarts,
    edit: &TextEdit,
    options: &ParseOptions,
) -> (String, Parsed, Change) {
    let (new_text, bom, edit) = apply(text, lines, edit);
    let (old, new) = (&text[bom..], &new_text[bom..]);
    let slice = Slicer {
        previous: &previous,
        old,
        new,
        lines,
        edit: edit.clone(),
    }
    .parse(None);
    lines.edit(
        edit.from,
        edit.to,
        &new[edit.from.byte..edit.from.byte + edit.written],
    );
    if let Some(slice) = slice {
        let change = slice.change(&previous, &edit);
        return (new_text, slice.splice_into(previous), change);
    }
    let parsed = parse(&new_text, options);
    let change = whole_change(&previous, &parsed, old, new, &edit);
    (new_text, parsed, change)
}

/// What changed from `previous` to `parsed`, the parse of the whole text after `edit`.
fn whole_change(previous: &Parsed, parsed: &Parsed, old: &str, new: &str, edit: &Edit) -> Change {
    let frontmatter = |parsed: &Parsed| {
        parsed
            .root
            .children
            .first()
            .filter(|node| matches!(node, Node::Yaml(_)))
            .cloned()
    };
    if previous.settings != parsed.settings
        || previous.syntax != parsed.syntax
        || frontmatter(previous) != frontmatter(parsed)
    {
        return Change::Whole;
    }
    let by = Move {
        lines: line_endings(new).cast_signed() - line_endings(old).cast_signed(),
        offset: edit.offset,
    };
    let old = Run::new(&previous.units, &previous.root.children);
    let new = Run::new(&parsed.units, &parsed.root.children);
    change(0, &old, &new, None, edit, by)
}

/// Units of a parse with their blocks.
struct Run<'p> {
    units: &'p [Unit],
    nodes: &'p [Node],
    /// Where each unit's blocks start in `nodes`, and where the last one's end.
    starts: Vec<usize>,
}

impl<'p> Run<'p> {
    fn new(units: &'p [Unit], nodes: &'p [Node]) -> Self {
        let starts = std::iter::once(0)
            .chain(units.iter().scan(0, |start, unit| {
                *start += unit.nodes;
                Some(*start)
            }))
            .collect();
        Self {
            units,
            nodes,
            starts,
        }
    }

    /// The blocks of the units `units`.
    fn blocks(&self, units: ops::Range<usize>) -> &'p [Node] {
        &self.nodes[self.starts[units.start]..self.starts[units.end]]
    }

    /// Whether unit `index`, moved `by`, is unit `other` of `new`.
    fn moved_is(&self, index: usize, by: Move, new: &Self, other: usize) -> bool {
        let mut unit = self.units[index].clone();
        unit.shift(by);
        let mut blocks = self.blocks(index..index + 1).to_vec();
        blocks.shift(by);
        unit == new.units[other] && blocks == new.blocks(other..other + 1)
    }
}

/// What changed from the units `old`, whose first block is block `first` of the old tree, to the units `new`, after
/// `edit`, which moved what follows it `by`; `after` is where the unit after `old` starts, if one does. A unit that
/// ends before the edit and comes out the same, or starts after it and comes out moved, didn't change.
fn change(
    first: usize,
    old: &Run,
    new: &Run,
    after: Option<Point>,
    edit: &Edit,
    by: Move,
) -> Change {
    let shortest = old.units.len().min(new.units.len());
    let head = (0..shortest)
        .take_while(|&index| {
            old.units[index].end.offset <= edit.start
                && old.units[index] == new.units[index]
                && old.blocks(index..index + 1) == new.blocks(index..index + 1)
        })
        .count();
    let tail = (0..shortest - head)
        .take_while(|&back| {
            let (index, other) = (old.units.len() - 1 - back, new.units.len() - 1 - back);
            old.units[index].start.offset >= edit.end && old.moved_is(index, by, new, other)
        })
        .count();
    let removed = head..old.units.len() - tail;
    let inserted = head..new.units.len() - tail;
    let defines = |nodes: &[Node]| {
        nodes.iter().any(|node| {
            matches!(
                node,
                Node::ComponentDefinition(_) | Node::ComponentSignature(_)
            )
        })
    };
    if defines(old.blocks(removed.clone())) || defines(new.blocks(inserted.clone())) {
        return Change::Whole;
    }
    let from = old.units.get(removed.end).map(|unit| unit.start).or(after);
    Change::Blocks(Spliced {
        blocks: first + old.starts[removed.start]..first + old.starts[removed.end],
        inserted: new.starts[inserted.end] - new.starts[inserted.start],
        moved: from.map_or(Moved::NOTHING, |from| Moved { from, by }),
    })
}

/// `text` after `edit`, the length of its byte order mark, and the edit in the text after it; `lines` are the line
/// starts of `text`.
fn apply(text: &str, lines: &LineStarts, edit: &TextEdit) -> (String, usize, Edit) {
    let bom = if text.starts_with('\u{feff}') {
        "\u{feff}"
    } else {
        ""
    };
    let body = &text[bom.len()..];
    let to = lines.place(body, edit.end);
    let from = lines.place(body, edit.start.min(to.offset));
    let new_text = [bom, &body[..from.byte], &edit.text, &body[to.byte..]].concat();
    let applied = Edit {
        start: from.offset,
        end: to.offset,
        offset: edit.text.chars().count().cast_signed() - (to.offset - from.offset).cast_signed(),
        from,
        to,
        written: edit.text.len(),
    };
    (new_text, bom.len(), applied)
}

/// The edit, in code points of the old text, with how many code points it adds; and where it is in bytes, with how
/// many bytes it writes.
#[derive(Clone)]
struct Edit {
    start: usize,
    end: usize,
    offset: isize,
    from: Place,
    to: Place,
    written: usize,
}

impl Edit {
    /// Where `place` of the old text, at or after the edit, is in the new text, in bytes.
    fn byte_after(&self, place: Place) -> usize {
        place.byte + self.written - (self.to.byte - self.from.byte)
    }
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
    /// The line starts of the old text.
    lines: &'a LineStarts,
    edit: Edit,
}

impl Slicer<'_> {
    /// The slice to splice into the previous parse, or `None` when the whole text has to be parsed again: the edit
    /// may touch the frontmatter, or the labels the document defines. With `tried`, notes each slice it tries.
    fn parse(&self, mut tried: Option<&mut Vec<Tried>>) -> Option<Slice> {
        let units = &self.previous.units;
        let last = units.len().checked_sub(1)?;
        let frontmatter = matches!(self.previous.root.children.first(), Some(Node::Yaml(_)));
        if !frontmatter && frontmatter::find(self.new).is_some() {
            return None;
        }
        let (start, end) = (self.edit.start, self.edit.end);
        let touched_first = units
            .partition_point(|unit| unit.end.offset < start)
            .min(last);
        let touched_last = units
            .partition_point(|unit| unit.start.offset <= end)
            .saturating_sub(1);
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
                Attempt::Done(slice) => return Some(*slice),
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
        let start = self.lines.place(self.old, from.offset).byte;
        let old_to = if to_end {
            self.lines.place(self.old, usize::MAX)
        } else {
            self.lines.place(self.old, units[last + 1].start.offset)
        };
        let old_slice = &self.old[start..old_to.byte];
        let new_slice = &self.new[start..self.edit.byte_after(old_to)];
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
        let slice = Slice {
            first,
            last,
            by,
            to_end,
            root,
            diagnostics,
            units: new_units,
        };
        if !to_end && !slice.ends_as(previous) {
            return Attempt::Grow;
        }
        Attempt::Done(Box::new(slice))
    }
}

/// Units `first..=last` of a parse, parsed again after an edit.
struct Slice {
    first: usize,
    last: usize,
    /// How far the edit moved what follows the slice.
    by: Move,
    /// Whether the slice reaches the end of the text.
    to_end: bool,
    root: Root,
    diagnostics: Vec<Diagnostic>,
    units: Vec<Unit>,
}

impl Slice {
    /// What `edit` changed from units `first..=last` of `previous` to the slice.
    fn change(&self, previous: &Parsed, edit: &Edit) -> Change {
        let (nodes, _) = Self::kept(previous, self.first, self.last + 1);
        let old = Run::new(
            &previous.units[self.first..=self.last],
            &previous.root.children[nodes.clone()],
        );
        let new = Run::new(&self.units, &self.root.children);
        let after = previous.units.get(self.last + 1).map(|unit| unit.start);
        change(nodes.start, &old, &new, after, edit, self.by)
    }

    /// Where units `first..last` of `previous` start, as blocks and diagnostics.
    fn kept(
        previous: &Parsed,
        first: usize,
        last: usize,
    ) -> (ops::Range<usize>, ops::Range<usize>) {
        let units = &previous.units;
        let nodes = |units: &[Unit]| units.iter().map(|unit| unit.nodes).sum::<usize>();
        let found = |units: &[Unit]| units.iter().map(|unit| unit.diagnostics).sum::<usize>();
        let head = previous.diagnostics.len() - found(units);
        (
            nodes(&units[..first])..nodes(&units[..last]),
            head + found(&units[..first])..head + found(&units[..last]),
        )
    }

    /// Whether its last unit came out as unit `last` of `previous`, moved: then what follows it parses as before.
    fn ends_as(&self, previous: &Parsed) -> bool {
        let (nodes, diagnostics) = Self::kept(previous, self.last, self.last + 1);
        let mut unit = previous.units[self.last].clone();
        unit.shift(self.by);
        let mut old_nodes = previous.root.children[nodes].to_vec();
        old_nodes.shift(self.by);
        let mut old_diagnostics = previous.diagnostics[diagnostics].to_vec();
        old_diagnostics.shift(self.by);
        self.units.last().is_some_and(|new| {
            *new == unit
                && self.root.children[self.root.children.len() - new.nodes..] == old_nodes[..]
                && self.diagnostics[self.diagnostics.len() - new.diagnostics..]
                    == old_diagnostics[..]
        })
    }

    /// `previous` with the slice in place of its units `first..=last`, and what follows them moved. The tree changes
    /// in place unless something else still shares it.
    fn splice_into(self, mut previous: Parsed) -> Parsed {
        let (nodes, diagnostics) = Self::kept(&previous, self.first, self.last + 1);
        let root = Rc::make_mut(&mut previous.root);
        if !self.to_end {
            root.children[nodes.end..].shift(self.by);
            previous.diagnostics[diagnostics.end..].shift(self.by);
            for unit in &mut previous.units[self.last + 1..] {
                unit.shift(self.by);
            }
        }
        if let Some(position) = &mut root.position {
            match self.root.position {
                Some(slice) if self.to_end => position.end = slice.end,
                _ => position.end.shift(self.by),
            }
        }
        root.children.splice(nodes, self.root.children);
        previous.diagnostics.splice(diagnostics, self.diagnostics);
        previous.units.splice(self.first..=self.last, self.units);
        previous
    }
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
        let chars: Vec<char> = text.chars().collect();
        let mut written = String::new();
        for (index, unit) in parsed.units.iter().enumerate() {
            let _ = writeln!(
                written,
                "  {index}: {}..{} {:?}, {} blocks, {} diagnostics",
                unit.start.offset,
                unit.end.offset,
                chars[unit.start.offset..unit.end.offset]
                    .iter()
                    .collect::<String>(),
                unit.nodes,
                unit.diagnostics
            );
        }
        written
    }

    /// `text` after `edit`, parsed again from `previous`, the parse of `text`.
    fn reparsed(
        previous: Parsed,
        text: &str,
        edit: &TextEdit,
        options: &ParseOptions,
    ) -> (String, Parsed, Change) {
        let mut lines = LineStarts::new(text);
        let reparsed = reparse(previous, text, &mut lines, edit, options);
        assert_eq!(lines, LineStarts::new(&reparsed.0), "the line starts");
        reparsed
    }

    /// How `edit` of `text`, parsed as `previous`, parses incrementally: the units before, each slice it tried and
    /// what came of it, and the units a parse of the whole new text gives.
    fn explain(previous: &Parsed, text: &str, edit: &TextEdit, options: &ParseOptions) -> String {
        let lines = LineStarts::new(text);
        let (new_text, bom, applied) = apply(text, &lines, edit);
        let mut tried = Vec::new();
        let outcome = Slicer {
            previous,
            old: &text[bom..],
            new: &new_text[bom..],
            lines: &lines,
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

    /// Why `change`, from `previous` to `parsed`, is wrong: a block outside its changed blocks isn't as it was, or
    /// moved.
    fn wrong_blocks(previous: &Parsed, parsed: &Parsed, change: &Change) -> Option<String> {
        let Change::Blocks(spliced) = change else {
            return None;
        };
        let (old, new) = (&previous.root.children, &parsed.root.children);
        if old.len() - spliced.blocks.len() != new.len() - spliced.inserted {
            return Some(format!("{spliced:?} doesn't count the blocks"));
        }
        if old[..spliced.blocks.start] != new[..spliced.blocks.start] {
            return Some(format!("a block before {spliced:?} changed"));
        }
        let after = &old[spliced.blocks.end..];
        if after.iter().any(|block| {
            !block
                .position()
                .is_some_and(|at| spliced.moved.moves(at.start))
        }) {
            return Some(format!(
                "a block after {spliced:?} isn't after where it moves"
            ));
        }
        let mut moved = after.to_vec();
        moved.shift(spliced.moved.by);
        (moved[..] != new[spliced.blocks.start + spliced.inserted..])
            .then(|| format!("a block after {spliced:?} didn't move"))
    }

    /// After every edit, the parse after an edit equals the parse of the whole new text: the tree, the diagnostics
    /// and the units; and what it says changed did.
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
                let (new_text, incremental, change) =
                    reparsed(parsed.clone(), &text, &edit, &options);
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
                if let Some(wrong) = wrong_blocks(&parsed, &incremental, &change) {
                    panic!("{wrong}: {}", context());
                }
                text = new_text;
                parsed = incremental;
            }
        }
    }

    #[test]
    fn an_edit_says_which_blocks_changed() {
        let options = ParseOptions::default();
        let text =
            "One.\n\nTwo.\n\n```compmd component=note\n---\nHi.\n```\n\nThree.\n\nSee [l].\n";
        let change = |start, inserted: &str| {
            let edit = TextEdit {
                start,
                end: start,
                text: inserted.into(),
            };
            reparsed(parse(text, &options), text, &edit, &options).2
        };
        let Change::Blocks(typed) = change(9, "!") else {
            panic!("a block changed");
        };
        assert_eq!((typed.blocks, typed.inserted), (1..2, 1));
        assert_eq!(typed.moved.from.offset, 12, "from the definition on");
        assert_eq!(
            typed.moved.by,
            Move {
                lines: 0,
                offset: 1
            }
        );
        assert_eq!(change(43, "!"), Change::Whole, "in a definition");
        assert_eq!(
            change(0, "---\na: 1\n---\n\n"),
            Change::Whole,
            "frontmatter"
        );
        let Change::Blocks(defined) = change(50, "[l]: /y\n\n") else {
            panic!("blocks changed");
        };
        assert_eq!(
            (defined.blocks, defined.inserted, defined.moved),
            (3..5, 3, Moved::NOTHING),
            "the definition, and the reference it makes a link"
        );
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
    fn an_edit_changes_a_tree_nothing_else_shares_in_place() {
        let options = ParseOptions::default();
        let text = "One.\n\nTwo.\n\nThree.\n";
        let parsed = parse(text, &options);
        let tree = Rc::as_ptr(&parsed.root);
        let edit = TextEdit {
            start: 9,
            end: 9,
            text: "!".into(),
        };
        let (_, edited, _) = reparsed(parsed, text, &edit, &options);
        assert_eq!(Rc::as_ptr(&edited.root), tree);
        let shared = Rc::clone(&edited.root);
        let (_, copied, _) = reparsed(edited, "One.\n\nTwo!.\n\nThree.\n", &edit, &options);
        assert_ne!(Rc::as_ptr(&copied.root), Rc::as_ptr(&shared));
        assert_eq!(
            shared.children.len(),
            3,
            "a tree another holds stays as it was"
        );
    }
}
