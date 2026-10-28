//! Params evaluated (spec §9.2, §9.3, §12.3): each value read by its kind and declared type, a bound input's value
//! with its name, an expression evaluated at each breakpoint where the body's params have breakpoint values, a
//! definition fence's params under the caller's, and all classes in one `class`.

use super::Expander;
use super::env::Env;
use crate::cel::{Error, Value};
use crate::diagnostic::{Code, Location};
use crate::expanded::{self, json};
use crate::tree::{DeclaredType, Param, ParamValue, Point, Signature, StringPart};
use crate::validate::params::value_at;

/// The breakpoints, narrowest first (spec §9.3).
pub(super) const BREAKPOINTS: [&str; 3] = ["sm", "md", "lg"];

/// An evaluated param.
#[derive(Clone, Debug)]
pub(super) struct Out {
    pub(super) key: String,
    pub(super) breakpoint: Option<String>,
    pub(super) value: Value,
    /// The input it is bound to.
    pub(super) input: Option<String>,
}

impl Out {
    fn into_param(self) -> expanded::Param {
        expanded::Param {
            key: self.key,
            breakpoint: self.breakpoint,
            value: json(&self.value),
            input: self.input,
        }
    }
}

/// A param's key: `class` for `.name`, `id` for `#name`.
pub(super) fn key_of(param: &Param) -> &str {
    match param.value {
        ParamValue::Class(_) => "class",
        ParamValue::Id(_) => "id",
        _ => param.key.as_deref().unwrap_or(""),
    }
}

impl<'r> Expander<'r> {
    /// The value of `param`, written in `env`, read as `declared`; `None` when its expression fails (F024), so that
    /// it is treated as absent.
    pub(super) fn param_value(
        &mut self,
        param: &Param,
        declared: Option<&DeclaredType>,
        env: &Env<'r>,
    ) -> Option<Value> {
        Some(match &param.value {
            ParamValue::Bare(text) => Value::bare(text, declared),
            ParamValue::String(parts) => Value::string(&self.quoted(parts, param, env)),
            ParamValue::Json(text) => Value::from_json(&serde_json::from_str(text).ok()?),
            ParamValue::Expr(source) => match self.evaluate(source, env) {
                Ok(value) => value,
                Err(error) => {
                    self.failed(&error, source, param, env);
                    return None;
                }
            },
            ParamValue::Flag => Value::Bool(true),
            ParamValue::Class(text) | ParamValue::Id(text) => Value::string(text),
        })
    }

    /// F024 at the column inside the expression.
    fn failed(&mut self, error: &Error, source: &str, param: &Param, env: &Env) {
        let at = value_at(param);
        let at = Location::new(at.line, at.column + 1 + error.at());
        self.report(
            Code::F024,
            env,
            at,
            format!("`{source}`: {}", error.message()),
        );
    }

    /// A quoted value's text, its interpolations filled in.
    fn quoted(&mut self, parts: &[StringPart], param: &Param, env: &Env<'r>) -> String {
        let at = value_at(param);
        let start = Point {
            line: at.line,
            column: at.column,
            offset: 0,
        };
        let mut places = self.lines[&env.document].paths_from(start).into_iter();
        let mut text = String::new();
        for part in parts {
            match part {
                StringPart::Text(literal) => text.push_str(&literal.value),
                StringPart::Interpolation(interpolation) => {
                    let at = places.next().unwrap_or(at);
                    text.push_str(
                        &self
                            .path_text(&interpolation.path, at, env)
                            .unwrap_or_default(),
                    );
                }
            }
        }
        text
    }

