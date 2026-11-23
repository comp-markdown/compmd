# Gradients in `background`

**Status: proposed (2026-11-23, CMD-387).** A box's `background` takes a surface token or a CSS color (spec §9.2).
Nothing in CompMD can fade a box, stripe it or cut its corner, so the sticky note's turned-up corner (CMD-386, CR-76)
leaves its yellow where the page should show through. This design lets `background` take a linear or a radial
gradient, checked part by part as a color is, with directions that follow the text's as the rest of the layout does.
Decisions 1 to 3 were settled with the maintainer on 2026-11-23; decision 4 is proposed here, found while trying the
sticky note's corner. The spec text waits for the maintainer's go.

**Terms.**
- **A gradient** is one of CSS's gradient functions, written as `background`'s value.
- **A stop** is a color with up to two positions, `#fff176 1rem`.
- **Logical** means it follows the text's direction, as `inset-inline-end` does: `inline-start` is the left in
  left-to-right text and the right in right-to-left text, `block-start` the top.
- **Physical** means a fixed side of the screen, `left` or `top`, whatever the text's direction.

## Decisions

1. **`background` takes a gradient:** `linear-gradient()`, `radial-gradient()`, `repeating-linear-gradient()` and
   `repeating-radial-gradient()`, besides its tokens and a CSS color. The other common params that take a color
   (`color`, `border`'s color) don't: text and lines in a gradient are another feature.
   - **Rejected:** conic gradients, until a document needs one.
   - **Rejected:** several backgrounds, comma-separated at the top level, as CSS's `background` allows. One gradient
     does what the examples below need.
2. **A gradient is checked part by part, never passed through raw**, as a color and a border are
   (`crates/processor/src/values/forms.rs`). Every part is a word from a fixed list, a number with a unit, or a color
   that `is_color` accepts. Anything else in it is F009:
   - another function: `url()`, `image()`, `image-set()`, `var()`, `env()`, `attr()`, `calc()`;
   - a quote, a backslash, `;`, `{` or `}`.

   So a gradient can't load anything or reach the rest of the page's CSS. The message names the forms `background`
   takes.
3. **Logical directions.** A linear gradient's `to` takes logical sides, one or a block side and an inline side:
   `to block-end`, `to inline-end`, `to block-end inline-end`. They are turned into physical sides for the box's text
   direction. CSS's physical sides (`to right`, `to bottom left`) are kept as CSS has them. Logical and physical sides
   are never mixed in one `to` (F009).
4. **Proposed: an angle is logical too.** CSS's `0deg` points up and `90deg` to the right. In CompMD, `0deg` points to
   `block-start` and `90deg` to `inline-end`, so in right-to-left text an angle is mirrored: `90deg` points left, and
   `315deg` to the top right.
   - **Why:** a cut corner needs an angle, not `to` a corner. `to block-start inline-start` points along the box's
     diagonal, so on a note taller than it is wide, the cut is steeper than 45°. Its flap is a square triangle at 45°,
     so the two don't meet. `315deg` is 45° on any box, and it follows right-to-left text as the flap does.
   - **The cost:** an angle copied from a CSS stylesheet means the same in left-to-right text, and the mirror image in
     right-to-left text.
   - **Rejected:** physical angles with logical words only. Then a 45° cut can't follow the text's direction, and the
     note's corner is wrong in Arabic or Hebrew.

## The grammar

What is accepted, written as CSS's own grammar, of which it is a part:

```text
linear-gradient( [ <angle> | to <sides> , ]? <stops> )
radial-gradient( [ <shape> || <size> ]? [ at <position> ]? , <stops> )     the first part optional, with its comma
repeating-linear-gradient( … )     as linear-gradient
repeating-radial-gradient( … )     as radial-gradient

<angle>     a number with deg, rad, grad or turn
<sides>     a logical side: block-start | block-end, and/or inline-start | inline-end, one of each at most
            or a physical side: top | bottom, and/or left | right, one of each at most
<shape>     circle | ellipse
<size>      closest-side | closest-corner | farthest-side | farthest-corner
            | a length (a circle) | two lengths or percentages (an ellipse)
<position>  one or two of: center, a logical or a physical side, a length, a percentage (as <sides>, never mixed)
<stops>     two or more <stop>, comma-separated
<stop>      <color> [ <length-or-percentage> [ <length-or-percentage> ] ]
```

A length is one that `Form::Length` accepts (`px`, `rem`, `em`, `%`, or `0`); a color is one that `is_color` accepts,
`color-mix()` included.

**Not accepted:**
- a color hint, a position alone between two stops;
- an interpolation space, `in oklch`;
- CSS's `<position>` with three or four values.

Each could be added when a document needs it, as conic gradients could.

## The renderers

- **The HTML renderer** writes the gradient to `background`, as it writes a color now (`npm-packages/render/src/styles.ts`).
  - A gradient with logical words or an angle gets two rules: the left-to-right one, and one under `:dir(rtl)` with
    `inline-start` and `inline-end` swapped and the angle mirrored (`360deg - a`). `:dir()` is in every browser the
    renderer supports (Chrome 120, Safari 16.4).
  - A gradient with physical words only gets one rule.
- **A processor whose output has a direction of its own**, such as a PDF page set right to left, uses that direction.
  **A plain renderer** shows no background, as now.
- **W126** (contrast) checks a box whose `color` and `background` are both opaque colors. A gradient isn't one, so it
  isn't checked, as a token isn't: the processor can't tell which of its colors the text sits on.
- **On paper**, a gradient prints as a color does.

## Tried on examples

The sticky note's corner, with decision 4. The note cuts its own corner, and its flap is the triangle that was there:

````md
```compmd component=sticky-note {padding=m padding-block-start=l background="linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)" color="#3b3000" shadow=m rotate=-2deg width=14rem max-width=100%}
---
…the pin…
{{md.slot}}
::: box {inset-block-end=0 inset-inline-end=0 width=1.98rem height=1.98rem background="linear-gradient(315deg, transparent 50%, #e0c93c 50%)"}
::: /box
```
````

The cut is the first 1.4rem along the gradient's line, from the block-end inline-end corner. At 45° that is a
triangle with legs of 1.98rem (1.4rem × √2), which is the flap box's size. In right-to-left text, both are mirrored to
the block-end inline-start corner. The note's shadow is a box's shadow, so it still follows the note's rectangle: a
thin shadow runs past the cut corner.

A fade under a card's image, from the photo into the card:

```md
::: box {background="linear-gradient(to block-end, transparent, color-mix(in srgb, currentColor 8%, transparent))" padding=m}
```

A striped row, for a table of results:

```md
::: box {background="repeating-linear-gradient(90deg, #f6f8fa 0 2rem, #ffffff 2rem 4rem)"}
```

A spotlight behind a quote:

```md
::: box {background="radial-gradient(circle at inline-start block-start, #fff5d6, transparent 60%)" padding=l}
```

Refused, each F009:
- `background="linear-gradient(url(x.png), red)"`: a function a gradient doesn't take;
- `background="conic-gradient(red, blue)"`: a gradient `background` doesn't take;
- `background="linear-gradient(to left block-end, red, blue)"`: physical and logical sides together;
- `background="linear-gradient(red)"`: one stop.

## Gaps left

- **The shadow at a cut corner.** A box's shadow follows its rectangle. A note with a cut corner shows a thin shadow
  past the cut. The spec has no shape for a shadow to follow.
- **Decision 4 is proposed:** if it isn't taken, angles stay physical, and the note's cut is made with `to` words and
  meets its flap only on a square note.
- **Vertical writing modes:** CompMD lays out horizontal text only (spec §9.2). A logical word or angle in vertical
  text is left for when the spec has vertical text.

## Conformance cases

- `params/background-gradients`: each of the four functions accepted, with an angle, logical and physical `to` words,
  a radial shape, size and `at` position, and stops with one and two positions. The expanded tree holds the value as
  written.
- `params/background-gradient-refused`: `url()`, `var()`, `calc()` and `conic-gradient()` inside or as
  `background`; mixed physical and logical sides; one stop; a quote or `;` inside. Each F009, at its param.

## What changes

- **The spec:** §9.2's table, where `background` takes "a token, a CSS color, or a linear or radial gradient (section
  9.4)", and §9.4, where the gradient's grammar, decisions 3 and 4, and W126's exception are. A changelog entry under
  *Unreleased*.
- **The code (CMD-388):** a gradient form in the processor, in a module of its own, and the renderer's rules.
- **The sticky note (CMD-386):** its corner, as above, in a new round of CR-76.
