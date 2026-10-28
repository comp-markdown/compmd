//! The breakpoints (spec §9.3): a layout param's value from a width up.

/// The breakpoints, narrowest first.
pub(crate) const BREAKPOINTS: [&str; 3] = ["sm", "md", "lg"];

/// How wide a breakpoint is: 0 without one, then 1 for `sm` up to 3 for `lg`; 0 for a name that isn't one.
pub(crate) fn rank(breakpoint: Option<&str>) -> usize {
    breakpoint
        .and_then(|breakpoint| BREAKPOINTS.iter().position(|known| *known == breakpoint))
        .map_or(0, |index| index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_rank_from_narrow_to_wide() {
        assert_eq!(rank(None), 0);
        assert_eq!(rank(Some("sm")), 1);
        assert_eq!(rank(Some("lg")), 3);
        assert_eq!(rank(Some("xl")), 0);
    }
}
