//! Each line of a document read again for its tokens, as the parser reads it: a component line by its form, text for
//! its inline components and interpolations, the frontmatter for its `md` key, a definition's fence for its name and
//! params. The expressions it reads on the way, params' `(…)` and interpolations' `{{…}}`, are kept with where they
//! are, for what looks into them ([`crate::written_expressions`]).

use std::collections::BTreeSet;

use super::{SyntaxToken, TokenKind};
use crate::diagnostic::{Location, Range};
use crate::parse::line::{Form, classify, matching_bracket};
use crate::parse::params::{self, List, Written, interpolation_at, is_space, name_len};
use crate::resolve::Document;
use crate::surrounding::{Surrounding, indent, surrounding};
use crate::tree::{Node, ParamValue};
use crate::written_expressions::WrittenExpression;

/// What reading a document's lines gives.
pub(crate) struct Read {
    /// Its tokens, in the order they start.
    pub(crate) tokens: Vec<SyntaxToken>,
    /// Its expressions, in the order they start.
    pub(crate) expressions: Vec<WrittenExpression>,
}

/// The tokens and the expressions of `document`, each in the order they start.
pub(crate) fn read(document: &Document) -> Read {
    let mut fences = BTreeSet::new();
    fence_lines(&document.root.children, &mut fences);
    let lines = document.lines();
    let (mut tokens, mut expressions) = (Vec::new(), Vec::new());
    for number in 1..=document.line_starts.count() {
        let (Some(text), Some(start)) = (lines.line(number), document.line_starts.start(number))
        else {
            continue;
        };
        let mut line = Line {
            document,
            number,
            start,
            text: text.chars().collect(),
            tokens: &mut tokens,
            expressions: &mut expressions,
        };
        if fences.contains(&number) {
            line.fence();
        } else {
            line.read();
        }
    }
    tokens.sort_by_key(|token| token.range.start);
    expressions.sort_by_key(|expression| expression.start);
    Read {
        tokens,
        expressions,
    }
}

/// The lines of `nodes` on which a definition or signature fence opens.
fn fence_lines(nodes: &[Node], lines: &mut BTreeSet<usize>) {
    for node in nodes {
        match node {
            Node::ComponentDefinition(_) | Node::ComponentSignature(_) => {
                if let Some(position) = node.position() {
                    lines.insert(position.start.line);
                }
            }
            Node::ComponentContainer(container) => {
                for slot in &container.slots {
                    fence_lines(&slot.children, lines);
                }
            }
            _ => {}
        }
        fence_lines(node.children(), lines);
    }
}

/// One line being read.
struct Line<'d, 't> {
    document: &'d Document,
    number: usize,
    /// The document's code point the line starts at.
    start: usize,
    text: Vec<char>,
    tokens: &'t mut Vec<SyntaxToken>,
    expressions: &'t mut Vec<WrittenExpression>,
}

