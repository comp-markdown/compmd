//! A definition fence's params (spec §7, §9.2): the universal params, and on a `compmd` fence `box`'s own;
//! `display=contents` bare, and then nothing for a box (W120).

use super::{Target, Verdict, param_at, value_at};
use crate::diagnostic::Code;
use crate::resolve::{Binding, Body, Component};
use crate::tree::{Param, ParamValue};
use crate::validate::Checker;
use crate::validate::scope::Scope;
use crate::validate::universal::Universal;

impl<'r> Checker<'r, '_> {
    /// A definition fence's params (spec §7, §9.2): the universal params, and on a `compmd` fence `box`'s own;
    /// `display=contents` bare, and then nothing for a box (W120).
    pub(in crate::validate) fn fence(&mut self, component: &'r Component, scope: &Scope<'r>) {
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
                Verdict::Declared(_)
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
