//! What expanding may spend (spec §8.5): the nodes it makes, and the work it does, iterations of `each` and template
//! slots and invocations, so an expansion that makes no nodes still ends. The first node or unit of work past a limit
//! is reported at the expansion under way, F034, which then renders its fallback (spec §10.3).

use super::env::Env;
use super::{Expander, Options, Report};
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::{self, Content, Origin};
use std::ops;

/// What an expansion spent of its limits (spec §8.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Used {
    /// The nodes it made.
    pub(crate) made: usize,
    /// The iterations and invocations it did.
    pub(crate) worked: usize,
}

impl ops::Add for Used {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            made: self.made + other.made,
            worked: self.worked + other.worked,
        }
    }
}

impl Options {
    /// These options with what `used` spent taken from their limits: for what expands after it and counts toward the
    /// same limits, as a document after its examples (spec §8.5).
    #[must_use]
    pub(crate) fn after(&self, used: Used) -> Self {
        Self {
            limit: self.limit.saturating_sub(used.made),
            work: self.work.saturating_sub(used.worked),
            ..self.clone()
        }
    }
}

impl Expander<'_> {
    /// What expanding has spent so far.
    pub(super) fn used(&self) -> Used {
        Used {
            made: self.made,
            worked: self.worked,
        }
    }

    /// A node made, counted against the node limit.
    pub(super) fn made(&mut self, content: Content, origin: Option<Origin>) -> expanded::Node {
        self.made += 1;
        if self.made > self.options.limit {
            let limit = self.options.limit;
            self.cross(format!(
                "the expanded document would have more than {limit} nodes; this renders its fallback"
            ));
        }
        expanded::Node { content, origin }
    }

    /// An iteration or an invocation done, counted against the work limit.
    fn worked(&mut self) {
        self.worked += 1;
        if self.worked > self.options.work {
            let work = self.options.work;
            self.cross(format!(
                "the expansion would take more than {work} iterations and invocations; this renders its fallback"
            ));
        }
    }

    /// The first crossing of a limit, reported at the expansion under way; outside one, nothing stops.
    fn cross(&mut self, message: String) {
        if self.crossed.is_some() {
            return;
        }
        let Some((id, at)) = self.expanding.last().copied() else {
            return;
        };
        self.crossed = Some(self.expanding.len());
        self.reports.push(Report {
            once: None,
            diagnostic: Diagnostic::new(Code::F034, at, message)
                .in_stage(Stage::Evaluate)
                .in_file(self.resolved.file(id)),
        });
    }

    /// Whether a limit has been crossed.
    pub(super) fn over_limit(&self) -> bool {
        self.crossed.is_some()
    }

    /// Runs `expansion`, an `each`, a template slot or an invocation written at `at` in `env`'s document, counted as
    /// work. `None` when it renders its fallback instead (spec §10.3): it crossed a limit itself, or starts once one
    /// is crossed. One under way when an inner one crossed it keeps what it made.
    pub(super) fn expanding<T>(
        &mut self,
        env: &Env,
        at: Location,
        expansion: impl FnOnce(&mut Self) -> T,
    ) -> Option<T> {
        if self.over_limit() {
            return None;
        }
        self.expanding.push((env.document, at));
        let depth = self.expanding.len();
        self.worked();
        let result = (!self.over_limit()).then(|| expansion(self));
        self.expanding.pop();
        result.filter(|_| self.crossed != Some(depth))
    }

    /// `iteration` for each index below `count`, each counted as work, until a limit is crossed.
    pub(super) fn iterate<T>(
        &mut self,
        count: usize,
        mut iteration: impl FnMut(&mut Self, usize) -> T,
    ) -> Vec<T> {
        let mut done = Vec::new();
        for index in 0..count {
            self.worked();
            if self.over_limit() {
                break;
            }
            done.push(iteration(self, index));
        }
        done
    }
}
