//! The params every component takes without declaring them: the common params, which style its box (spec §9.2),
//! and `class`, `id`, `lang` and `dir` (spec §5.1).

use crate::named::named;
use crate::values::Kind;

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
        BorderInlineStart = "border-inline-start",
        BorderInlineEnd = "border-inline-end",
        BorderBlockStart = "border-block-start",
        BorderBlockEnd = "border-block-end",
        BorderRadius = "border-radius",
        Shadow = "shadow",
        Rotate = "rotate",
        Background = "background",
        FontSize = "font-size",
        FontWeight = "font-weight",
        FontStyle = "font-style",
        FontFamily = "font-family",
        Color = "color",
        TextAlign = "text-align",
        LineHeight = "line-height",
        LetterSpacing = "letter-spacing",
        TextTransform = "text-transform",
        TextDecoration = "text-decoration",
        GridArea = "grid-area",
        Order = "order",
        AlignSelf = "align-self",
        JustifySelf = "justify-self",
        AspectRatio = "aspect-ratio",
        ObjectFit = "object-fit",
        ObjectPosition = "object-position",
        InsetInlineStart = "inset-inline-start",
        InsetInlineEnd = "inset-inline-end",
        InsetBlockStart = "inset-block-start",
        InsetBlockEnd = "inset-block-end",
        Scroll = "scroll",
        Fill = "fill",
        Sticky = "sticky",
        StretchLink = "stretch-link",
        Show = "show",
        Hide = "hide",
        Class = "class",
        Id = "id",
        Lang = "lang",
        Dir = "dir",
    }
}