    /// The params `params` give a node, evaluated in `env` and read by `signature`, in written order. A bound input
    /// gives its value and name. An expression without a breakpoint, in a body whose params have breakpoint values,
    /// is evaluated at each breakpoint, and each value that differs from the one before becomes a param of that
    /// breakpoint (spec §9.3). A param whose expression fails, or whose value is `null`, is left out.
    pub(super) fn evaluated(
        &mut self,
        params: &[&'r Param],
        signature: Option<&Signature>,
        env: &Env<'r>,
    ) -> Vec<Out> {
        let mut out = Vec::new();
        for param in params {
            let key = key_of(param).to_string();
            let declared = signature
                .and_then(|signature| signature.params.iter().find(|d| d.name == key))
                .map(|declaration| &declaration.declared);
            if let (Some(DeclaredType::Input { .. }), ParamValue::Bare(input)) =
                (declared, &param.value)
            {
                out.push(Out {
                    value: env.input(input),
                    key,
                    breakpoint: param.breakpoint.clone(),
                    input: Some(input.clone()),
                });
                continue;
            }
            let Some(value) = self.param_value(param, declared, env) else {
                continue;
            };
            let mut previous = json(&value);
            out.push(Out {
                key: key.clone(),
                breakpoint: param.breakpoint.clone(),
                value,
                input: None,
            });
            let responsive =
                matches!(param.value, ParamValue::Expr(_)) && param.breakpoint.is_none();
            let Some(values) = env.breakpoints.clone().filter(|_| responsive) else {
                continue;
            };
            for (values, breakpoint) in values.iter().zip(BREAKPOINTS) {
                let at = Env {
                    values: values.clone(),
                    ..env.clone()
                };
                let Some(value) = self.param_value(param, declared, &at) else {
                    continue;
                };
                let written = json(&value);
                if written != previous {
                    previous = written;
                    out.push(Out {
                        key: key.clone(),
                        breakpoint: Some(breakpoint.to_string()),
                        value,
                        input: None,
                    });
                }
            }
        }
        out.retain(|out| !matches!(out.value, Value::Null));
        out
    }
}

/// The params of a component's box: the fence's, less what the caller's cover (spec §9.2), then the caller's, with
/// all classes in one `class`, last.
pub(super) fn boxed(fence: Vec<Out>, caller: Vec<Out>) -> Vec<expanded::Param> {
    let mut params = Vec::new();
    for out in fence {
        under(out, &caller, &mut params);
    }
    params.extend(caller);
    let mut classes: Vec<String> = Vec::new();
    for out in params.iter().filter(|out| out.key == "class") {
        let names = out.value.written().ok().flatten().unwrap_or_default();
        for name in names.split_whitespace() {
            if !classes.iter().any(|known| known == name) {
                classes.push(name.to_string());
            }
        }
    }
    params.retain(|out| out.key != "class");
    if !classes.is_empty() {
        params.push(Out {
            key: "class".into(),
            breakpoint: None,
            value: Value::string(&classes.join(" ")),
            input: None,
        });
    }
    params.into_iter().map(Out::into_param).collect()
}

/// A fence param kept unless a caller's param covers it from its breakpoint or a wider one; a shorthand the caller
/// covers in part is written as its longhands first.
fn under(fence: Out, caller: &[Out], kept: &mut Vec<Out>) {
    if fence.key == "class" {
        kept.push(fence);
        return;
    }
    let from_here =
        |out: &&Out| rank(out.breakpoint.as_deref()) <= rank(fence.breakpoint.as_deref());
    if caller
        .iter()
        .filter(from_here)
        .any(|out| covers(&out.key, &fence.key))
    {
        return;
    }
    let in_part = caller
        .iter()
        .filter(from_here)
        .any(|out| out.key != fence.key && covers(&fence.key, &out.key));
    match longhands(&fence).filter(|_| in_part) {
        Some(parts) => {
            for part in parts {
                under(part, caller, kept);
            }
        }
        None => kept.push(fence),
    }
}

/// Where a breakpoint is among the breakpoints: none, the widest range, first.
fn rank(breakpoint: Option<&str>) -> usize {
    breakpoint.map_or(0, |breakpoint| {
        BREAKPOINTS
            .iter()
            .position(|known| *known == breakpoint)
            .map_or(0, |index| index + 1)
    })
}

/// The parts a shorthand sets: `padding` its block and inline padding, those their two sides.
fn parts(key: &str) -> &'static [&'static str] {
    match key {
        "padding" => &["padding-block", "padding-inline"],
        "padding-block" => &["padding-block-start", "padding-block-end"],
        "padding-inline" => &["padding-inline-start", "padding-inline-end"],
        _ => &[],
    }
}

