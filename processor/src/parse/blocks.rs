//! Block nodes, and CompMD's containers, slots and leaves built from the component lines among one parent's
//! children (spec §4, §10.3). A container opens and closes among the children of one CommonMark parent, so it nests
//! with lists and quotes (spec §4.1).

use super::Converter;
use super::line::{Form, classify};
use super::params::{List, Written, identity};
use super::source::right;
use crate::diagnostic::{Code, Diagnostic, Location};
use crate::tree::{
    Alert, Align, Blockquote, Code as CodeBlock, Container, FootnoteDefinition, Heading,
    Interpolation, Leaf, LinkDefinition, List as ListNode, ListItem, Literal, Math, Node, Param,
    Parent, Point, Position, Slot, Table, Void, Yaml,
};
use markdown::mdast;
use std::collections::HashSet;

/// A container open around a nested parent, as that parent sees it.
#[derive(Clone)]
pub(super) struct Outer {
    name: String,
    opener: Point,
}

/// Which containers of the outer parents a closer in a nested parent closed (F032).
type Closed = HashSet<usize>;

/// A container being built.
struct Open {
    container: Container,
    opener: Point,
    /// Where each named slot's line is.
    slot_lines: Vec<Point>,
    /// The named slot content goes to now, or the default slot.
    current: Option<usize>,
    /// A closer in a nested list item or quote named it (F032): it closes at its parent's end without F004.
    closed_inside: bool,
}

impl Open {
    fn content(&mut self) -> &mut Vec<Node> {
        match self.current {
            Some(slot) => &mut self.container.slots[slot].children,
            None => &mut self.container.children,
        }
    }
}

/// Who takes the params of a continuation line: the opener or leaf right before it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Owner {
    Nobody,
    Opener,
    Leaf,
}

/// What building one parent's children keeps track of.
struct Scope<'o> {
    /// The containers open around this parent, outermost first.
    outer: &'o [Outer],
    /// The containers open in this parent, innermost last.
    open: Vec<Open>,
    out: Vec<Node>,
    closed_outer: Closed,
    owner: Owner,
    /// The line the previous child ends on, when it is a paragraph (W108).
    paragraph_end: Option<usize>,
}

impl Scope<'_> {
    fn push(&mut self, node: Node) {
        match self.open.last_mut() {
            Some(open) => open.content().push(node),
            None => self.out.push(node),
        }
    }

    fn last_leaf(&mut self) -> Option<&mut Leaf> {
        let last = match self.open.last_mut() {
            Some(open) => open.content().last_mut(),
            None => self.out.last_mut(),
        };
        match last {
            Some(Node::ComponentLeaf(leaf)) => Some(leaf),
            _ => None,
        }
    }

    fn close(&mut self, open: Open) {
        let mut container = open.container;
        container.auto_closed = container.close_position.is_none();
        self.push(Node::ComponentContainer(container));
    }

    /// The containers open around a parent nested in this one.
    fn nested(&self) -> Vec<Outer> {
        let own = self.open.iter().map(|open| Outer {
            name: open.container.name.clone(),
            opener: open.opener,
        });
        self.outer.iter().cloned().chain(own).collect()
    }
}

/// Where a unit of the document's blocks starts (incremental processing): the blocks and diagnostics before it.
pub(super) struct Mark {
    pub(super) nodes: usize,
    pub(super) diagnostics: usize,
    pub(super) start: Point,
    /// Where the block before it ends.
    pub(super) after: Point,
}

/// Where the units of the document's blocks start, and where its last block ends.
#[derive(Default)]
pub(super) struct Marks {
    pub(super) starts: Vec<Mark>,
    pub(super) end: Point,
}

/// A component line, as the fork found it.
struct ComponentLine {
    text: Vec<char>,
    position: Option<Position>,
    start: Point,
}

