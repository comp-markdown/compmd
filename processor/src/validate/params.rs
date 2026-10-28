//! Params against signatures (spec §5.3, §9.2): unknown params and breakpoints (F008), values outside their type or
//! grammar (F009), missing required params (F010), the types of expressions (F024), bound inputs (F030), and names
//! that can't be bound (F013). Then the invocation's slots (F006, F018), and W102, W103 and W120.

use super::Checker;
use super::grammar::Grammar;
use super::scope::Scope;
use super::universal::{Takes, Universal};
use crate::cel::Type;
use crate::cel::time::parse_date;
use crate::diagnostic::{Code, Location};
use crate::md::Signatures;
use crate::nearest::nearest;
use crate::reserved::{self, Keyword};
use crate::resolve::{Binding, Body, Component};
use crate::tree::{
    Container, Declaration, DeclaredType, Node, Param, ParamValue, Point, Position, Signature,
    SlotDeclaration, StringPart, TypeName,
};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// The breakpoints (spec §9.3).
const BREAKPOINTS: [&str; 3] = ["sm", "md", "lg"];

/// The grammar of a layout or styling type.
fn grammar(name: TypeName) -> Option<Grammar> {
    match name {
        TypeName::Length => Some(Grammar::Length),
        TypeName::Space => Some(Grammar::Space),
        TypeName::Color => Some(Grammar::Color),
        TypeName::Tracks => Some(Grammar::Tracks),
        TypeName::Surface => Some(Grammar::Surface),
        TypeName::Border => Some(Grammar::Border),
        TypeName::Radius => Some(Grammar::Radius),
        _ => None,
    }
}

/// Whether a value written as `value` is of the type `name` (spec §5.1). A string with interpolations is checked
/// when it is rendered.
fn accepts(name: TypeName, value: &ParamValue) -> bool {
    let Some(text) = literal(value) else {
        return matches!(value, ParamValue::String(_))
            || match name {
                TypeName::Boolean => *value == ParamValue::Flag,
                TypeName::Array => matches!(value, ParamValue::Json(json) if json.starts_with('[')),
                TypeName::Object => {
                    matches!(value, ParamValue::Json(json) if json.starts_with('{'))
                }
                TypeName::Enum => true,
                _ => false,
            };
    };
    let bare = matches!(value, ParamValue::Bare(_));
    match name {
        TypeName::String | TypeName::Enum => true,
        TypeName::Number => bare && text.parse::<f64>().is_ok_and(f64::is_finite),
        TypeName::Boolean => bare && (text == "true" || text == "false"),
        TypeName::Date => parse_date(&text).is_some(),
        TypeName::Array | TypeName::Object => false,
        TypeName::Name => bare && is_identifier(&text),
        layout => grammar(layout).is_some_and(|grammar| grammar.accepts(&text)),
    }
}

/// The text of a bare value, or of a string without interpolations.
fn literal(value: &ParamValue) -> Option<Cow<'_, str>> {
    match value {
        ParamValue::Bare(text) => Some(Cow::Borrowed(text)),
        ParamValue::String(parts) => parts
            .iter()
            .map(|part| match part {
                StringPart::Text(text) => Some(text.value.as_str()),
                StringPart::Interpolation(_) => None,
            })
            .collect::<Option<String>>()
            .map(Cow::Owned),
        _ => None,
    }
}

/// Whether a value written as `value` is of the type `declared`; a bound input is checked by its name instead.
fn fits(declared: &DeclaredType, value: &ParamValue) -> bool {
    match declared {
        DeclaredType::Enum { members } => {
            !matches!(value, ParamValue::Flag | ParamValue::Json(_))
                && literal(value).is_none_or(|text| members.iter().any(|member| *member == text))
        }
        DeclaredType::Name(name) => accepts(*name, value),
        DeclaredType::Other(_) | DeclaredType::Input { .. } => true,
    }
}

