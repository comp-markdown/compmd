# Sizing a component in its box

**Status: proposed (2026-11-20, CMD-8, CR-29).** How an author sizes a host-language component, whatever library draws
it, and what the spec, the sandbox and `@compmd/pack` change for it. It comes from [the spike of wrapping an npm
package](../spikes/wrap-npm-package/README.md), whose QR codes were the column's width with no way the author could
see to make them smaller, and whose frames scrolled once a use had a `width` and a border. It replaces decision 8 of
[writing a pack](writing-a-pack.md) as first written there ("fills its box's width"), which a box with a height broke.

**Terms.** A use's **box** is its outer box, the component's `target` (spec §7.4, §9.2), which the common params size:
`width`, `height`, `min-width`, `max-width`, `aspect-ratio`, `fill`. Its **inside** is the box less its border and
padding. A drawing's **aspect ratio** is its own when it has one (a QR code is square, an SVG has its `viewBox`); a
chart or a map has none. **Fit** (CSS's `object-fit: contain`): as large as the inside allows with the aspect ratio
kept, centred. **Fill**: the whole inside, both ways.

## Decisions

1. **A component fits its box** (spec §7.4, a new rule, SHOULD for components). It takes the inside's width. It takes
   the inside's height too when the box has one (from `height`, `aspect-ratio`, `fill`, or a grid row that stretches
   it); otherwise the box is as tall as what it draws. A drawing with an aspect ratio of its own fits the inside; one
   without fills it. So an author sizes every component the same way, with the common params, and a component has no
   size params of its own (`size`, `width` or `height` in pixels).