impl Converter<'_, '_> {
    /// The children of one parent, which ends at `end`, with the containers in `outer` open around it. Returns them
    /// and which of `outer` a closer in them closed. With `marks`, notes where each unit of them starts: a unit ends
    /// where a blank line follows and no container is open, unless a continuation line comes next.
    pub(super) fn blocks(
        &mut self,
        nodes: Vec<mdast::Node>,
        end: Point,
        outer: &[Outer],
        mut marks: Option<&mut Marks>,
    ) -> (Vec<Node>, Closed) {
        let mut scope = Scope {
            outer,
            open: Vec::new(),
            out: Vec::new(),
            closed_outer: Closed::new(),
            owner: Owner::Nobody,
            paragraph_end: None,
        };
        let mut before: Option<&markdown::unist::Point> = None;
        let positions: Vec<Option<markdown::unist::Position>> =
            nodes.iter().map(|node| node.position().cloned()).collect();
        for (node, position) in nodes.into_iter().zip(&positions) {
            if let (Some(marks), Some(position)) = (marks.as_deref_mut(), position) {
                let starts = before.is_none_or(|before| {
                    scope.open.is_empty()
                        && blank_between(self.source.text(), before.offset, position.start.offset)
                        && !is_continuation(&node)
                });
                if starts {
                    let start = self.source.point(position.start.offset);
                    marks.starts.push(Mark {
                        nodes: scope.out.len(),
                        diagnostics: self.diagnostics.len(),
                        start,
                        after: before.map_or(start, |before| self.source.point(before.offset)),
                    });
                }
                marks.end = self.source.point(position.end.offset);
                before = Some(&position.end);
            }
            if let mdast::Node::CompmdRaw(raw) = node {
                let position = self.source.position(raw.position.as_ref());
                let line = ComponentLine {
                    text: raw.value.chars().collect(),
                    start: position.map_or(end, |position| position.start),
                    position,
                };
                scope.owner = self.component_line(&line, &mut scope);
                scope.paragraph_end = None;
                continue;
            }
            let paragraph_end = matches!(node, mdast::Node::Paragraph(_))
                .then(|| {
                    node.position()
                        .map(|position| self.source.point(position.end.offset).line)
                })
                .flatten();
            let nested = scope.nested();
            let (converted, closed) = self.block(node, &nested);
            for index in closed {
                match index.checked_sub(outer.len()) {
                    Some(own) => scope.open[own].closed_inside = true,
                    None => {
                        scope.closed_outer.insert(index);
                    }
                }
            }
            if let Some(converted) = converted {
                scope.push(converted);
            }
            scope.owner = Owner::Nobody;
            scope.paragraph_end = paragraph_end;
        }
        while let Some(open) = scope.open.pop() {
            if !open.closed_inside {
                self.diagnostics.push(
                    Diagnostic::new(
                        Code::F004,
                        open.opener.into(),
                        format!("`::: {}` isn't closed", open.container.name),
                    )
                    .with_related(Location::from(end), "closed here"),
                );
            }
            scope.close(open);
        }
        (scope.out, scope.closed_outer)
    }

    /// One block, with the containers in `outer` open around it.
    fn block(&mut self, node: mdast::Node, outer: &[Outer]) -> (Option<Node>, Closed) {
        let mut closed = Closed::new();
        let mut children =
            |this: &mut Self, nodes: Vec<mdast::Node>, position: Option<Position>| {
                let end = position.map_or(super::START, |position| position.end);
                let (children, inner) = this.blocks(nodes, end, outer, None);
                closed.extend(inner);
                children
            };
        let node = match node {
            mdast::Node::Blockquote(quote) => {
                let position = self.source.position(quote.position.as_ref());
                let mut blockquote = Blockquote {
                    children: children(self, quote.children, position),
                    alert: None,
                    position,
                };
                if self.syntax.compmd {
                    mark_alert(&mut blockquote);
                }
                Node::Blockquote(blockquote)
            }
            mdast::Node::List(list) => {
                let mut items = Vec::new();
                for item in list.children {
                    let (item, inner) = self.block(item, outer);
                    closed.extend(inner);
                    items.extend(item);
                }
                Node::List(ListNode {
                    ordered: list.ordered,
                    start: list.start,
                    spread: list.spread,
                    children: items,
                    position: self.source.position(list.position.as_ref()),
                })
            }
            mdast::Node::ListItem(item) => {
                let position = self.source.position(item.position.as_ref());
                Node::ListItem(ListItem {
                    spread: item.spread,
                    checked: item.checked,
                    children: children(self, item.children, position),
                    position,
                })
            }
            mdast::Node::FootnoteDefinition(footnote) => {
                let position = self.source.position(footnote.position.as_ref());
                Node::FootnoteDefinition(FootnoteDefinition {
                    identifier: footnote.identifier,
                    label: footnote.label,
                    children: children(self, footnote.children, position),
                    position,
                })
            }
            other => return (self.leaf_block(other), closed),
        };
        (Some(node), closed)
    }

