# Sizing a component in its box

**Status: proposed (2026-11-20, CMD-8, CR-29).** How an author sizes and places what a host-language component draws,
whatever library draws it, and what the spec, the renderer, the sandbox and `@compmd/pack` change for it. It comes
from [the spike of wrapping an npm package](../spikes/wrap-npm-package/README.md), whose QR codes were the column's
width with no way an author could see to make them smaller, and whose frames scrolled once a use had a `width` and a
border. It replaces decision 8 of [writing a pack](writing-a-pack.md) as first written ("fills its box's width").

**Terms.** A use's **box** is its outer box (spec §7.4, §9.2), which the common params size: `width`, `height`,
`min-width`, `max-width`, `aspect-ratio`, `fill`, or the layout around it (a grid row that stretches it). Its
**inside** is the box less its border and padding. A drawing's **aspect ratio** is its own when it has one (a QR code
is square, an SVG has its `viewBox`); a chart or a map has none. A component's **natural size** is the size it has
when its box gives it none, as CSS sizes content before it knows its container: a percentage of an unknown size
counts as `auto`. A use is **framed** when the sandbox draws it in a frame of its own (host mode), **in the page** when
its component runs in the document that lays it out (a trusted component, or every component of a page drawn in one
frame).

## Decisions

1. **A component fits its box, and `object-fit` and `object-position` say how** (spec §7.4 and §9.2; two new common
   params). The component takes its box's inside: the width, and the height when the box has one; a box with no
   height is as tall as the component's natural height at that width.
   - **`object-fit`**, for a drawing with an aspect ratio of its own: `contain` (the default: as large as fits, the
     ratio kept), `cover` (the box filled, the ratio kept, the rest cropped) or `fill` (stretched to the box). CSS's
     values and meanings; its `none` and `scale-down` are left out, since a drawing here has no size in pixels to
     keep. A drawing with no aspect ratio (a chart) fills its box whatever the value. `fill` is CSS's name for
     stretching, beside the flag `fill` (the box takes the space left over, §9.2): the two are different params.
   - **`object-position`**, where a drawing smaller or larger than the box sits in it: one or two of `start`,
     `center`, `end`, the block position then the inline one, as `padding` orders its two values; one value is
     both. `center` by default. Logical, as every direction in the spec: `object-position="center start"` is the
     left in a left-to-right document and the right in a right-to-left one.
   - **Both are written without a breakpoint** (F008 with one, "a breakpoint the param doesn't take"): a component gets them as
     values, which a breakpoint, CSS's to resolve, isn't. They may be expressions.
   - **The component gets them** in the host contract, as it gets `scheme`: `context.fit`, `{ mode: "contain" |
     "cover" | "fill", x: 0 | 0.5 | 1, y: 0 | 0.5 | 1 }`, the position resolved by the processor to fractions of the
     free space from the left and the top, the direction counted in. When they change, the processor sets `fit` and
     calls `update`. A component whose content flows (text, controls) ignores it.
   - **A component has no size or placement params of its own** (`size`, `width` or `height` in pixels, `align`).
2. **Framing is invisible to layout** (spec §7.4, MUST for a processor). A use is laid out the same framed as in the
   page: its box the same size, the component's `target` the box's inside, the box's params applied once, and what it
   draws the same size in the same place. This is the one rule for every case that was open: a use in a grid row, in a
   row, inline, with or without a size. None needs a rule of its own, since CSS decides it in the page and the frame
   must match. A frame never scrolls unless its box has `scroll` (§9.2): content larger than a box with a height is
   clipped, as the box's edge is the frame's. *The mechanism*, below, is how the sandbox does it; the prototype
   compares 28 uses drawn both ways, and they are the same in Chrome and WebKit.
3. **A component's natural size is CSS's** (spec §7.4):
   - a drawing with an aspect ratio takes the width it is given and its height from the ratio; where nothing gives
     it a width either (in a row, inline), it is CSS's 300 pixels wide;
   - a drawing with no aspect ratio is CSS's 300 by 150 pixels, as an `iframe` or a `canvas` is;
   - content that flows is as tall as its lines.

   **A drawing sized in script from its box contains its size** (CSS `contain: size`, with `contain-intrinsic-size`):
   what it drew last then never counts as its natural size. Without it, a box that was stretched would never shrink
   again, since the drawing it holds is as tall as the box was.