/// A CEL identifier, as `name` params, `as` names and inputs are.
fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether an input declared `input` can be bound to a param declared `input<param>` (spec §7.3).
fn binds(param: &DeclaredType, input: &DeclaredType) -> bool {
    match (param, input) {
        (DeclaredType::Name(TypeName::Enum), DeclaredType::Enum { .. }) => true,
        (DeclaredType::Enum { members: wanted }, DeclaredType::Enum { members }) => {
            wanted == members
        }
        (DeclaredType::Name(wanted), DeclaredType::Name(name)) => wanted == name,
        (DeclaredType::Other(wanted), DeclaredType::Other(name)) => wanted == name,
        _ => false,
    }
}

/// Where a param's value is: past its key, `=` and breakpoint; for a flag, a class or an id, the param itself.
pub(crate) fn value_at(param: &Param) -> Location {
    let start = param
        .position
        .map(|position| position.start)
        .unwrap_or_default();
    let key = param.key.as_deref().unwrap_or("");
    let prefix = match param.value {
        ParamValue::Flag | ParamValue::Class(_) | ParamValue::Id(_) => 0,
        _ => {
            param
                .breakpoint
                .as_ref()
                .map_or(0, |breakpoint| breakpoint.chars().count() + 1)
                + key.chars().count()
                + 1
        }
    };
    Location::new(start.line, start.column + prefix)
}

fn param_at(param: &Param) -> Location {
    param
        .position
        .map_or(Location::new(1, 1), |position| position.start.into())
}

/// The component an invocation's params are checked against.
struct Target<'r> {
    component: &'r Component,
    signature: &'r Signature,
    /// Whether it has no box, so that params for a box are W120.
    contents: bool,
}

impl Target<'_> {
    /// Whether its declared params take breakpoints: a `compmd` component's, and `box`'s, which are layout params
    /// (spec §9.3); not a host-language component's.
    fn takes_breakpoints(&self) -> bool {
        match self.component.body {
            Body::Compmd(_) => true,
            Body::Processor => self.is(Keyword::Box),
            Body::Host { .. } | Body::File { .. } => false,
        }
    }

    /// Whether it is the built-in `keyword`.
    fn is(&self, keyword: Keyword) -> bool {
        self.component.keyword() == Some(keyword)
    }
}

/// A template slot of an invocation (spec §7.5).
#[derive(Debug)]
struct Template {
    /// The slot.
    slot: String,
    /// The name the author binds each element to.
    bound: String,
    /// The type of the elements.
    element: Type,
}

/// What an invocation's slots see: its template slots bind a name to each element.
#[derive(Debug, Default)]
pub(super) struct Invoked {
    templates: Vec<Template>,
}

impl Invoked {
    /// What the slot `name` sees, inside the invocation whose own scope is `outer`.
    pub(super) fn scope<'r>(&self, name: &str, outer: &Scope<'r>) -> Scope<'r> {
        self.templates
            .iter()
            .find(|template| template.slot == name)
            .map_or_else(
                || outer.clone(),
                |template| outer.template(&template.bound, template.element.clone()),
            )
    }
}

/// What checking a declared param gave: given, with the type of its expression, or refused, so that a missing value
/// isn't reported again (F010).
enum Checked {
    Given(Option<Type>),
    Refused,
}

/// How a param of an invocation was judged.
enum Verdict {
    /// A declared param, with the type of its expression.
    Declared(Option<Type>),
    /// A param for the component's box: a common param, a class or an id.
    Universal,
    /// Treated as absent.
    Refused,
}

