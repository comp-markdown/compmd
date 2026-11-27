//! What the last render evaluated in the processed document (spec §8.1, §8.2): each interpolation and each expression
//! param written in it, with the value it gave and how many times it was evaluated, for an editor that shows a value
//! beside what gave it.
//!
//! Only what is written in the processed document is kept, in its body and in its own definitions; not what an
//! imported document or an example (spec §7.8) evaluates, nor the interpolations of link and image destinations or a
//! declared param's default.

use crate::cel::Value;
use crate::diagnostic::Location;
use crate::tree::Moved;
use std::collections::HashMap;

/// An interpolation or an expression param of the processed document, as the last render evaluated it.
#[derive(Clone, Debug)]
pub struct Evaluation {
    /// Where it is written: an interpolation's path, past `{{` and the spaces after it; an expression param's `(`.
    pub at: Location,
    /// The value its first evaluation gave; `None` when one of its evaluations failed, or gave an interpolation a
    /// value it can't write (F024 says why).
    pub value: Option<Value>,
    /// How many times the render evaluated it: once in the document's body, and once more for each iteration of an
    /// `each` or a template slot around it, each invocation of the definition it is in, and each breakpoint a
    /// responsive body evaluates an expression at (spec §9.3).
    pub count: usize,
}

/// The evaluations of a part of a render, by where each is written.
#[derive(Clone, Debug, Default)]
pub(crate) struct Evaluations(HashMap<Location, Evaluation>);

impl Evaluations {
    /// One more evaluation of what is written `at`: the value it gave, `None` when it failed.
    pub(crate) fn record(&mut self, at: Location, value: Option<&Value>) {
        self.add(Evaluation {
            at,
            value: value.cloned(),
            count: 1,
        });
    }

    /// `evaluation` added to what is written at its place: the first value is kept, a failure kept as one.
    fn add(&mut self, evaluation: Evaluation) {
        let Some(known) = self.0.get_mut(&evaluation.at) else {
            self.0.insert(evaluation.at, evaluation);
            return;
        };
        known.count += evaluation.count;
        if evaluation.value.is_none() {
            known.value = None;
        }
    }

    /// The evaluations of `later`, a part of the render after this one, added.
    pub(crate) fn extend(&mut self, later: &Self) {
        for evaluation in later.0.values() {
            self.add(evaluation.clone());
        }
    }

    /// Moved, as an edit before them moved the processed document.
    pub(crate) fn shift(&mut self, moved: Moved) {
        self.0 = std::mem::take(&mut self.0)
            .into_values()
            .map(|mut evaluation| {
                moved.location(&mut evaluation.at);
                (evaluation.at, evaluation)
            })
            .collect();
    }

    /// The evaluations, in the order they are written.
    pub(crate) fn in_order(&self) -> Vec<Evaluation> {
        let mut evaluations: Vec<Evaluation> = self.0.values().cloned().collect();
        evaluations.sort_by_key(|evaluation| evaluation.at);
        evaluations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Move, Point};

    fn written(evaluations: &Evaluations) -> Vec<(usize, Option<String>, usize)> {
        evaluations
            .in_order()
            .into_iter()
            .map(|evaluation| {
                let value = evaluation.value.map(|value| value.to_json().to_string());
                (evaluation.at.line, value, evaluation.count)
            })
            .collect()
    }

    #[test]
    fn the_first_value_is_kept_and_the_evaluations_counted() {
        let mut evaluations = Evaluations::default();
        evaluations.record(Location::new(3, 1), Some(&Value::string("Ada")));
        evaluations.record(Location::new(3, 1), Some(&Value::string("Kenji")));
        evaluations.record(Location::new(1, 4), Some(&Value::Bool(true)));
        assert_eq!(
            written(&evaluations),
            [(1, Some("true".into()), 1), (3, Some("\"Ada\"".into()), 2)]
        );
    }

    #[test]
    fn one_failure_fails_it_whichever_part_saw_it() {
        let mut first = Evaluations::default();
        first.record(Location::new(2, 1), Some(&Value::Null));
        first.record(Location::new(5, 1), None);
        let mut later = Evaluations::default();
        later.record(Location::new(2, 1), None);
        later.record(Location::new(5, 1), Some(&Value::Null));
        later.record(Location::new(7, 1), Some(&Value::Null));
        first.extend(&later);
        assert_eq!(
            written(&first),
            [(2, None, 2), (5, None, 2), (7, Some("null".into()), 1)]
        );
    }

    #[test]
    fn an_edit_moves_those_on_its_line_and_after() {
        let mut evaluations = Evaluations::default();
        evaluations.record(Location::new(1, 1), Some(&Value::Null));
        evaluations.record(Location::new(4, 1), Some(&Value::Null));
        let moved = Moved {
            from: Point {
                line: 2,
                column: 1,
                offset: 10,
            },
            by: Move {
                lines: 2,
                offset: 5,
            },
        };
        evaluations.shift(moved);
        let lines: Vec<usize> = written(&evaluations)
            .iter()
            .map(|(line, ..)| *line)
            .collect();
        assert_eq!(lines, [1, 6]);
    }
}