4. **A signature fence may give the component's box its defaults,** as a host-language definition fence already may
   (spec §7, *The component's box*): ```` ```compmd component=qr-code signature {width=12rem max-width=100%} ````,
   the caller's params over them as for a definition's fence (§12.3). A pack's component is a built module with no
   definition fence, so its signature fence is the only place for them. It is where a pack says what 300 pixels
   can't: a QR code is 12rem, a chart `{aspect-ratio=16/9}`, a photo `{object-fit=cover}`. It takes what a
   host-language definition fence takes: the common params, `class`, `id`, `lang` and `dir`; any other key is F008.
   A processor that doesn't run the component still knows its box, so a use holds its place while its pack loads.
   **F036 changes meaning:** today "a param list on a signature fence"; now "a component whose box both its
   definition fence and its signature fence style", reported at the signature fence's `{`, the definition fence's
   list kept.
5. **The common params' names are the processor's, as `class` and `id` are:** a signature that declares one
   (`width`, `height`, `object-fit`, …), or `lang` or `dir`, is F013 (§5.1). The processor applies them to the box
   and never passes them on (§7.4), so such a param could never reach the component: the spike's layerchart `height`
   was dropped without a word.
6. **Sizes include padding and border** (§9.2): "`width`, `max-width` and `aspect-ratio` measure the block with its
   padding and border", as the renderer does already (`box-sizing: border-box`). The spec says padding only.

## What it looks like

````md
<!-- in the pack's index.comp.md -->
```compmd component=qr-code signature {width=12rem max-width=100%}
params:
  value: string
```

<!-- in a document -->
:: qr-code[Our site] {value="https://compmd.dev/"}                                     12rem wide, square
:: qr-code[Our site] {value="https://compmd.dev/" width=6rem}                          6rem
:: qr-code[Our site] {value="https://compmd.dev/" width=100% height=6rem}              6rem tall, centred in the column
:: qr-code[Our site] {value="https://compmd.dev/" width=100% height=6rem object-position="center start"}   at the start
:: photo[The harbour at dusk] {src="harbour.jpg" aspect-ratio=16/9 object-fit=cover object-position="end center"}  cropped to 16:9, its bottom kept
:: chart[Sales] {rows=(md.data.sales) x=month y=total height=20rem}                    the column's width, 20rem tall
````

## The mechanism

What the renderer and the sandbox do for decision 2, tried in
[the prototype](../spikes/wrap-npm-package/sizing/page.html) before any of it is in the packages:

- **A host-language component's box is a grid of one column, in the page and framed.** A percentage height in it is
  then of its cell, which CSS defines for every way a box gets a height (its own, its `aspect-ratio`, a row that
  stretches it), and every browser sizes alike. As a block, WebKit grows a box with an `aspect-ratio` by its border
  and padding when a child is `height: 100%` (the 4 pixels the spike saw).
- **A framed box holds a sizer, and the frame in it.** The sizer fills the box's one cell and holds an invisible block
  the component's natural size, so the box sizes as if it held the component: it is as tall as the component when it
  has no height, and as wide where its width comes from its content. The frame is positioned over the sizer
  (`position: absolute`, all of it), so it is always the box's inside and adds nothing to its size.
- **In the frame, `target` is the frame's size,** a grid as the box is, so the component is laid out in the box's
  inside, a definite size. The part gets none of the box's params: the page's box has them.
- **The frame reports the component's natural size,** which it measures by laying `target` out with no size from the
  frame, in one task, so nothing is painted or observed in between: first the height at the frame's width, then the
  width, with `target` laid out anew as a block (WebKit otherwise gives the frame's own height or width as a
  drawing's natural width, which never settles). The natural size doesn't depend on the frame's, so the two can't
  feed each other: a row that stretched a box lets it shrink again (the prototype's grid rows, 20rem then 5rem).
- **It reports when it has mounted, when its content changes size, and when the page says its box did.** Never from
  an animation frame, and never from its own observers alone: a browser pauses both in a frame that is off screen
  (Chrome), and its box would have no size until it is scrolled to. The page's observer of the sizer is never paused,
  and a message is always delivered.
- **The page's message carries the width of the box's inside (`box {width}`), and the frame measures its height at
  that width,** not at its own: off screen, Chrome doesn't give a frame its new size either, so a box whose width
  comes from its content (in a row, inline) would keep the height it had at its first width, a pixel. Found by
  `@compmd/sandbox`'s comparison test (CMD-355); the prototype measured at the frame's width.
- **The frame's document never scrolls** (`overflow: hidden`), unless the box has `scroll`.
- **`fit`** goes to the frame with the part, resolved by the page.

## Tried on the prototype

`spikes/wrap-npm-package/sizing/`: one page draws each use in the page and framed (a sandboxed frame from another
origin), with the same component code, the boxes bordered in magenta; `sizing.ts` compares the two in Chrome and
WebKit, with every frame on screen and with most of them off it (`--height=900`). All 28 uses are the same both ways
in both browsers, and the six in grid rows again after the row's tall box is made short; no frame reports more than
five times. Sizes are the box's inside, in pixels, the page 16 pixels a `rem`:

| The use | What is drawn | In |
|---|---|---|
| QR code, no size | 464 × 464 | the column, 464 wide |
| QR code, `width=10rem` | 156 × 156 | 156 × 156 |
| QR code, `height=6rem` | 92 × 92, centred | 464 × 92 |
| QR code, `width=16rem height=6rem` | 92 × 92, centred | 252 × 92 |
| QR code, `width=6rem height=12rem` | 92 × 92, centred | 92 × 188 |
| QR code, `width=12rem aspect-ratio=2/1` | 92 × 92, centred | 188 × 92, in WebKit too |
| QR code, `height=6rem object-position=start` | 92 × 92, at the start | 464 × 92 |
| QR code, `width=6rem height=12rem object-position=end` | 92 × 92, at the end | 92 × 188 |
| QR code, `width=16rem height=6rem object-fit=cover` | 252 × 252, cropped to the box | 252 × 92 |
| QR code, `width=16rem height=6rem object-fit=fill` | 252 × 92, stretched | 252 × 92 |
| Chart, no size | 464 × 150 | the column, its natural height |
| Chart, `aspect-ratio=3/1` | 464 × 152 | 464 × 152 |
| Chart, `height=8rem` | 464 × 124 | 464 × 124 |
| Text, no size | its two lines | the column |
| Text, `height=2rem` | its lines, clipped at the box | 464 × 28 |
| A grid row: a chart of `height=20rem`, and a chart, text, or a QR code with no size | each fills, or fits, the row's 316 | 226 × 316 each |
| The same once the first chart is `height=5rem` | the second chart 150 tall (natural), the text 76, the QR code 226 (its natural height at that width) | the row follows |
| A row (flexbox): a QR code, a QR code of `width=5rem`, a chart, no sizes | 300 × 300, 76 × 76 in a box the row's height, 300 × 150 | natural widths |
| Inline: a QR code of `width=3rem`, a chart of `width=6rem height=2rem`, a QR code with no size | 44 × 44, 92 × 28, 300 × 300 | in the line |

What the prototype doesn't run, each a browser test of the sandbox's task (*What changes*): `object-position` in a
right-to-left document; `scroll`; a drawing that is an `img` or a `canvas`, which takes CSS's own `object-fit`; and a
script-sized drawing that doesn't contain its size, which decision 3 says holds its box open.

Earlier, on the real packs (`spikes/wrap-npm-package/try.ts`): fitting only the width overflowed every box with a
height; `draw()` of `@compmd/pack` puts the drawing in a `div` with no height, so a drawing can't take the box's
height through it; and every framed use with a `width` scrolled by its border, the sandbox applying the box's params
to the page's box and again inside the frame.

## Conformance cases

All the cases were read for these decisions (`conformance/`, stages 1 to 4b). Decisions 2, 3 and 6 are about layout,
which the set doesn't cover: their tests are browser tests. Decisions 1, 4 and 5 change these:

| Case | Today | With this design |
|---|---|---|
| `definitions/fence-params` | F036 at 11:38, a parser case: `chart`'s signature fence has `{height=16rem}` | `chart` has a definition fence with `{height=16rem}` too, so F036 still applies, but at resolving, since which fences a component has is known only then (a signature may attach to a definition imported from a component file, §7.1). The case loses its F036 (it stays a parser case, for F011 and F017), and its `parse.json` gains the signature fence's `params`. |
| `definitions/signature-types` | declares `width: length?` | F013 under decision 5; the case is about types, so the param becomes `stroke: length?` (its `parse.json` with it) |
| `resolve/params-differ-in-dash-F013` | `max-width` and `max_width`: one F013, at 1:1 | `max-width` would be a second F013; the case is about `-` and `_`, so they become `line-width` and `line_width`, the one diagnostic as it is |
| `expand/host-component-box` | the definition fence's box params under the caller's | unchanged |
| `validate/code-renderer-render-F009-F013-W101` | `class` and `id` declared: F013 | unchanged |
| `validate/params-and-common-params`, `validate/layout-values`, `expand/component-box` | the common params on uses, and their values | unchanged; no case uses `object-fit` or `object-position` today, as params or as anything else |
| **new** `validate/object-fit-and-position` | — | every value of both on a leaf and a container, one and two positions; `object-fit=none`, `object-position=left` and three positions: F009 at each; `md:object-fit=cover`: F008 |
| **new** `expand/object-fit-and-position` | — | both on a host-language leaf, from its signature fence's defaults and the caller's over them, and as an expression: the expanded `params` |
| **new** `resolve/signature-and-definition-box-F036` | — | a host-language definition fence and its signature fence both with box params: F036 at the signature's `{` |
| **new** `expand/signature-box-defaults` | — | a signature fence's `{width=12rem max-width=100%}` and no definition fence's list, used with no params, with `width=6rem`, and with `md:width=8rem`: the expanded `params` as `host-component-box`'s |
| **new** `validate/signature-box-unknown-F008` | — | `{title=x}` on a signature fence: F008 |
| **new** `resolve/common-param-declared-F013` | — | a signature declaring `width`, `height`, `padding`, `object-fit`, `lang` and `dir`, and a `compmd` definition declaring `border`: F013 at each |

**The tree schema:** a signature fence's node (`componentSignature`) gains `params`, as written, the same shape as a
definition fence's (§12.1), so a tool reads a pack's box defaults from its parse tree. `conformance/README.md`'s rows
for `definitions/`, `resolve/`, `validate/` and `expand/` say so.

## What changes

**The spec** (for the maintainer's go; a changelog entry under *Unreleased*):

- §9.2, the common params' table: `object-fit` and `object-position`, with decision 1's rules; "padding and border"
  (decision 6).
- §7.4, the host contract: `fit` in `mount`'s context; decisions 1 to 3 beside *The outer box is the processor's*,
  which becomes "doesn't receive those params, but for `fit`".
- §7.6: the React and Svelte bindings pass `compmd.fit`, as they pass `compmd.scheme`.
- §7, *The component's box*: decision 4, the signature fence's param list; §10.2's F036 row.
- §5.1 and §10.2's F013 row: decision 5.
- §12.1: the signature fence's `params`.

**`@compmd/render`:**

- A host-language component's box is a grid of one column; `object-fit` and `object-position` validated, resolved
  with the box's direction, and given as `context.fit`, with an `update` when they change.
- The contract's type, and the React and Svelte bindings.

**`@compmd/sandbox`** (the scroll bars, and *The mechanism*):

- The sizer and the frame over it, in place of a frame as tall as its content.
- The frame's `target` the frame's size; the part with none of the box's params; no scrolling without `scroll`.
- The protocol, version 3: `size` carries the natural width and height; a `box` message from the page when the frame
  has loaded and when its box changes size; `fit` with the part
  ([the sandbox's frame protocol](sandbox-protocol.md)).
- Browser tests, in Chrome and WebKit: the prototype's cases, each use drawn by a trusted component in the page and
  by the same component framed, compared; with the four the prototype doesn't run, and with most frames off screen.

**`@compmd/pack`:**

- `draw()` fits what it is given: an SVG by its `preserveAspectRatio` (`meet`, `slice` or `none`, and where it
  sits), an `img` or a `canvas` by CSS's `object-fit`, from `context.fit`; its own `div` is the box's size.
- `fitBox(element, fit)` and `fillBox(element)` in `@compmd/pack/component`, for a component written as plain
  `mount`: the first does what `draw()` does, the second is the box's size and contains it (decision 3), for a
  drawing sized in script.
- `mountForTest` takes `fit`, and has `setFit`.
- The test page gets a **Show boxes** switch, which outlines every use's box, so an author sees the box against the
  drawing without borders in the demo.
- `create-pack`'s templates fit their box, and their demos have a use with `width`, one with `height` and one with
  `object-fit=cover`.
- [Wrapping an npm package](wrapping-an-npm-package.md): the glue of each kind fits the box; decision 6 there leaves
  out size props, and the common params' names, which decision 5 here makes F013; `compmd.wrap.json`'s `box` is the
  generated signature fence's param list.

**[Writing a pack](writing-a-pack.md), decision 8,** points here.

## Not this design's

- **A box's size on paper** is its size on screen (`rem`), and what fits it on screen fits it there. A `canvas`
  drawing prints at the screen's resolution, which is [printing a document](printing.md)'s.
- **The box's own place in its parent** is `align-self` and `justify-self` (§9.2), as today: `object-position` places
  the drawing in its box, not the box.