impl<'r> Checker<'r, '_> {
    /// Checks an invocation of `name`, and gives what its slots see.
    pub(super) fn invocation(
        &mut self,
        name: &str,
        params: &'r [Param],
        position: Option<Position>,
        container: Option<&'r Container>,
        scope: &Scope<'r>,
    ) -> Invoked {
        let start: Point = position.map(|position| position.start).unwrap_or_default();
        let resolved = self.resolved;
        let Some(Binding::Component(id, _)) = resolved.document(self.id).names.get(name) else {
            self.loose(params, scope);
            return Invoked::default();
        };
        let component = resolved.component(*id);
        if !component.body.supported() {
            self.report(
                Code::W102,
                start.into(),
                format!("this processor has no binding for `{name}`'s language or runtime, so it renders the fallback"),
            );
            self.fall_back(position);
        }
        let Some(signature) = &component.signature else {
            self.untyped(params, scope);
            return Invoked::default();
        };
        let target = Target {
            component,
            signature,
            contents: component.is_contents(),
        };
        let mut given = HashSet::new();
        let mut refused = HashSet::new();
        let mut types = HashMap::new();
        for param in params {
            let key = param.key.as_deref().unwrap_or("");
            match self.param(&target, param, scope) {
                Verdict::Declared(ty) => {
                    given.insert(key);
                    types.extend(ty.map(|ty| (key, ty)));
                }
                Verdict::Refused => {
                    refused.insert(key);
                    self.refuse_param(param);
                }
                Verdict::Universal => {}
            }
        }
        let required = signature
            .params
            .iter()
            .filter(|param| !param.optional && param.default.is_none());
        let mut falls_back = false;
        for declared in required.filter(|param| !given.contains(param.name.as_str())) {
            falls_back = true;
            // A refused value has been reported already.
            if !refused.contains(declared.name.as_str()) {
                self.report(
                    Code::F010,
                    start.into(),
                    format!(
                        "`{name}` needs `{}`; it renders its fallback",
                        declared.name
                    ),
                );
            }
        }
        if falls_back {
            self.fall_back(position);
        }
        if let Some(container) = container {
            self.slots(name, container, signature, start);
            if target.is(Keyword::Each)
                && !given.contains("join")
                && !refused.contains("join")
                && is_single_list_item(&container.children)
            {
                self.report(
                    Code::W103,
                    start.into(),
                    "the default slot is one list item, so each item becomes a list of its own; `join=list` makes one list",
                );
            }
        }
        Invoked {
            templates: templates(signature, params, &types),
        }
    }

    /// The params of a component without a signature (W105): the universal ones style the box the processor makes,
    /// the others go through unchecked.
    fn untyped(&mut self, params: &'r [Param], scope: &Scope<'r>) {
        for param in params {
            match param.key.as_deref().and_then(Universal::from_name) {
                Some(universal) => {
                    if !self.universal(universal, param, scope) {
                        self.refuse_param(param);
                    }
                }
                None => self.loose(std::slice::from_ref(param), scope),
            }
        }
    }

    /// The params of an unknown component, or ones nothing declares: their expressions and paths alone.
    fn loose(&mut self, params: &'r [Param], scope: &Scope<'r>) {
        for param in params {
            let at = value_at(param);
            match &param.value {
                ParamValue::Expr(source) => {
                    self.expression(source, at, scope, None, "");
                }
                ParamValue::String(parts) => self.string_paths(parts, at, scope),
                _ => {}
            }
        }
    }

    /// Checks one param of an invocation.
    fn param(&mut self, target: &Target<'r>, param: &'r Param, scope: &Scope<'r>) -> Verdict {
        if matches!(param.value, ParamValue::Class(_) | ParamValue::Id(_)) {
            if target.contents {
                self.no_box(param);
                return Verdict::Refused;
            }
            return Verdict::Universal;
        }
        let key = param.key.as_deref().unwrap_or("");
        if let Some(breakpoint) = param
            .breakpoint
            .as_deref()
            .filter(|b| !BREAKPOINTS.contains(b))
        {
            self.report(
                Code::F008,
                param_at(param),
                format!("`{breakpoint}` isn't a breakpoint; they are `sm`, `md` and `lg`. The param is ignored"),
            );
            return Verdict::Refused;
        }
        if let Some(declared) = target
            .signature
            .params
            .iter()
            .find(|declared| declared.name == key)
        {
            if param.breakpoint.is_some() && !target.takes_breakpoints() {
                self.no_breakpoint(param, key);
                return Verdict::Refused;
            }
            return match self.declared(target, declared, param, scope) {
                Checked::Given(ty) => Verdict::Declared(ty),
                Checked::Refused => Verdict::Refused,
            };
        }
        match Universal::from_name(key) {
            Some(universal) if target.contents && universal.needs_box() => self.no_box(param),
            Some(universal) if self.universal(universal, param, scope) => {
                return Verdict::Universal;
            }
            Some(_) => {}
            None => {
                let known = target
                    .signature
                    .params
                    .iter()
                    .map(|declared| declared.name.as_str())
                    .chain(Universal::NAMES.iter().copied());
                let hint = nearest(key, known)
                    .map_or_else(String::new, |near| format!("; did you mean `{near}`?"));
                let name = &target.component.name;
                self.report(
                    Code::F008,
                    param_at(param),
                    format!("`{name}` has no param `{key}`, so it is ignored{hint}"),
                );
            }
        }
        Verdict::Refused
    }

    fn no_box(&mut self, param: &Param) {
        self.report(
            Code::W120,
            param_at(param),
            "the component is `display=contents`, so it has no box for this param; ignored",
        );
    }

    fn no_breakpoint(&mut self, param: &Param, key: &str) {
        self.report(
            Code::F008,
            param_at(param),
            format!("`{key}` takes no breakpoint; the param is ignored"),
        );
    }

    /// A declared param's value against its declaration.
    fn declared(
        &mut self,
        target: &Target<'r>,
        declared: &Declaration,
        param: &'r Param,
        scope: &Scope<'r>,
    ) -> Checked {
        let at = value_at(param);
        let key = declared.name.as_str();
        if target.is(Keyword::Box) && param.is_display_contents() {
            self.report(
                Code::F009,
                at,
                "`display=contents` is only for a definition fence, which has the box it removes",
            );
            return Checked::Refused;
        }
        match (&param.value, &declared.declared) {
            (ParamValue::Expr(_), DeclaredType::Name(TypeName::Name)) => self.refuse(
                at,
                &format!("`{key}` binds a name, written as is, not an expression"),
            ),
            (ParamValue::Expr(_), DeclaredType::Input { .. }) => self.refuse(
                at,
                &format!("`{key}` binds an input by its name, not an expression"),
            ),
            (ParamValue::Expr(source), declared_type) => {
                match self.expression(source, at, scope, Some(&Type::declared(declared_type)), key)
                {
                    Some(ty) => Checked::Given(Some(ty)),
                    None => Checked::Refused,
                }
            }
            (value, DeclaredType::Input { input }) => {
                self.bind(target, key, value, input, at, scope)
            }
            (value, declared_type) => {
                if let ParamValue::String(parts) = value {
                    self.string_paths(parts, at, scope);
                }
                let areas = key != "grid-template-areas"
                    || literal(value).is_none_or(|text| Grammar::Areas.accepts(&text));
                if !(fits(declared_type, value) && areas) {
                    return self.refuse(at, &format!("`{key}` takes {}", written(declared_type)));
                }
                if let (ParamValue::Bare(bound), DeclaredType::Name(TypeName::Name)) =
                    (value, declared_type)
                    && let Some(why) = reserved::as_binding(bound)
                {
                    self.report(Code::F013, at, why);
                }
                Checked::Given(None)
            }
        }
    }

    /// F009: the param is treated as absent.
    fn refuse(&mut self, at: Location, message: &str) -> Checked {
        self.report(
            Code::F009,
            at,
            format!("{message}; the param is treated as absent"),
        );
        Checked::Refused
    }

    /// A param of type `input<…>`: the name of a declared input of a type it binds (F030).
    fn bind(
        &mut self,
        target: &Target,
        key: &str,
        value: &ParamValue,
        input_type: &DeclaredType,
        at: Location,
        scope: &Scope,
    ) -> Checked {
        let ParamValue::Bare(name) = value else {
            return self.refuse(at, &format!("`{key}` binds an input by its name"));
        };
        let Some(input) = scope.input(name) else {
            self.report(
                Code::F030,
                at,
                format!("no input `{name}` is declared under `md.inputs` of this document"),
            );
            return Checked::Refused;
        };
        if !binds(input_type, &input.declaration.declared) {
            self.report(
                Code::F030,
                at,
                format!(
                    "`{key}` binds {}, but `{name}` is {}",
                    written(input_type),
                    written(&input.declaration.declared)
                ),
            );
            return Checked::Refused;
        }
        if target.component.body.is_host() {
            self.bound_by_hosts.insert(name.clone());
        }
        Checked::Given(None)
    }

    /// A universal param's value against its grammar, and its breakpoint; whether it is accepted.
    fn universal(&mut self, universal: Universal, param: &'r Param, scope: &Scope<'r>) -> bool {
        let key = universal.name();
        if param.breakpoint.is_some() && !universal.takes_breakpoint() {
            self.no_breakpoint(param, key);
            return false;
        }
        let at = value_at(param);
        let value = &param.value;
        if let ParamValue::Expr(source) = value {
            return self
                .expression(source, at, scope, Some(&Type::String), key)
                .is_some();
        }
        if let ParamValue::String(parts) = value {
            self.string_paths(parts, at, scope);
        }
        let fits = match universal.takes() {
            Takes::Flag => {
                matches!(value, ParamValue::Flag)
                    || matches!(value, ParamValue::Bare(text) if text == "true" || text == "false")
            }
            Takes::Text => matches!(value, ParamValue::Bare(_) | ParamValue::String(_)),
            Takes::Value(grammar) => {
                matches!(value, ParamValue::Bare(_) | ParamValue::String(_))
                    && literal(value).is_none_or(|text| grammar.accepts(&text))
            }
        };
        if !fits {
            self.refuse(at, &format!("`{key}` doesn't take this value"));
        }
        fits
    }

    /// Type-checks an expression written at `at` (its `(`), against what `key` takes: F024 at the column inside
    /// it. Gives its type.
    pub(super) fn expression(
        &mut self,
        source: &str,
        at: Location,
        scope: &Scope,
        expected: Option<&Type>,
        key: &str,
    ) -> Option<Type> {
        let inside = Location::new(at.line, at.column + 1);
        let checked = self
            .parse(source)
            .and_then(|program| program.check(&scope.cel(), &Signatures));
        match checked {
            Err(error) => {
                let at = Location::new(inside.line, inside.column + error.at());
                self.report(Code::F024, at, error.message());
                None
            }
            Ok(ty) => {
                let fits = expected.is_none_or(|expected| {
                    ty.fits(expected) || (*expected == Type::Double && ty.is_numeric())
                });
                if fits {
                    return Some(ty);
                }
                let expected = expected.map(ToString::to_string).unwrap_or_default();
                self.report(
                    Code::F024,
                    inside,
                    format!("`{key}` takes {expected}, but the expression gives {ty}"),
                );
                None
            }
        }
    }

    /// F014 for the interpolations of a quoted param whose `"` is at `at`.
    fn string_paths(&mut self, parts: &[StringPart], at: Location, scope: &Scope) {
        let paths: Vec<&str> = parts
            .iter()
            .filter_map(|part| match part {
                StringPart::Interpolation(interpolation) => Some(interpolation.path.as_str()),
                StringPart::Text(_) => None,
            })
            .collect();
        if paths.is_empty() {
            return;
        }
        let start = Point {
            line: at.line,
            column: at.column,
            offset: 0,
        };
        for (path, at) in paths.into_iter().zip(self.lines.paths_from(start)) {
            self.path(path, at, scope);
        }
    }

    /// F018 for a body given to a component without slots; F006 for an undeclared slot.
    fn slots(&mut self, name: &str, container: &Container, signature: &Signature, start: Point) {
        let Some(slots) = &signature.slots else {
            if !container.children.is_empty() || !container.slots.is_empty() {
                self.report(
                    Code::F018,
                    start.into(),
                    format!("`{name}` declares no slots, so it takes no body; the body follows it as normal content"),
                );
            }
            return;
        };
        let declared: Vec<&str> = slots
            .iter()
            .map(|slot| match slot {
                SlotDeclaration::Name(slot) | SlotDeclaration::Template { name: slot, .. } => {
                    slot.as_str()
                }
            })
            .collect();
        for slot in container
            .slots
            .iter()
            .filter(|slot| !declared.contains(&slot.name.as_str()))
        {
            let hint = nearest(&slot.name, declared.iter().copied())
                .map_or_else(String::new, |near| format!("; did you mean `{near}`?"));
            let at = slot
                .position
                .map_or(start.into(), |position| position.start.into());
            self.report(
                Code::F006,
                at,
                format!(
                    "`{name}` has no slot `{}`, so its content joins the default slot{hint}",
                    slot.name
                ),
            );
        }
    }

    /// A definition fence's params (spec §7, §9.2): the universal params, and on a `compmd` fence `box`'s own;
    /// `display=contents` bare, and then nothing for a box (W120).
    pub(super) fn fence(&mut self, component: &'r Component, scope: &Scope<'r>) {
        let compmd = matches!(component.body, Body::Compmd(_));
        let contents = component.is_contents();
        let resolved = self.resolved;
        let layout = match resolved.document(self.id).names.get("box") {
            Some(Binding::Component(id, _)) if compmd => {
                let layout = resolved.component(*id);
                layout.signature.as_ref().map(|signature| Target {
                    component: layout,
                    signature,
                    contents: false,
                })
            }
            _ => None,
        };
        for param in &component.fence {
            if !self.fence_param(param, compmd, contents, layout.as_ref(), scope) {
                self.refuse_param(param);
            }
        }
    }

    /// One param of a definition fence; whether it is accepted.
    fn fence_param(
        &mut self,
        param: &'r Param,
        compmd: bool,
        contents: bool,
        layout: Option<&Target<'r>>,
        scope: &Scope<'r>,
    ) -> bool {
        if param.is_display_contents() {
            if param.breakpoint.is_some() {
                self.report(
                    Code::F009,
                    value_at(param),
                    "`display=contents` is written bare, with no breakpoint: a component has a box or not",
                );
                return false;
            }
            if compmd {
                return true;
            }
        }
        if contents {
            self.no_box(param);
            return false;
        }
        if matches!(param.value, ParamValue::Class(_) | ParamValue::Id(_)) {
            return true;
        }
        let key = param.key.as_deref().unwrap_or("");
        if let Some(universal) = Universal::from_name(key) {
            return self.universal(universal, param, scope);
        }
        let own = layout.and_then(|layout| {
            let declared = layout
                .signature
                .params
                .iter()
                .find(|declared| declared.name == key)?;
            Some((layout, declared))
        });
        if let Some((layout, declared)) = own {
            return matches!(
                self.declared(layout, declared, param, scope),
                Checked::Given(_)
            );
        }
        let takes = if compmd {
            "the common params, `class`, `id`, `lang`, `dir` and `box`'s params"
        } else {
            "the common params, `class`, `id`, `lang` and `dir`"
        };
        self.report(
            Code::F008,
            param_at(param),
            format!("this fence takes {takes}, not `{key}`; ignored"),
        );
        false
    }
}

