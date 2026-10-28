//! What a piece of a document sees while it is expanded: the names validating knows there, their values, a
//! definition's slots and breakpoint values, how its headings are leveled, and the origin of what it makes.

use super::Expander;
use crate::cel::{Type, Value, Values};
use crate::context::key;
use crate::expanded::Origin;
use crate::resolve::DocumentId;
use crate::tree::{Node, Slot};
use crate::validate::scope::Scope;
use std::rc::Rc;

/// What a piece of a document sees.
#[derive(Clone)]
pub(super) struct Env<'r> {
    /// The document it is written in.
    pub(super) document: DocumentId,
    /// The names it knows, as validating checked them.
    pub(super) scope: Scope<'r>,
    /// The values of its variables: `md`, a definition's params, `as` names.
    pub(super) values: Values,
    /// In a `compmd` definition's body: the invocation's slots.
    pub(super) slots: Option<Rc<Slots<'r>>>,
    /// In a body whose params have breakpoint values: the variables at `sm`, `md` and `lg` (spec §9.3).
    pub(super) breakpoints: Option<Rc<[Values; 3]>>,
    /// In a body with relative headings: the depth of the heading the invocation is under (spec §7.2).
    pub(super) shift: Option<u8>,
    /// In a body: the range of the invocation in the processed document that led to it, every node's origin.
    pub(super) origin: Option<Origin>,
}

/// The slots of a `compmd` component's invocation, expanded where its body places them.
pub(super) struct Slots<'r> {
    /// The default slot.
    pub(super) default: &'r [Node],
    /// The named slots.
    pub(super) named: &'r [Slot],
    /// The caller's environment, which slot content is expanded in.
    pub(super) env: Env<'r>,
    /// The template slots: each one's elements and the name each is bound to (spec §7.5).
    pub(super) templates: Vec<Template>,
}

/// A template slot of an invocation.
pub(super) struct Template {
    pub(super) slot: String,
    pub(super) items: Rc<Vec<Value>>,
    pub(super) name: String,
}

impl<'r> Env<'r> {
    /// The body of the document `id`.
    pub(super) fn document(expander: &Expander<'r>, id: DocumentId) -> Self {
        Self {
            document: id,
            scope: expander.scope(id),
            values: expander.values(id),
            slots: None,
            breakpoints: None,
            shift: None,
            origin: None,
        }
    }

    /// The current value of the input `name`: `null` where inputs aren't seen.
    pub(super) fn input(&self, name: &str) -> Value {
        let Some(Value::Map(md)) = self.values.get("md") else {
            return Value::Null;
        };
        match md.get(&key("input")) {
            Some(Value::Map(inputs)) => inputs.get(&key(name)).cloned().unwrap_or(Value::Null),
            _ => Value::Null,
        }
    }

    /// This environment inside a template: `name` bound to element `index` of `items`, and `md.loop` (spec §8.3).
    pub(super) fn iteration(&self, name: &str, items: &[Value], index: usize) -> Self {
        let count = items.len();
        let int = |n: usize| Value::Int(i64::try_from(n).unwrap_or(i64::MAX));
        let looped = Value::Map(Rc::new(
            [
                (key("index"), int(index)),
                (key("number"), int(index + 1)),
                (key("count"), int(count)),
                (key("first"), Value::Bool(index == 0)),
                (key("last"), Value::Bool(index + 1 == count)),
            ]
            .into_iter()
            .collect(),
        ));
        let mut inner = self.clone();
        inner.scope = self.scope.template(name, Type::Dyn);
        inner.values = with_loop(&self.values, &looped).with(name, items[index].clone());
        inner.breakpoints = self.breakpoints.as_ref().map(|values| {
            Rc::new(
                values
                    .as_ref()
                    .clone()
                    .map(|values| with_loop(&values, &looped).with(name, items[index].clone())),
            )
        });
        inner
    }
}

/// `values` with `md.loop` set to `looped`.
fn with_loop(values: &Values, looped: &Value) -> Values {
    let md = match values.get("md") {
        Some(Value::Map(fields)) => {
            let mut fields = fields.as_ref().clone();
            fields.insert(key("loop"), looped.clone());
            Value::Map(Rc::new(fields))
        }
        _ => Value::Null,
    };
    values.clone().with("md", md)
}