    /// A block that holds no blocks.
    fn leaf_block(&mut self, node: mdast::Node) -> Option<Node> {
        Some(match node {
            mdast::Node::Paragraph(paragraph) => {
                let position = self.source.position(paragraph.position.as_ref());
                let children = self.inlines(paragraph.children, false);
                match &children[..] {
                    [Node::Interpolation(only)] if self.syntax.compmd => {
                        Node::Interpolation(Interpolation {
                            path: only.path.clone(),
                            block: true,
                            position,
                        })
                    }
                    _ => Node::Paragraph(Parent { children, position }),
                }
            }
            mdast::Node::Heading(heading) => Node::Heading(Heading {
                depth: heading.depth,
                children: self.inlines(heading.children, false),
                position: self.source.position(heading.position.as_ref()),
            }),
            mdast::Node::Code(code) => return Some(self.code(code)),
            mdast::Node::Math(math) => Node::Math(Math {
                meta: math.meta,
                value: math.value,
                position: self.source.position(math.position.as_ref()),
            }),
            mdast::Node::Html(html) => {
                let position = self.source.position(html.position.as_ref());
                if self.syntax.compmd {
                    self.component_lines_in_html(&html.value, position);
                }
                Node::Html(Literal {
                    value: html.value,
                    position,
                })
            }
            mdast::Node::Table(table) => Node::Table(Table {
                align: table.align.into_iter().map(align).collect(),
                children: self.leaf_blocks(table.children),
                position: self.source.position(table.position.as_ref()),
            }),
            mdast::Node::TableRow(row) => Node::TableRow(Parent {
                children: self.leaf_blocks(row.children),
                position: self.source.position(row.position.as_ref()),
            }),
            mdast::Node::TableCell(cell) => Node::TableCell(Parent {
                children: self.inlines(cell.children, true),
                position: self.source.position(cell.position.as_ref()),
            }),
            mdast::Node::Definition(definition) => Node::Definition(LinkDefinition {
                url_parts: self.url_parts(&definition.url),
                identifier: definition.identifier,
                label: definition.label,
                url: definition.url,
                title: definition.title,
                position: self.source.position(definition.position.as_ref()),
            }),
            mdast::Node::ThematicBreak(rule) => Node::ThematicBreak(Void {
                position: self.source.position(rule.position.as_ref()),
            }),
            mdast::Node::Yaml(yaml) => Node::Yaml(Yaml {
                value: yaml.value,
                parsed: self.frontmatter.take(),
                position: self.source.position(yaml.position.as_ref()),
            }),
            other => return self.inline(other, false),
        })
    }

    fn leaf_blocks(&mut self, nodes: Vec<mdast::Node>) -> Vec<Node> {
        nodes
            .into_iter()
            .filter_map(|node| self.leaf_block(node))
            .collect()
    }

    /// A fenced code block: a component definition when its info string has `component=` (spec §7), display math
    /// when its language is `math` (spec §2.3).
    fn code(&mut self, code: mdast::Code) -> Node {
        let position = self.source.position(code.position.as_ref());
        if self.syntax.compmd {
            if let Some(definition) = self.definition(&code, position) {
                return definition;
            }
            if code.lang.as_deref() == Some("math") {
                return Node::Math(Math {
                    meta: None,
                    value: code.value,
                    position,
                });
            }
        }
        Node::Code(CodeBlock {
            lang: code.lang,
            meta: code.meta,
            value: code.value,
            position,
        })
    }