// Each question about a param is a `match` over every one, so a new param doesn't compile until it has every answer.
impl Universal {
    /// What its value is.
    pub(crate) fn kind(self) -> Kind<'static> {
        match self {
            Self::Width
            | Self::Height
            | Self::MinWidth
            | Self::MaxWidth
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd => Kind::SIZE,
            Self::Padding | Self::PaddingInline | Self::PaddingBlock => Kind::SPACES,
            Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd => Kind::SPACE,
            Self::Border
            | Self::BorderInlineStart
            | Self::BorderInlineEnd
            | Self::BorderBlockStart
            | Self::BorderBlockEnd => Kind::BORDER,
            Self::BorderRadius => Kind::RADIUS,
            Self::Shadow => Kind::SHADOW,
            Self::Rotate => Kind::ANGLE,
            Self::Background => Kind::SURFACE,
            Self::FontSize => Kind::FONT_SIZE,
            Self::FontWeight => Kind::one_of(&["normal", "bold"][..]),
            Self::FontStyle => Kind::one_of(&["normal", "italic"][..]),
            Self::FontFamily => Kind::one_of(&["body", "heading", "mono"][..]),
            Self::Color => Kind::TEXT_COLOR,
            Self::TextAlign | Self::JustifySelf => Kind::one_of(&["start", "center", "end"][..]),
            Self::LineHeight => Kind::LINE_HEIGHT,
            Self::LetterSpacing => Kind::LETTER_SPACING,
            Self::TextTransform => {
                Kind::one_of(&["none", "uppercase", "lowercase", "capitalize"][..])
            }
            Self::TextDecoration => Kind::one_of(&["none", "underline", "line-through"][..]),
            Self::GridArea => Kind::AREA_NAME,
            Self::Order => Kind::one_of(&["first", "last", "none"][..]),
            Self::AlignSelf => Kind::one_of(&["start", "center", "end", "stretch"][..]),
            Self::AspectRatio => Kind::RATIO,
            Self::ObjectFit => Kind::one_of(&["contain", "cover", "fill"][..]),
            Self::ObjectPosition => Kind::POSITIONS,
            Self::Scroll | Self::Fill | Self::StretchLink => Kind::BOOLEAN,
            Self::Sticky => Kind::one_of(&["top", "bottom", "none"][..]),
            Self::Show | Self::Hide => Kind::one_of(&["sm", "md", "lg"][..]),
            Self::Class | Self::Id => Kind::TEXT,
            Self::Lang => Kind::LANGUAGE,
            Self::Dir => Kind::one_of(&["ltr", "rtl", "auto"][..]),
        }
    }

    /// Whether it may carry a breakpoint (spec §9.3). `object-fit` and `object-position` reach the component as
    /// values (spec §7.4), which a breakpoint, CSS's to resolve, isn't.
    pub(crate) fn takes_breakpoint(self) -> bool {
        match self {
            Self::GridArea
            | Self::ObjectFit
            | Self::ObjectPosition
            | Self::Scroll
            | Self::Fill
            | Self::StretchLink
            | Self::Show
            | Self::Hide
            | Self::Class
            | Self::Id
            | Self::Lang
            | Self::Dir => false,
            Self::Width
            | Self::Height
            | Self::MinWidth
            | Self::MaxWidth
            | Self::Padding
            | Self::PaddingInline
            | Self::PaddingBlock
            | Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd
            | Self::Border
            | Self::BorderInlineStart
            | Self::BorderInlineEnd
            | Self::BorderBlockStart
            | Self::BorderBlockEnd
            | Self::BorderRadius
            | Self::Shadow
            | Self::Rotate
            | Self::Background
            | Self::FontSize
            | Self::FontWeight
            | Self::FontStyle
            | Self::FontFamily
            | Self::Color
            | Self::TextAlign
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::TextTransform
            | Self::TextDecoration
            | Self::Order
            | Self::AlignSelf
            | Self::JustifySelf
            | Self::AspectRatio
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd
            | Self::Sticky => true,
        }
    }

    /// Whether it applies to the component's box, which a `display=contents` component doesn't have (W120).
    pub(crate) fn needs_box(self) -> bool {
        match self {
            Self::Lang | Self::Dir => false,
            Self::Width
            | Self::Height
            | Self::MinWidth
            | Self::MaxWidth
            | Self::Padding
            | Self::PaddingInline
            | Self::PaddingBlock
            | Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd
            | Self::Border
            | Self::BorderInlineStart
            | Self::BorderInlineEnd
            | Self::BorderBlockStart
            | Self::BorderBlockEnd
            | Self::BorderRadius
            | Self::Shadow
            | Self::Rotate
            | Self::Background
            | Self::FontSize
            | Self::FontWeight
            | Self::FontStyle
            | Self::FontFamily
            | Self::Color
            | Self::TextAlign
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::TextTransform
            | Self::TextDecoration
            | Self::GridArea
            | Self::Order
            | Self::AlignSelf
            | Self::JustifySelf
            | Self::AspectRatio
            | Self::ObjectFit
            | Self::ObjectPosition
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd
            | Self::Scroll
            | Self::Fill
            | Self::Sticky
            | Self::StretchLink
            | Self::Show
            | Self::Hide
            | Self::Class
            | Self::Id => true,
        }
    }

    /// Whether it is one of the common params of spec §9.2, which style the box; the others are attributes (§5.1).
    pub(crate) fn is_common(self) -> bool {
        match self {
            Self::Class | Self::Id | Self::Lang | Self::Dir => false,
            Self::Width
            | Self::Height
            | Self::MinWidth
            | Self::MaxWidth
            | Self::Padding
            | Self::PaddingInline
            | Self::PaddingBlock
            | Self::PaddingInlineStart
            | Self::PaddingInlineEnd
            | Self::PaddingBlockStart
            | Self::PaddingBlockEnd
            | Self::Border
            | Self::BorderInlineStart
            | Self::BorderInlineEnd
            | Self::BorderBlockStart
            | Self::BorderBlockEnd
            | Self::BorderRadius
            | Self::Shadow
            | Self::Rotate
            | Self::Background
            | Self::FontSize
            | Self::FontWeight
            | Self::FontStyle
            | Self::FontFamily
            | Self::Color
            | Self::TextAlign
            | Self::LineHeight
            | Self::LetterSpacing
            | Self::TextTransform
            | Self::TextDecoration
            | Self::GridArea
            | Self::Order
            | Self::AlignSelf
            | Self::JustifySelf
            | Self::AspectRatio
            | Self::ObjectFit
            | Self::ObjectPosition
            | Self::InsetInlineStart
            | Self::InsetInlineEnd
            | Self::InsetBlockStart
            | Self::InsetBlockEnd
            | Self::Scroll
            | Self::Fill
            | Self::Sticky
            | Self::StretchLink
            | Self::Show
            | Self::Hide => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_param_has_its_kind_and_breakpoint_rule() {
        assert_eq!(
            Universal::from_name("padding-inline").map(Universal::kind),
            Some(Kind::SPACES)
        );
        assert_eq!(Universal::Scroll.kind(), Kind::BOOLEAN);
        assert_eq!(Universal::Id.kind(), Kind::TEXT);
        assert!(Universal::Width.takes_breakpoint());
        assert!(!Universal::Show.takes_breakpoint());
        assert!(Universal::Class.needs_box());
        assert!(!Universal::Dir.needs_box());
        assert!(Universal::Fill.is_common() && !Universal::Lang.is_common());
    }
}