/// Whether a param of `key` sets the property `other`: it is the same, or one of its parts.
fn covers(key: &str, other: &str) -> bool {
    key == other || parts(key).iter().any(|part| covers(part, other))
}

/// A shorthand's longhands, its one or two values split between them; `None` for a param that isn't one.
fn longhands(out: &Out) -> Option<Vec<Out>> {
    let names = parts(&out.key);
    let text = out.value.written().ok().flatten()?;
    let values: Vec<&str> = text.split_whitespace().collect();
    let (first, second) = match values.as_slice() {
        [one] => (*one, *one),
        [first, second] => (*first, *second),
        _ => return None,
    };
    let [one, two] = names else {
        return None;
    };
    Some(
        [(one, first), (two, second)]
            .into_iter()
            .map(|(name, value)| Out {
                key: (*name).to_string(),
                breakpoint: out.breakpoint.clone(),
                value: Value::string(value),
                input: None,
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn out(key: &str, breakpoint: Option<&str>, value: &str) -> Out {
        Out {
            key: key.into(),
            breakpoint: breakpoint.map(Into::into),
            value: Value::string(value),
            input: None,
        }
    }

    fn written(params: &[expanded::Param]) -> Vec<String> {
        params
            .iter()
            .map(|param| match &param.breakpoint {
                Some(breakpoint) => format!("{breakpoint}:{}={}", param.key, param.value),
                None => format!("{}={}", param.key, param.value),
            })
            .collect()
    }

    #[test]
    fn the_callers_params_cover_the_fences_from_their_breakpoint_up() {
        let fence = || {
            vec![
                out("class", None, "card"),
                out("padding", None, "s"),
                out("border", None, "subtle"),
                out("padding", Some("md"), "m"),
            ]
        };
        let wide = boxed(
            fence(),
            vec![out("padding", None, "l"), out("class", None, "wide card")],
        );
        assert_eq!(
            written(&wide),
            ["border=\"subtle\"", "padding=\"l\"", "class=\"card wide\""]
        );
        let from_md = boxed(fence(), vec![out("padding", Some("md"), "l")]);
        assert_eq!(
            written(&from_md),
            [
                "padding=\"s\"",
                "border=\"subtle\"",
                "md:padding=\"l\"",
                "class=\"card\""
            ]
        );
    }

    #[test]
    fn a_shorthand_covered_in_part_is_split() {
        let split = boxed(
            vec![out("padding", None, "s l")],
            vec![out("padding-inline-start", None, "m")],
        );
        assert_eq!(
            written(&split),
            [
                "padding-block=\"s\"",
                "padding-inline-end=\"l\"",
                "padding-inline-start=\"m\""
            ]
        );
        let covered = boxed(
            vec![out("padding-inline", None, "l")],
            vec![out("padding", None, "s")],
        );
        assert_eq!(written(&covered), ["padding=\"s\""]);
        let odd = boxed(
            vec![out("padding", None, "s m l")],
            vec![out("padding-block", None, "m")],
        );
        assert_eq!(written(&odd), ["padding=\"s m l\"", "padding-block=\"m\""]);
        assert!(longhands(&out("border", None, "s")).is_none());
    }

    #[test]
    fn whole_numbers_are_written_without_a_fraction() {
        let number = Out {
            key: "n".into(),
            breakpoint: None,
            value: Value::Double(5.0),
            input: Some("limit".into()),
        };
        let param = number.into_param();
        assert_eq!(serde_json::to_value(&param).unwrap()["value"], json!(5));
        assert_eq!(rank(Some("xl")), 0);
    }
}
