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
mod tests;