impl<'d> Line<'d, '_> {
    /// The line's tokens, by what holds it.
    fn read(&mut self) {
        let indent = indent(&self.text);
        match self.surrounding(indent) {
            Surrounding::Text if self.text[indent..].starts_with(&[':', ':']) => {
                self.component_line(indent);
            }
            Surrounding::Text => self.inline(0, self.text.len()),
            Surrounding::Frontmatter => self.yaml_key(),
            Surrounding::Toml => self.toml_key(),
            Surrounding::Code(_) | Surrounding::Opaque => {}
        }
    }

    fn surrounding(&self, at: usize) -> Surrounding<'d> {
        surrounding(&self.document.root.children, self.start + at)
    }

    /// The line's code points `start..end` as an expression.
    fn expression(&mut self, start: usize, end: usize) {
        if end > start {
            self.expressions.push(WrittenExpression {
                start: self.start + start,
                source: self.text[start..end].iter().collect(),
            });
        }
    }

    /// The code points `start..end` as a token of `kind`.
    fn token(&mut self, start: usize, end: usize, kind: TokenKind) {
        self.push(start, end, kind, false);
    }

    fn push(&mut self, start: usize, end: usize, kind: TokenKind, unknown: bool) {
        if end > start {
            self.tokens.push(SyntaxToken {
                kind,
                range: Range {
                    start: Location::new(self.number, start + 1),
                    end: Some(Location::new(self.number, end + 1)),
                },
                unknown,
            });
        }
    }

    /// The component name at `start`, known to the document or not.
    fn name(&mut self, start: usize, name: &str) {
        let unknown = self.document.names.get(name).is_none();
        self.push(
            start,
            start + name.chars().count(),
            TokenKind::ComponentName,
            unknown,
        );
    }

    /// A line that starts with `::` past its `indent`.
    fn component_line(&mut self, indent: usize) {
        let line = self.text[indent..].to_vec();
        let colons = line.iter().take_while(|c| **c == ':').count();
        let name_start = indent
            + colons
            + line[colons..]
                .iter()
                .take_while(|c| is_space(Some(c)))
                .count();
        self.token(indent, indent + colons, TokenKind::ComponentMarker);
        match classify(&line) {
            Form::Opener { name, params } => {
                self.name(name_start, &name);
                self.params(&params, indent);
            }
            Form::Leaf {
                name,
                fallback,
                params,
            } => {
                self.name(name_start, &name);
                if let Some((start, end)) = fallback {
                    self.inline(indent + start, indent + end);
                }
                self.params(&params, indent);
            }
            Form::Closer { name } => {
                self.token(name_start, name_start + 1, TokenKind::ComponentMarker);
                self.name(name_start + 1, &name);
            }
            Form::Continuation { params } => {
                self.token(name_start, name_start + 1, TokenKind::ComponentMarker);
                self.params(&params, indent);
            }
            Form::Slot { name } => {
                self.token(name_start, name_start + 1, TokenKind::ComponentMarker);
                self.token(
                    name_start + 1,
                    name_start + 1 + name.chars().count(),
                    TokenKind::SlotName,
                );
            }
            Form::Malformed { .. } => {
                if let Some(len) = name_len(&self.text, name_start) {
                    let name: String = self.text[name_start..name_start + len].iter().collect();
                    self.name(name_start, &name);
                }
            }
        }
    }

    /// The params of `list`, whose indexes are the line's past `shift`.
    fn params(&mut self, list: &List, shift: usize) {
        for param in &list.params {
            self.param(param, shift);
        }
    }

    fn param(&mut self, param: &Written, shift: usize) {
        let (start, end) = (param.start + shift, param.end + shift);
        let mut key_start = start;
        match &param.value {
            ParamValue::Class(_) => return self.token(start, end, TokenKind::ParamClass),
            ParamValue::Id(_) => return self.token(start, end, TokenKind::ParamId),
            _ => {}
        }
        if let Some(breakpoint) = &param.breakpoint {
            let breakpoint_end = start + breakpoint.chars().count();
            self.token(start, breakpoint_end, TokenKind::Breakpoint);
            key_start = breakpoint_end + 1;
        }
        let key_end = key_start + param.key.as_deref().map_or(0, |key| key.chars().count());
        self.token(key_start, key_end, TokenKind::ParamKey);
        let value_start = key_end + 1;
        match &param.value {
            ParamValue::Bare(_) => self.token(value_start, end, TokenKind::ParamBare),
            ParamValue::String(_) => self.string(value_start, end),
            ParamValue::Expr(_) => {
                self.token(value_start, value_start + 1, TokenKind::ExpressionParen);
                self.token(end - 1, end, TokenKind::ExpressionParen);
                self.expression(value_start + 1, end - 1);
            }
            // A flag has no value; JSON is its own language.
            _ => {}
        }
    }

    /// The quoted string at `start..end`: its text as strings, its interpolations as their own tokens.
    fn string(&mut self, start: usize, end: usize) {
        let mut piece = start;
        let mut at = start + 1;
        while at + 1 < end {
            if self.text[at] == '\\' && self.text[at + 1].is_ascii_punctuation() {
                at += 2;
            } else if let Some((_, len)) = interpolation_at(&self.text, at) {
                self.token(piece, at, TokenKind::ParamString);
                self.interpolation(at, len);
                at += len;
                piece = at;
            } else {
                at += 1;
            }
        }
        self.token(piece, end, TokenKind::ParamString);
    }

    /// The interpolation of `len` code points at `at`.
    fn interpolation(&mut self, at: usize, len: usize) {
        let path_start = at
            + 2
            + self.text[at + 2..]
                .iter()
                .take_while(|c| is_space(Some(c)))
                .count();
        let path_len = self.text[path_start..]
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
            .count();
        self.token(at, at + 2, TokenKind::InterpolationBrace);
        self.token(
            path_start,
            path_start + path_len,
            TokenKind::InterpolationPath,
        );
        self.token(at + len - 2, at + len, TokenKind::InterpolationBrace);
        self.expression(at + 2, at + len - 2);
    }

    /// The inline components and interpolations of the text at `from..to`, outside code spans and math.
    fn inline(&mut self, from: usize, to: usize) {
        // The param lists already read, which aren't text.
        let mut lists: Vec<(usize, usize)> = Vec::new();
        let mut at = from;
        while at < to {
            if let Some(&(_, end)) = lists.iter().find(|(start, _)| *start == at) {
                at = end;
                continue;
            }
            if self.text[at] == '\\' {
                at += 2;
                continue;
            }
            if let Some((_, len)) = interpolation_at(&self.text, at)
                && self.is_text(at)
            {
                self.interpolation(at, len);
                at += len;
                continue;
            }
            if let Some(end) = self.inline_component(at, &mut lists) {
                at = end;
                continue;
            }
            at += 1;
        }
    }

    /// The inline component `:name[fallback]{params}` whose `:` is at `colon`: its marker, its name and its params,
    /// the list added to `lists`; the index past its name, where its fallback starts.
    fn inline_component(&mut self, colon: usize, lists: &mut Vec<(usize, usize)>) -> Option<usize> {
        let text = &self.text;
        let after_word = colon
            .checked_sub(1)
            .is_some_and(|before| text[before] == ':' || text[before].is_alphanumeric());
        if text[colon] != ':' || after_word || text.get(colon + 1) == Some(&':') {
            return None;
        }
        let len = name_len(text, colon + 1)?;
        let end = colon + 1 + len;
        if text.get(end) != Some(&'[') || !self.is_text(colon) {
            return None;
        }
        let name: String = text[colon + 1..end].iter().collect();
        let list = matching_bracket(text, end)
            .map(|close| close + 1)
            .filter(|brace| {
                text.get(*brace) == Some(&'{') && interpolation_at(text, *brace).is_none()
            })
            .map(|brace| (brace, params::braced(text, brace)));
        self.token(colon, colon + 1, TokenKind::ComponentMarker);
        self.name(colon + 1, &name);
        if let Some((brace, list)) = list {
            self.params(&list, 0);
            lists.push((brace, list.end));
        }
        Some(end)
    }

    fn is_text(&self, at: usize) -> bool {
        self.surrounding(at) == Surrounding::Text
    }

    /// A YAML frontmatter's `md` key, at the start of its line.
    fn yaml_key(&mut self) {
        if self.text.starts_with(&['m', 'd', ':']) {
            self.token(0, 2, TokenKind::FrontmatterKey);
        }
    }

    /// A TOML frontmatter's `md` key: `md.version = 1`, `md = {…}`, `[md]`, `[[md.data]]`.
    fn toml_key(&mut self) {
        let mut at = self.text.iter().take_while(|c| is_space(Some(c))).count();
        at += self.text[at..]
            .iter()
            .take_while(|c| **c == '[')
            .count()
            .min(2);
        let key = self.text.get(at..at + 2) == Some(&['m', 'd'][..]);
        if key && matches!(self.text.get(at + 2), Some('.' | ']' | '=' | ' ' | '\t')) {
            self.token(at, at + 2, TokenKind::FrontmatterKey);
        }
    }

    /// A definition or signature fence's info: `component=name`, its flags (`signature`) and keys (`runtime=…`), then
    /// the param list that styles the component's box.
    fn fence(&mut self) {
        let brace = self
            .text
            .iter()
            .position(|c| *c == '{')
            .unwrap_or(self.text.len());
        // Past the indentation, the fence and the language.
        let fence = self.text.iter().take_while(|c| is_space(Some(c))).count();
        let mut at = fence
            + self.text[fence..]
                .iter()
                .take_while(|c| !is_space(Some(c)))
                .count();
        while at < brace {
            if is_space(self.text.get(at)) {
                at += 1;
                continue;
            }
            let word_end = at
                + self.text[at..brace]
                    .iter()
                    .take_while(|c| !is_space(Some(c)))
                    .count();
            let word: String = self.text[at..word_end].iter().collect();
            match word.split_once('=') {
                Some((key, value)) => {
                    let equals = at + key.chars().count();
                    self.token(at, equals, TokenKind::ParamKey);
                    if key == "component" {
                        self.name(equals + 1, value);
                    } else {
                        self.token(equals + 1, word_end, TokenKind::ParamBare);
                    }
                }
                None => self.token(at, word_end, TokenKind::ParamKey),
            }
            at = word_end;
        }
        if brace < self.text.len() {
            let list = params::braced(&self.text, brace);
            self.params(&list, 0);
        }
    }
}
