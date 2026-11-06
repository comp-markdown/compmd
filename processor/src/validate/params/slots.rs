//! An invocation's slots against its component's signature (spec §7.5): F018 for a body given to a component without
//! slots, and F006 for an undeclared slot or for content before the first slot line with no default slot.

use crate::diagnostic::{Code, Fix, Location};
use crate::nearest::nearest;
use crate::tree::{Container, Point, Signature, SlotDeclaration};
use crate::validate::Checker;

impl Checker<'_, '_> {
    /// F018 for a body given to a component without slots; F006 for an undeclared slot, and for content before the
    /// first slot line of a component without a default slot.
    pub(super) fn slots(
        &mut self,
        name: &str,
        container: &Container,
        signature: &Signature,
        start: Point,
    ) {
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
        if !signature.declares_slot("default") {
            self.no_default_slot(name, container, start);
        }
        let declared: Vec<&str> = slots.iter().map(SlotDeclaration::name).collect();
        for slot in container
            .slots
            .iter()
            .filter(|slot| !signature.declares_slot(&slot.name))
        {
            let near = nearest(&slot.name, declared.iter().copied());
            let hint = near.map_or_else(String::new, |near| format!("; did you mean `{near}`?"));
            let at = slot
                .position
                .map_or(start.into(), |position| position.start.into());
            // The slot's name follows the `#` that `name_at` finds past the colons.
            let fix = near.zip(slot.position).map(|(near, position)| {
                let hash = self.lines.name_at(position.start);
                Fix::replacing(Location::new(hash.line, hash.column + 1), &slot.name, near)
            });
            self.report_fixed(
                Code::F006,
                at,
                format!(
                    "`{name}` has no slot `{}`, so its content joins the default slot{hint}",
                    slot.name
                ),
                fix,
            );
        }
    }

    /// F006 at the content before the first slot line of a component that declares no default slot: the spec doesn't
    /// say what becomes of it, so this processor shows it after the component, as normal content
    /// (`processor/README.md`).
    fn no_default_slot(&mut self, name: &str, container: &Container, start: Point) {
        let Some(first) = container.children.first() else {
            return;
        };
        let at = first
            .position()
            .map_or(start.into(), |position| position.start.into());
        self.report(
            Code::F006,
            at,
            format!("`{name}` has no default slot, so the content before its first slot line follows it as normal content"),
        );
    }
}
