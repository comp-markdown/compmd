//! The walk through a body: interpolations in text and URLs (F014), definitions, and every invocation, whose slots
//! a template gives its own names.

use super::Checker;
use super::params::{literal, value_at};
use super::scope::Scope;
use crate::diagnostic::{Code, Location, Stage};
use crate::parse::params::interpolations;
use crate::reserved::Keyword;
use crate::resolve::{Binding, Body, Renderer};
use crate::tree::{
    Code as CodeBlock, Container, Definition, Interpolation, Node, Param, ParamValue, Position,
    StringPart, start_of,
};
use crate::values::forms::area_names;
use std::rc::Rc;

impl<'r> Checker<'r, '_> {
    pub(super) fn nodes(&mut self, nodes: &'r [Node], scope: &Scope<'r>) {
        for node in nodes {
            self.node(node, scope);
        }
    }

    fn node(&mut self, node: &'r Node, scope: &Scope<'r>) {
        let url = match node {
            Node::Link(link) => link.url_parts.as_deref().zip(link.position),
            Node::Image(image) => image.url_parts.as_deref().zip(image.position),
            Node::Definition(definition) => {
                definition.url_parts.as_deref().zip(definition.position)
            }
            _ => None,
        };
        if let Some((parts, position)) = url {
            self.url(parts, position, scope);
        }
        match node {
            Node::Interpolation(interpolation) => self.interpolation(interpolation, scope),
            Node::ComponentContainer(container) => self.container(container, scope),
            Node::ComponentLeaf(leaf) | Node::ComponentInline(leaf) => {
                self.invocation(&leaf.name, &leaf.params, leaf.position, None, scope);
                self.within(None, |this| this.nodes(&leaf.fallback, scope));
            }
            Node::ComponentDefinition(definition) => {
                self.within(None, |this| this.definition(definition));
            }
            Node::Code(code) => self.code_block(code, scope),
            _ => self.within(None, |this| this.nodes(node.children(), scope)),
        }
    }

    /// Runs `check` with `areas` as the areas of the box whose items it checks.
    fn within(&mut self, areas: Option<Rc<[String]>>, check: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.areas, areas);
        check(self);
        self.areas = outer;
    }

    /// The areas a container's items are placed in, when they are known: those of a literal `grid-template-areas` its
    /// invocation gives, or else its definition's fence; the outer ones in `if` and `each`, which expand to their
    /// content; none for a host-language component, whose code makes its box.
    fn items_areas(&self, container: &Container) -> Option<Rc<[String]>> {
        let resolved = self.resolved;
        let Some(Binding::Component(id, _)) = resolved.document(self.id).names.get(&container.name)
        else {
            return None;
        };
        let component = resolved.component(*id);
        if matches!(component.keyword(), Some(Keyword::If | Keyword::Each)) {
            return self.areas.clone();
        }
        if component.body.is_host() {
            return None;
        }
        let gives_areas = |param: &&Param| param.key.as_deref() == Some("grid-template-areas");
        let mut written: Vec<&Param> = container.params.iter().filter(gives_areas).collect();
        if written.is_empty() {
            written = component.fence.iter().filter(gives_areas).collect();
        }
        if written.is_empty() {
            return None;
        }
        let mut names: Vec<String> = Vec::new();
        for param in written {
            for name in area_names(&literal(&param.value)?)? {
                if !names.iter().any(|known| known == name) {
                    names.push(name.to_string());
                }
            }
        }
        Some(names.into())
    }

    /// A code block a code renderer renders (spec §7.7): F023 when two at one level render its language; otherwise
    /// what is wrong with its info string, its params as a leaf's, and W101 without `alt`.
    fn code_block(&mut self, code: &'r CodeBlock, scope: &Scope<'r>) {
        let resolved = self.resolved;
        let renderers = &resolved.document(self.id).renderers;
        let (Some(language), Some(info)) = (code.lang.as_deref(), &code.info) else {
            return;
        };
        let Some(renderer) = renderers.get(language) else {
            return;
        };
        let at = start_of(code.position);
        match renderer {
            Renderer::Ambiguous(names) => {
                self.report(
                    Code::F023,
                    at,
                    format!(
                        "`{language}` blocks are rendered by {}; list one in its import to choose",
                        names.join(" and ")
                    ),
                );
                self.fall_back(code.position);
            }
            Renderer::Component(name, _) => {
                for problem in &info.problems {
                    self.found.diagnostics.push(
                        problem
                            .clone()
                            .in_stage(Stage::Validate)
                            .in_file(self.file.clone()),
                    );
                }
                self.invocation(name, &info.params, code.position, None, scope);
                if let Some(alt) = &info.alt
                    && let ParamValue::String(parts) = &alt.value
                {
                    self.string_paths(parts, value_at(alt), scope);
                }
                if info.alt.is_none() {
                    self.report(
                        Code::W101,
                        at,
                        format!("`{name}` draws this block, which has no text alternative for screen readers; add `alt=\"…\"` after the language"),
                    );
                }
            }
        }
    }

    /// F014 for a path nothing in `scope` gives, reported at the path: with why, when a colon without a space after
    /// it in the frontmatter took the key's value.
    pub(super) fn path(&mut self, path: &str, at: Location, scope: &Scope) {
        if scope.knows(path) {
            return;
        }
        let why = scope
            .missing_space(path)
            .unwrap_or_else(|| "to write braces as text, use `\\{{` or a code span".to_string());
        self.report(
            Code::F014,
            at,
            format!("unknown interpolation path `{path}`; {why}"),
        );
    }

    fn interpolation(&mut self, interpolation: &Interpolation, scope: &Scope) {
        let start = interpolation
            .position
            .map(|position| position.start)
            .unwrap_or_default();
        let at = self.lines.path_at(start);
        self.path(&interpolation.path, at, scope);
    }

    /// The interpolations of a link's, an image's or a link definition's URL: the last ones in its text, since the
    /// URL follows the link text.
    fn url(&mut self, parts: &[StringPart], position: Position, scope: &Scope) {
        let paths: Vec<&str> = parts
            .iter()
            .filter_map(|part| match part {
                StringPart::Interpolation(interpolation) => Some(interpolation.path.as_str()),
                StringPart::Text(_) => None,
            })
            .collect();
        let span = self.lines.span(position);
        let chars: Vec<char> = span.iter().map(|(c, _)| *c).collect();
        let found = interpolations(&chars);
        let found = &found[found.len().saturating_sub(paths.len())..];
        for (path, (index, _)) in paths.into_iter().zip(found) {
            self.path(path, span[*index].1, scope);
        }
    }

    fn container(&mut self, container: &'r Container, scope: &Scope<'r>) {
        let invoked = self.invocation(
            &container.name,
            &container.params,
            container.position,
            Some(container),
            scope,
        );
        let areas = self.items_areas(container);
        self.within(areas, |this| {
            this.nodes(&container.children, &invoked.scope("default", scope));
        });
        self.within(None, |this| {
            for slot in &container.slots {
                this.nodes(&slot.children, &invoked.scope(&slot.name, scope));
            }
        });
    }

    /// A definition of this document: its fence's params, and a `compmd` body, in the definition's scope.
    pub(super) fn definition(&mut self, definition: &Definition) {
        let resolved = self.resolved;
        let document = resolved.document(self.id);
        let Some(Binding::Component(id, _)) = document.names.get(&definition.name) else {
            return;
        };
        let component = resolved.component(*id);
        if component.document != self.id || component.position != definition.position {
            return;
        }
        let scope = Scope::definition(&document.settings, component.signature.as_ref());
        self.fence(component, &scope);
        if let Body::Compmd(body) = &component.body {
            self.nodes(body, &scope);
        }
    }
}