    /// W116: a line inside an HTML block that starts with `::` is HTML, not a component line (spec §3.2).
    fn component_lines_in_html(&mut self, html: &str, position: Option<Position>) {
        let start = position.map_or(super::START, |position| position.start);
        for (i, line) in html.split('\n').enumerate().skip(1) {
            let indent = line.bytes().take_while(|b| *b == b' ').count();
            if indent <= 3 && line[indent..].starts_with("::") {
                self.diagnose(
                    Code::W116,
                    Location::new(start.line + i, indent + 1),
                    "this line is part of the HTML block above it, not a component line; put a blank line before it",
                );
            }
        }
    }

    /// One component line; returns who takes the params of a continuation line after it.
    fn component_line(&mut self, line: &ComponentLine, scope: &mut Scope) -> Owner {
        match classify(&line.text) {
            Form::Malformed { expected } => {
                let text: String = line.text.iter().collect();
                self.diagnose(
                    Code::F016,
                    line.start,
                    format!(
                        "`{text}` starts with `::` but is no component line; expected {expected}"
                    ),
                );
                scope.push(Node::Paragraph(Parent {
                    children: vec![Node::Text(Literal {
                        value: text,
                        position: line.position,
                    })],
                    position: line.position,
                }));
                Owner::Nobody
            }
            Form::Opener { name, params } => {
                self.after_paragraph(line, scope);
                let mut container = Container {
                    name,
                    position: line.position,
                    ..Container::default()
                };
                container.recovered = self.add_params(&mut container.params, params, line.start);
                scope.open.push(Open {
                    container,
                    opener: line.start,
                    slot_lines: Vec::new(),
                    current: None,
                    closed_inside: false,
                });
                Owner::Opener
            }
            Form::Leaf {
                name,
                fallback,
                params,
            } => {
                self.after_paragraph(line, scope);
                let fallback = if let Some((start, end)) = fallback {
                    let text: String = line.text[start..end].iter().collect();
                    self.fallback(&text, right(line.start, start))
                } else {
                    self.diagnose(
                        Code::W101,
                        line.start,
                        format!("`:: {name}` has no fallback text, which plain renderers show in its place"),
                    );
                    Vec::new()
                };
                let mut leaf = Leaf {
                    name,
                    fallback,
                    position: line.position,
                    ..Leaf::default()
                };
                leaf.recovered = self.add_params(&mut leaf.params, params, line.start);
                scope.push(Node::ComponentLeaf(leaf));
                Owner::Leaf
            }
            Form::Continuation { params } => self.continuation(line, params, scope),
            Form::Slot { name } => {
                self.slot(name, line, scope);
                Owner::Nobody
            }
            Form::Closer { name } => {
                self.closer(&name, line, scope);
                Owner::Nobody
            }
        }
    }

    /// W108: a container opener or leaf right after a paragraph line (spec §3.2 rule 2).
    fn after_paragraph(&mut self, line: &ComponentLine, scope: &Scope) {
        if scope
            .paragraph_end
            .is_some_and(|end| end + 1 == line.start.line)
        {
            self.diagnose(
                Code::W108,
                line.start,
                "a component line right after a paragraph line; plain renderers read it as part of the paragraph, so put a blank line before it",
            );
        }
    }

    /// `::: + params`: more params for the opener or leaf right before it (F012 otherwise).
    fn continuation(&mut self, line: &ComponentLine, params: List, scope: &mut Scope) -> Owner {
        match scope.owner {
            Owner::Nobody => {
                self.diagnose(
                    Code::F012,
                    line.start,
                    "a continuation line must follow a container opener, a leaf or another continuation line; ignored",
                );
            }
            Owner::Opener => {
                if let Some(open) = scope.open.last_mut() {
                    let broke = self.add_params(&mut open.container.params, params, line.start);
                    open.container.recovered |= broke;
                }
            }
            Owner::Leaf => {
                if let Some(leaf) = scope.last_leaf() {
                    let broke = self.add_params(&mut leaf.params, params, line.start);
                    leaf.recovered |= broke;
                }
            }
        }
        scope.owner
    }

