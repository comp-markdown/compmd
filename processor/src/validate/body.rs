//! The walk through a body: interpolations in text and URLs (F014), definitions, and every invocation, whose slots
//! a template gives its own names.

use super::Checker;
use super::scope::Scope;
use crate::diagnostic::{Code, Location};
use crate::parse::params::interpolations;
use crate::resolve::{Binding, Body};
use crate::tree::{Container, Definition, Interpolation, Node, Position, StringPart};

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
                self.nodes(&leaf.fallback, scope);
            }
            Node::ComponentDefinition(definition) => self.definition(definition),
            _ => self.nodes(node.children(), scope),
        }
    }

    /// F014 for a path nothing in `scope` gives, reported at the path.
    pub(super) fn path(&mut self, path: &str, at: Location, scope: &Scope) {
        if !scope.knows(path) {
            self.report(
                Code::F014,
                at,
                format!(
                    "unknown interpolation path `{path}`; to write braces as text, use `\\{{{{` or a code span"
                ),
            );
        }
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
        self.nodes(&container.children, &invoked.scope("default", scope));
        for slot in &container.slots {
            self.nodes(&slot.children, &invoked.scope(&slot.name, scope));
        }
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
