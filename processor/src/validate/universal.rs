//! The params every component takes without declaring them: the common params, which style its box (spec §9.2),
//! and `class`, `id`, `lang` and `dir` (spec §5.1).

use super::grammar::Grammar;
use crate::named::named;

named! {
    /// A param every component takes.
    pub(crate) enum Universal {
        Width = "width",
        Height = "height",
        MinWidth = "min-width",
        MaxWidth = "max-width",
        Padding = "padding",
        PaddingInline = "padding-inline",
        PaddingBlock = "padding-block",
        PaddingInlineStart = "padding-inline-start",
        PaddingInlineEnd = "padding-inline-end",
        PaddingBlockStart = "padding-block-start",
        PaddingBlockEnd = "padding-block-end",
        Border = "border",
        BorderRadius = "border-radius",
        Background = "background",
        GridArea = "grid-area",
        Order = "order",
        AlignSelf = "align-self",
        JustifySelf = "justify-self",
        AspectRatio = "aspect-ratio",
        InsetInlineStart = "inset-inline-start",
        InsetInlineEnd = "inset-inline-end",
        InsetBlockStart = "inset-block-start",
        InsetBlockEnd = "inset-block-end",
        Scroll = "scroll",
        Fill = "fill",
        Sticky = "sticky",
        Show = "show",
        Hide = "hide",
        Class = "class",
        Id = "id",
        Lang = "lang",
        Dir = "dir",
    }
}

/// What a universal param's value is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Takes {
    /// A value in this grammar.
    Value(Grammar),
    /// A flag, written without a value.
    Flag,
    /// Any text.
    Text,
}

impl Universal {
    /// What its value is.
    pub(crate) fn takes(self) -> Takes {
        use Grammar::{AreaName, Keywords, Language, Ratio, Size, Space, Spaces};
        match self {
            Self::Width
            | Self::Height
            | Self::MinWidth
            | Self::MaxWidth
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd => Takes::Value(Size),
            Self::Padding | Self::PaddingInline | Self::PaddingBlock => Takes::Value(Spaces),
            Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd => Takes::Value(Space),
            Self::Border => Takes::Value(Grammar::Border),
            Self::BorderRadius => Takes::Value(Grammar::Radius),
            Self::Background => Takes::Value(Grammar::Surface),
            Self::GridArea => Takes::Value(AreaName),
            Self::Order => Takes::Value(Keywords(&["first", "last", "none"])),
            Self::AlignSelf => Takes::Value(Keywords(&["start", "center", "end", "stretch"])),
            Self::JustifySelf => Takes::Value(Keywords(&["start", "center", "end"])),
            Self::AspectRatio => Takes::Value(Ratio),
            Self::Scroll | Self::Fill => Takes::Flag,
            Self::Sticky => Takes::Value(Keywords(&["top", "bottom", "none"])),
            Self::Show | Self::Hide => Takes::Value(Keywords(&["sm", "md", "lg"])),
            Self::Class | Self::Id => Takes::Text,
            Self::Lang => Takes::Value(Language),
            Self::Dir => Takes::Value(Keywords(&["ltr", "rtl", "auto"])),
        }
    }

    /// Whether it may carry a breakpoint (spec §9.3).
    pub(crate) fn takes_breakpoint(self) -> bool {
        !matches!(
            self,
            Self::GridArea
                | Self::Scroll
                | Self::Fill
                | Self::Show
                | Self::Hide
                | Self::Class
                | Self::Id
                | Self::Lang
                | Self::Dir
        )
    }

    /// Whether it applies to the component's box, which a `display=contents` component doesn't have (W120).
    pub(crate) fn needs_box(self) -> bool {
        !matches!(self, Self::Lang | Self::Dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_param_has_its_grammar_and_breakpoint_rule() {
        assert_eq!(
            Universal::from_name("padding-inline").map(Universal::takes),
            Some(Takes::Value(Grammar::Spaces))
        );
        assert_eq!(Universal::Scroll.takes(), Takes::Flag);
        assert_eq!(Universal::Id.takes(), Takes::Text);
        assert!(Universal::Width.takes_breakpoint());
        assert!(!Universal::Show.takes_breakpoint());
        assert!(Universal::Class.needs_box());
        assert!(!Universal::Dir.needs_box());
    }
}