    fn slot(&mut self, name: String, line: &ComponentLine, scope: &mut Scope) {
        let Some(open) = scope.open.last_mut() else {
            self.diagnose(
                Code::F015,
                line.start,
                format!("`:: #{name}` is outside any container; ignored"),
            );
            return;
        };
        if let Some(index) = open
            .container
            .slots
            .iter()
            .position(|slot| slot.name == name)
        {
            let first = open.slot_lines[index];
            open.current = Some(index);
            self.diagnostics.push(
                Diagnostic::new(
                    Code::F007,
                    line.start.into(),
                    format!(
                        "slot `{name}` is given twice in `::: {}`; its content joins the first",
                        open.container.name
                    ),
                )
                .with_related(first.into(), "first given here"),
            );
            return;
        }
        open.container.slots.push(Slot {
            name,
            children: Vec::new(),
            position: line.position,
        });
        open.slot_lines.push(line.start);
        open.current = Some(open.container.slots.len() - 1);
    }

    fn closer(&mut self, name: &str, line: &ComponentLine, scope: &mut Scope) {
        if let Some(index) = scope
            .open
            .iter()
            .rposition(|open| open.container.name == name)
        {
            while scope.open.len() - 1 > index {
                if let Some(inner) = scope.open.pop() {
                    self.diagnostics.push(
                        Diagnostic::new(
                            Code::F004,
                            inner.opener.into(),
                            format!(
                                "`::: {}` isn't closed before `::: /{name}`",
                                inner.container.name
                            ),
                        )
                        .with_related(line.start.into(), "closed here"),
                    );
                    scope.close(inner);
                }
            }
            if let Some(mut open) = scope.open.pop() {
                open.container.close_position = line.position;
                scope.close(open);
            }
        } else if let Some(index) = scope.outer.iter().rposition(|open| open.name == name) {
            self.diagnostics.push(
                Diagnostic::new(
                    Code::F032,
                    line.start.into(),
                    format!("`::: /{name}` closes a container opened outside its list item or quote; ignored"),
                )
                .with_related(scope.outer[index].opener.into(), "opened here"),
            );
            scope.closed_outer.insert(index);
        } else if let Some(innermost) = scope.open.pop() {
            self.diagnostics.push(
                Diagnostic::new(
                    Code::F003,
                    line.start.into(),
                    format!(
                        "`::: /{name}` names no open container; it closes `::: {}`",
                        innermost.container.name
                    ),
                )
                .with_related(innermost.opener.into(), "opened here"),
            );
            scope.close(innermost);
        } else {
            self.diagnose(
                Code::F005,
                line.start,
                format!("`::: /{name}` closes nothing; no container is open"),
            );
        }
    }

    /// Adds the params of `list`, written on the line that starts at `line_start`, to `params`: F017 for a param
    /// already there (the first wins), F011 where the list broke. Returns whether it broke.
    pub(super) fn add_params(
        &mut self,
        params: &mut Vec<Param>,
        list: List,
        line_start: Point,
    ) -> bool {
        self.add_params_at(params, list, |index| right(line_start, index))
    }

    /// [`Self::add_params`], with `place` giving where each index of the list's text is.
    pub(super) fn add_params_at(
        &mut self,
        params: &mut Vec<Param>,
        list: List,
        place: impl Fn(usize) -> Point,
    ) -> bool {
        let owned = |(key, breakpoint): (&str, Option<&str>)| {
            (key.to_string(), breakpoint.map(str::to_string))
        };
        let mut taken: Vec<(String, Option<String>)> = params
            .iter()
            .filter_map(|param| {
                identity(
                    param.key.as_deref(),
                    param.breakpoint.as_deref(),
                    &param.value,
                )
            })
            .map(owned)
            .collect();
        for written in list.params {
            if let Some((key, breakpoint)) = written.identity() {
                let identity = owned((key, breakpoint));
                if taken.contains(&identity) {
                    self.diagnose(
                        Code::F017,
                        place(written.start),
                        format!("`{key}` is given twice; the first one is used"),
                    );
                    continue;
                }
                taken.push(identity);
            }
            params.push(param(written, &place));
        }
        match list.broke_at {
            Some(at) => {
                self.diagnose(
                    Code::F011,
                    place(at),
                    "malformed param list: a quote or bracket that doesn't close, a character a bare value can't hold (quote the value, or write an expression), or `=` with no value",
                );
                true
            }
            None => false,
        }
    }
}

