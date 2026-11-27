//! The six primitives ([a document drawn in the editor], decision 2), and a set of them: what a client draws.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

/// One thing an editor can draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Primitive {
    /// A range not shown.
    Hide,
    /// A range shown as a text.
    Replace,
    /// A text at a position, shifting what follows.
    Insert,
    /// A line of its own, above or below a line.
    Line,
    /// A text at the start of each of some lines.
    Prefix,
    /// A range in a style.
    Style,
}

impl Primitive {
    /// Every primitive.
    pub(crate) const ALL: [Self; 6] = [
        Self::Hide,
        Self::Replace,
        Self::Insert,
        Self::Line,
        Self::Prefix,
        Self::Style,
    ];

    /// Its name, as a client declares it and a mark's `kind` gives it.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Hide => "hide",
            Self::Replace => "replace",
            Self::Insert => "insert",
            Self::Line => "line",
            Self::Prefix => "prefix",
            Self::Style => "style",
        }
    }

    /// The primitive `name` names; `None` for a name that is no primitive.
    pub(crate) fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|primitive| primitive.name() == name)
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of primitives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Primitives(u8);

impl Primitives {
    /// Whether the set holds `primitive`.
    pub(crate) const fn contains(self, primitive: Primitive) -> bool {
        self.0 & primitive.bit() != 0
    }
}

impl FromIterator<Primitive> for Primitives {
    fn from_iter<I: IntoIterator<Item = Primitive>>(primitives: I) -> Self {
        Self(
            primitives
                .into_iter()
                .fold(0, |set, primitive| set | primitive.bit()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_primitive_is_known_by_its_name() {
        for primitive in Primitive::ALL {
            assert_eq!(Primitive::named(primitive.name()), Some(primitive));
        }
        assert_eq!(Primitive::named("image"), None);
    }

    #[test]
    fn a_set_holds_what_it_is_made_of() {
        let set: Primitives = [Primitive::Hide, Primitive::Style].into_iter().collect();
        assert!(set.contains(Primitive::Hide) && set.contains(Primitive::Style));
        assert!(!set.contains(Primitive::Line));
        assert!(!Primitives::default().contains(Primitive::Hide));
    }
}