/// Whether a default slot is a single list item, which `join=list` would make one list of (W103).
fn is_single_list_item(children: &[Node]) -> bool {
    matches!(children, [Node::List(list)] if list.children.len() == 1)
}

/// The template slots of an invocation: each slot, the name its `as` param binds, and the type of the elements of
/// its `each` param, when it is a list.
fn templates(
    signature: &Signature,
    params: &[Param],
    types: &HashMap<&str, Type>,
) -> Vec<Template> {
    let slots = signature.slots.as_deref().unwrap_or_default();
    slots
        .iter()
        .filter_map(|slot| {
            let SlotDeclaration::Template {
                name,
                each,
                binding,
            } = slot
            else {
                return None;
            };
            let bound = params.iter().find_map(|param| match &param.value {
                ParamValue::Bare(bound) if param.key.as_deref() == Some(binding.as_str()) => {
                    Some(bound.clone())
                }
                _ => None,
            })?;
            let element = match types.get(each.as_str()) {
                Some(Type::List(element)) => (**element).clone(),
                _ => Type::Dyn,
            };
            Some(Template {
                slot: name.clone(),
                bound,
                element,
            })
        })
        .collect()
}

/// A declared type as messages write it.
fn written(declared: &DeclaredType) -> String {
    match declared {
        DeclaredType::Name(name) => format!("a {}", name.name()),
        DeclaredType::Other(name) => format!("a {name}"),
        DeclaredType::Enum { members } => format!("one of {}", members.join(", ")),
        DeclaredType::Input { input } => format!("an input of {}", written(input)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare(text: &str) -> ParamValue {
        ParamValue::Bare(text.into())
    }

    #[test]
    fn kinds_accept_what_their_type_reads() {
        assert!(accepts(TypeName::Number, &bare("3")));
        assert!(!accepts(TypeName::Number, &bare("three")));
        assert!(!accepts(TypeName::Number, &bare("inf")));
        assert!(accepts(TypeName::Boolean, &ParamValue::Flag));
        assert!(accepts(TypeName::Boolean, &bare("false")));
        assert!(accepts(TypeName::Date, &bare("2026-10-21")));
        assert!(!accepts(TypeName::Date, &bare("2026-13-01")));
        assert!(accepts(TypeName::Array, &ParamValue::Json("[1]".into())));
        assert!(!accepts(TypeName::Object, &ParamValue::Json("[1]".into())));
        assert!(accepts(TypeName::Name, &bare("t")));
        assert!(!accepts(TypeName::Name, &bare("a-b")));
        assert!(accepts(TypeName::Space, &bare("m")));
        assert!(!accepts(TypeName::Array, &bare("x")));
        assert!(accepts(TypeName::Enum, &ParamValue::Flag));
        for (name, good, bad) in [
            (TypeName::Length, "2rem", "wide"),
            (TypeName::Color, "#fff", "loud"),
            (TypeName::Tracks, "1fr 2fr", "wide"),
            (TypeName::Surface, "subtle", "loud"),
            (TypeName::Border, "1px solid", "thick-dotted"),
            (TypeName::Radius, "full", "round"),
        ] {
            assert!(
                accepts(name, &bare(good)) && !accepts(name, &bare(bad)),
                "{name:?}"
            );
        }
        assert!(!accepts(TypeName::String, &ParamValue::Flag));
        assert!(accepts(
            TypeName::Number,
            &ParamValue::String(vec![StringPart::Interpolation(
                crate::tree::Interpolation::default()
            )])
        ));
    }

    #[test]
    fn inputs_bind_by_type() {
        let name = DeclaredType::named;
        let members = |m: &[&str]| DeclaredType::Enum {
            members: m.iter().map(|s| (*s).to_string()).collect(),
        };
        assert!(binds(&name("enum"), &members(&["a", "b"])));
        assert!(binds(&members(&["a"]), &members(&["a"])));
        assert!(!binds(&members(&["a"]), &members(&["b"])));
        assert!(binds(&name("date"), &name("date")));
        assert!(!binds(&name("date"), &members(&["a"])));
        assert!(!binds(&members(&["a"]), &name("string")));
        assert!(binds(&name("odd"), &name("odd")));
        assert_eq!(written(&name("odd")), "a odd");
        let input = DeclaredType::Input {
            input: Box::new(name("date")),
        };
        assert!(fits(&input, &ParamValue::Flag));
        assert!(!fits(&members(&["a"]), &ParamValue::Flag));
        assert!(fits(&name("unknown-type"), &ParamValue::Flag));
        assert_eq!(
            written(&DeclaredType::Input {
                input: Box::new(name("date"))
            }),
            "an input of a date"
        );
    }
}