2. **The box a component sees is exactly the use's box** (spec §7.4, MUST for processors): its params applied once,
   `target`'s inside the box's inside. A processor that draws a use somewhere else (the sandbox's frame for each use)
   gives it a `target` the box's inside's size and none of the box's params again. Today the sandbox applies them twice
   (*The sandbox*, below), which is the frames' scroll bars.
3. **A signature fence may give the component's box its defaults,** as a host-language definition fence already may
   (spec §7, *The component's box*): ```` ```compmd component=qr-code signature {width=12rem max-width=100%} ````, the caller's params over
   them as for a definition's fence (§12.3). A pack's component is a built module with no definition fence, so its
   signature fence is the only place for them, and a QR code without a `width` is then 12rem, not the column. It takes
   what a host-language definition fence takes: the common params, `class`, `id`, `lang` and `dir`; any other key is
   F008. **F036 changes meaning:** today "a param list on a signature fence"; now "a component whose box both its
   definition fence and its signature fence style", reported at the signature fence's `{`, the definition fence's list
   kept.
4. **The common params' names are the processor's, as `class` and `id` are:** a signature that declares one, or `lang`
   or `dir`, is F013 (§5.1). The processor applies them to the box and never passes them on (§7.4), so such a param
   could never reach the component: the spike's layerchart `height` was dropped without a word. `lang` and `dir`,
   which §5.1 already calls the processor's, join `class` and `id` in F013.
5. **Sizes include padding and border** (§9.2): "`width`, `max-width` and `aspect-ratio` measure the block with its
   padding and border", as the renderer does already (`box-sizing: border-box`). The spec says padding only.
6. **Not now: covering, stretching and placing.** Filling the inside and cropping (`object-fit: cover`), stretching
   out of the aspect ratio (`fill`), and placing a smaller drawing at a start or end (`object-position`) aren't in the
   common params. Nothing in [the first components](first-components.md) needs them: a chart has no aspect ratio, and a
   QR code or a diagram must never be cropped or stretched. A component that needs one (a photo, a map) takes a param
   of its own, `fit`; a common param waits until several do.

## What it looks like

````md
<!-- in the pack's index.comp.md -->
```compmd component=qr-code signature {width=12rem max-width=100%}
params:
  value: string
```

<!-- in a document -->
:: qr-code[Our site] {value="https://compmd.dev/"}                                12rem wide, square
:: qr-code[Our site] {value="https://compmd.dev/" width=6rem}                     6rem
:: qr-code[Our site] {value="https://compmd.dev/" height=6rem}                    6rem tall, centred in a 12rem box
:: chart[Sales] {rows=(md.data.sales) x=month y=total height=20rem}               the column's width, 20rem tall
:: chart[Sales] {rows=(md.data.sales) x=month y=total aspect-ratio=3/1}           the column's width, a third as tall
````

## Tried on the spike

Each use with a magenta border (`border="2px solid #e5007d"`), in Chrome and WebKit, the drawing measured against the
box's inside (`spikes/wrap-npm-package/try.ts`); the QR codes are react-qr-code's and qrcode's, the chart layerchart's
`BarChart`:

| The use | QR codes | Chart |
|---|---|---|
| no size | the column (760 px) square: too big, which decision 3 fixes | the column, its default height |
| `width=10rem` | 156 × 156, fills the inside | — |
| `width=20rem` | — | fills 316 × 288 |
| `height=6rem` | 92 × 92, centred in 760 × 92 | fills 760 × 124 (`height=8rem`) |
| `width=16rem height=6rem` | 92 × 92, centred in 252 × 92 | fills 380 × 156 (`width=24rem height=10rem`) |
| `width=6rem height=12rem` | 92 × 92, centred in 92 × 188 | — |
| `width=12rem aspect-ratio=2/1` | 92 × 92, centred in 188 × 92 | fills 760 × 251 (`aspect-ratio=3/1`) |

Chrome's numbers; WebKit's are the same but with `aspect-ratio`, where its box is 4 px taller (188 × 96, and the
chart's 760 × 255): it takes the ratio from the inside rather than from the box with its border, which decision 5
makes the measure. A browser test of decision 5 keeps the two apart.

What it found besides:

- **Fitting only the width fails** once a box has a height: the drawing overflows it (react-qr-code's `size`, an SVG at
  `width: 100%; height: auto`). Fitting both ways is SVG's own default, `meet`, at `width: 100%; height: 100%`; with no
  height, `100%` is the drawing's own and the box follows it.
- **`draw()` of `@compmd/pack` breaks it:** it puts the drawing in a `div` with no height, so a drawing can't take the
  box's height through it; qrcode's overflowed until its glue wrote into `target` itself.
- **Every framed use with a `width` scrolled by its border** (4 px, a 2 px border each side), in both browsers: the
  sandbox's double box (decision 2).

## Conformance cases

All the cases were read for these decisions (`conformance/`, stages 1 to 4b). Decisions 1, 2, 5 and 6 are about layout,
which the set doesn't cover; their tests are browser tests (*The sandbox*, *`@compmd/pack`*). Decisions 3 and 4 change
these:

| Case | Today | With this design |
|---|---|---|
| `definitions/fence-params` | F036 at 11:38, a parser case: `chart`'s signature fence has `{height=16rem}` | `chart` has a definition fence with `{height=16rem}` too, so F036 still applies, but at resolving, since which fences a component has is known only then (a signature may attach to a definition imported from a component file, §7.1). The case loses its F036 (it stays a parser case, for F011 and F017), and its `parse.json` gains the signature fence's `params`. |
| `definitions/signature-types` | declares `width: length?` | F013 under decision 4; the case is about types, so the param becomes `stroke: length?` (its `parse.json` with it) |
| `resolve/params-differ-in-dash-F013` | `max-width` and `max_width`: one F013, at 1:1 | `max-width` would be a second F013; the case is about `-` and `_`, so they become `line-width` and `line_width`, the one diagnostic as it is |
| `expand/host-component-box` | the definition fence's box params under the caller's | unchanged |
| `validate/code-renderer-render-F009-F013-W101` | `class` and `id` declared: F013 | unchanged |
| `validate/params-and-common-params`, `expand/component-box` | the common params on uses | unchanged |
| **new** `resolve/signature-and-definition-box-F036` | — | a host-language definition fence and its signature fence both with box params: F036 at the signature's `{` |
| **new** `expand/signature-box-defaults` | — | a component with a signature fence's `{width=12rem max-width=100%}` and no definition fence's list, used with no params, with `width=6rem`, and with `md:width=8rem`: the expanded `params` as `host-component-box`'s |
| **new** `validate/signature-box-unknown-F008` | — | `{title=x}` on a signature fence: F008 |
| **new** `resolve/common-param-declared-F013` | — | a signature declaring `width`, `height`, `padding`, `lang` and `dir`, and a `compmd` definition declaring `border`: F013 at each |

**The tree schema:** a signature fence's node (`componentSignature`) gains `params`, as written, the same shape as a
definition fence's (§12.1), so a tool reads a pack's box defaults from its parse tree. `conformance/README.md`'s rows
for `definitions/`, `resolve/` and `validate/` say so.

## What changes

**The spec** (for the maintainer's go; a changelog entry under *Unreleased*):

- §7.4, the host contract: decisions 1 and 2, beside *The outer box is the processor's*.
- §7, *The component's box*: decision 3, the signature fence's param list; §10.2's F036 row.
- §5.1 and §10.2's F013 row: decision 4.
- §9.2: decision 5, "padding and border".
- §12.1: the signature fence's `params`.

**The sandbox** (`@compmd/sandbox`, `@compmd/render`), the scroll bars:

- In a use's frame, the part's box takes none of the use's box params: the page's box has them, and the part fills the
  frame (`createRegion`'s `part` gives the box no params).
- When the page's box has a height of its own, the frame takes it (`height: 100%`) instead of its content's; otherwise
  the frame is as tall as its content, as today ([running components in the sandbox](running-components-in-the-sandbox.md), Q5).
- Browser tests, in Chrome and WebKit: a framed use with a border, with each case of the table above; no frame scrolls,
  and the frame's inside is the box's.

**`@compmd/pack`:**

- `draw()`'s `div` fills the box (`display: block; width: 100%; height: 100%`).
- `fitBox(svg)` in `@compmd/pack/component`: an SVG element or markup given the style that fits it, for the packs that
  draw SVG.
- The test page gets a **Show boxes** switch, which outlines every use's box, so an author sees the box against the
  drawing without borders in the demo.
- `create-pack`'s templates fit their box, and their demos have a use with `width` and one with `height`.
- [Wrapping an npm package](wrapping-an-npm-package.md): the glue of each kind fits the box; decision 6 leaves out
  size props, and the common params' names, which decision 4 makes F013.

**[Writing a pack](writing-a-pack.md), decision 8,** points here.

## Gaps

- **S1. A box's height in a grid row.** A use in a `grid` whose row stretches it has a height its params don't give;
  the frame reports its content's height, so the row's stretch doesn't reach the component. Decision 1 counts it as a
  height; the sandbox's fix sees only the box's own. To check with the `columns` component (CMD-348), whose columns
  are such boxes.
- **S2. Inline components** (`:qr-code[…]` in text) have a box too, laid out inline; decision 1 holds for them, and
  their frames report a width already. Not tried in the spike.
- **S3. Printing.** A box sized in `rem` prints at that size; a drawing that fits it prints sharp if it's SVG. A canvas
  drawing prints at screen resolution ([printing a document](printing.md)).