/// Whether the text between byte `from` and byte `to` is white space with a blank line in it.
fn blank_between(text: &str, from: usize, to: usize) -> bool {
    let gap = &text[from..to];
    let endings =
        gap.matches('\n').count() + gap.matches('\r').count() - gap.matches("\r\n").count();
    endings >= 2 && gap.chars().all(char::is_whitespace)
}

/// Whether `node` is a continuation line, `::: + params`, which belongs with the component line before it.
fn is_continuation(node: &mdast::Node) -> bool {
    matches!(node, mdast::Node::CompmdRaw(raw) if raw.value.strip_prefix(":::").is_some_and(|rest| rest.trim_start_matches([' ', '\t']).starts_with('+')))
}

fn param(written: Written, place: &impl Fn(usize) -> Point) -> Param {
    Param {
        key: written.key,
        breakpoint: written.breakpoint,
        value: written.value,
        position: Some(Position {
            start: place(written.start),
            end: place(written.end),
        }),
    }
}

fn align(kind: mdast::AlignKind) -> Option<Align> {
    match kind {
        mdast::AlignKind::Left => Some(Align::Left),
        mdast::AlignKind::Right => Some(Align::Right),
        mdast::AlignKind::Center => Some(Align::Center),
        mdast::AlignKind::None => None,
    }
}

/// `> [!NOTE]` on a quote's first line makes it an alert, without that line (spec §2.3).
fn mark_alert(quote: &mut Blockquote) {
    let Some(Node::Paragraph(first)) = quote.children.first_mut() else {
        return;
    };
    let Some(Node::Text(text)) = first.children.first_mut() else {
        return;
    };
    let Some((alert, marker_len)) = alert_marker(&text.value) else {
        return;
    };
    let rest = &text.value[marker_len..];
    if rest.is_empty() {
        first.children.remove(0);
    } else {
        if let Some(position) = &mut text.position {
            position.start = Point {
                line: position.start.line + 1,
                column: 1,
                offset: position.start.offset + text.value[..marker_len].chars().count(),
            };
        }
        text.value = rest.to_string();
    }
    if first.children.is_empty() {
        quote.children.remove(0);
    }
    quote.alert = Some(alert);
}

/// The alert `[!TYPE]` that starts `text` alone on its line, and the bytes it takes with its line ending.
fn alert_marker(text: &str) -> Option<(Alert, usize)> {
    let name_end = text.strip_prefix("[!")?.find(']')? + 2;
    let alert = match text[2..name_end].to_ascii_lowercase().as_str() {
        "note" => Alert::Note,
        "tip" => Alert::Tip,
        "important" => Alert::Important,
        "warning" => Alert::Warning,
        "caution" => Alert::Caution,
        _ => return None,
    };
    let after = &text[name_end + 1..];
    let spaces = after.len() - after.trim_start_matches([' ', '\t']).len();
    let rest = &after[spaces..];
    let ending = match rest.as_bytes() {
        [] => 0,
        [b'\n', ..] => 1,
        _ => return None,
    };
    Some((alert, name_end + 1 + spaces + ending))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_markers_stand_alone_on_their_line() {
        assert_eq!(alert_marker("[!NOTE]"), Some((Alert::Note, 7)));
        assert_eq!(alert_marker("[!tip] \nRest"), Some((Alert::Tip, 8)));
        assert_eq!(alert_marker("[!Caution]\t"), Some((Alert::Caution, 11)));
        assert_eq!(alert_marker("[!WARNING] text"), None);
        assert_eq!(alert_marker("[!OTHER]"), None);
        assert_eq!(alert_marker("[!NOTE"), None);
        assert_eq!(alert_marker("text"), None);
    }
}
