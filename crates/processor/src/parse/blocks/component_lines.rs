//! The component lines among one parent's children (spec §3, §4, §10.3): openers, leaves, continuation lines, slot
//! lines and closers, with their params and the diagnostics of each: F003–F005, F007, F011, F012, F015–F017, F032,
//! W101 and W108.

use super::{ComponentLine, Open, Owner, Scope};
use crate::diagnostic::{Code, Diagnostic};
use crate::parse::Converter;
use crate::parse::line::{Form, classify};
use crate::parse::params::{List, Written, identity};
use crate::parse::source::right;
use crate::tree::{Container, Leaf, Literal, Node, Param, Parent, Point, Position, Slot};

impl Converter<'_, '_> {
    /// One component line; returns who takes the params of a continuation line after it.
    pub(super) fn component_line(&mut self, line: &ComponentLine, scope: &mut Scope) -> Owner {
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
    pub(in crate::parse) fn add_params(
        &mut self,
        params: &mut Vec<Param>,
        list: List,
        line_start: Point,
    ) -> bool {
        self.add_params_at(params, list, |index| right(line_start, index))
    }

    /// [`Self::add_params`], with `place` giving where each index of the list's text is.
    pub(in crate::parse) fn add_params_at(
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
