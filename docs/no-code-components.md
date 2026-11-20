# What `compmd` definitions need for the no-code components

**Status: proposed (2026-11-20, CMD-359).** The no-code components of [the first list](first-components.md)
(`link-card`, `gallery`, `figure`, `badge`, `columns`, `sticky-note`, `timeline`) are `compmd` definitions (spec §7.2), so
every processor renders them and a plain Markdown reader reads them. `card` (CMD-313) went in as one. Each of the
others stops at something the definition language can't say yet. This design lists those gaps and closes them with
the smallest spec changes that do, tried on each component's own example. The spec text waits for the maintainer's
go. The blocked tasks (CMD-314, CMD-315, CMD-316, CMD-317, CMD-348) are answered from it, and CMD-318 and CMD-319
wait for it.

**Terms.** The **caller** is the document whose invocation uses a definition. A **destination** is a link's or an
image's URL. A **base** is the file a relative destination resolves against (spec §8.6, *Relative paths*): today the
definition's own file for a destination written in its body, and the caller's for slot content.

## Decisions

1. **A destination that starts with a param resolves against the caller.** In a definition body,
   `[{{title}}]({{href}})` and `![]({{src}})` take the caller's base when the destination starts with an
   interpolation of a param, of a field of one, or of an `each` or template slot name over one. A destination that
   starts with literal text keeps the definition's base: `![](./icons/{{icon}}.svg)` is the pack's icon. This mirrors
   the rule that already looks at a destination's start for its scheme (§8.1: relative, `https:`, `http:` or `mailto:`,
   else W109). Today `href=harbour.comp.md` would point into the pack's folder, or out of it (F026).
   - **Rejected:** a `url` param type. It names the same thing at the signature, but a list of records (`gallery`'s
     `rows`) holds paths in fields no type reaches, and decision 1 covers both.
   - **Rejected:** "write absolute URLs" (the data rule, §8.6). Authors link their own posts with relative paths.
2. **`md.fallback` is a leaf's or inline invocation's fallback, in its body**: rendered where it is interpolated as
   text, and as its plain text in an image's alt, where interpolation is now allowed too (§8.1, *Where*, adds "image
   alt text"). So `figure`'s one sentence is both its caption and its alt, as the first list says, and a plain reader
   shows it in the figure's place. A container has no `md.fallback`; using it there is F014.
3. **`stretch-link`, a flag on a box: its first link covers the whole box.** Markdown links are inline, so no link
   can hold a card's heading and text. The flag makes the box's first link (in `link-card`, its title) take clicks
   anywhere on the box, as CSS does with the link's `::after` set over the box, which is already positioned. The
   other links in the box stay above it. The link's text is the accessible name, not the whole card's text, and a plain
   reader shows the title as the link. A box with the flag and no link is W124, at the flag.
4. **An inline invocation is one inline box,** styled by its fence's params under the caller's, as a block invocation
   is (§7.2, which today says "It has no box, so the fence's params don't apply"). The box holds the children of the
   body's paragraph. It is laid out as an inline block, so padding, border, radius and background apply as they do
   to a block. `box`'s layout params (`display`, `gap`, …) on the fence don't apply to an inline use. In the expanded
   tree, it is a `componentInline` named `box`.
   - **Rejected:** an inline `:box[…]{…}` of its own. A badge would then be a definition whose body is one `:box`, the
     same box one level down, and every inline definition would need to write it.
5. **`md.slots.<name>`, true when the caller gave that slot content,** in a definition's expressions: its fence
   params and the expressions in its body (`if`, params of what it uses). `md.slots.default` is the default slot. So
   `columns` counts its columns from the slots given, and `::: if {when=(md.slots.middle)}` leaves out an empty one.
   - **Rejected:** a `count` param. It says twice what the slots say, and `count=2` with a `:: #middle` loses the
     middle column without a word.
6. **`shadow`, `rotate` and `border`'s sides, common params** (§9.2): `shadow` is `none`, `s`, `m` or `l`, from the
   visual tokens (`--compmd-shadow-s` …), as `border-radius` is; `rotate` is an angle in `deg` from `-15deg` to
   `15deg`, as CSS's `rotate`, which moves nothing around it; `border-inline-start`, `border-inline-end`,
   `border-block-start` and `border-block-end` take `border`'s values, as `padding`'s sides take `padding`'s, `border`
   covering them in the cascade of §7. All print. A sticky note stands out of the page, and a timeline's line runs
   along its start edge; a component can't add them, since a pack ships no CSS.
   - **Rejected:** letting a pack ship a stylesheet for its definitions. CSS from a pack in the host's page can cover
     the page with its own content and load URLs (`url()`), which the sandbox exists to prevent.
7. **`gallery` needs no change of its own** once decision 1 is in: it is a leaf over data rows, each one a `card`
   with its image, title and description, made in the definition with `each`. The fallback is the gallery's
   sentence, and its images' paths are the caller's (decision 1).

## The components with these decisions

````md
<!-- link-card (decisions 1, 3) -->
```compmd component=link-card {stretch-link}
params:
  title: string
  href: string
  description: string = ""
---
::: card
### [{{title}}]({{href}})

{{description}}
::: /card
```

:: link-card[The harbour at dusk] {title="The harbour at dusk" href="harbour.comp.md" description="A walk along the old quay."}
````

`href` is the caller's: `harbour.comp.md` next to the post. A plain reader shows the leaf's fallback, the title.

````md
<!-- gallery (decisions 1, 7) -->
```compmd component=gallery
params:
  rows: array
---
::: grid {min=14rem gap=m}
::: each {in=(rows) as=walk}
::: card
![{{walk.alt}}]({{walk.image}})

### {{walk.title}}

{{walk.description}}
::: /card
::: /each
::: /grid
```

:: gallery[Three walks around the harbour] {rows=(md.data.walks)}
````

`walk.image` is a field of a param, so `images/quay.jpg` in the caller's data is the caller's.

````md
<!-- figure (decisions 1, 2) -->
```compmd component=figure {padding-block=s}
params:
  src: string
---
![{{md.fallback}}]({{src}})

{{md.fallback}}
```

:: figure[Boats at the quay, the lamps just lit.] {src="harbour.jpg"}
````

The caption is the fallback, so a plain reader shows it where the figure is, and the image's alt is its text.

````md
<!-- badge (decision 4) -->
```compmd component=badge {padding-inline=s border-radius=full background=(tone)}
params:
  tone: subtle | strong | accent = subtle
---
{{md.fallback}}
```

The new :badge[beta]{tone=accent} search, from version :badge[2.1].
````

The badge's text is its fallback (decision 2), in an inline, tinted box. A plain reader shows `beta` and `2.1` in
the sentence.

````md
<!-- columns (decision 5) -->
```compmd component=columns {display=grid gap=m md:grid-template-columns=(split == "wide-start" ? (md.slots.middle ? "2fr 1fr 1fr" : "2fr 1fr") : split == "wide-end" ? (md.slots.middle ? "1fr 1fr 2fr" : "1fr 2fr") : split == "wide-middle" ? "1fr 2fr 1fr" : (md.slots.middle ? "1fr 1fr 1fr" : "1fr 1fr"))}
params:
  split: even | wide-start | wide-middle | wide-end = even
slots: [default, middle, end]
---
::: box
{{md.slot}}
::: /box

::: if {when=(md.slots.middle)}
::: box
{{md.slot.middle}}
::: /box
::: /if

::: box
{{md.slot.end}}
::: /box
```
````

Below `md` the grid has one column, so the columns stack in order, as a plain reader shows them. `wide-middle` with no
middle slot gives two columns the widths `1fr 2fr` (the first two of three), and nothing reports it: *Gaps left*.

````md
<!-- sticky-note (decision 6) -->
```compmd component=sticky-note {padding=m background=accent shadow=m rotate=-2deg width=14rem}
---
{{md.slot}}
```

::: sticky-note {inset-inline-end=1rem inset-block-start=4rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
````

````md
<!-- timeline (decision 6) -->
```compmd component=timeline {border-inline-start=strong padding-inline-start=m}
---
{{md.slot}}
```

::: timeline
- **1897**: the harbour wall is built.
- **1952**: the quay is rebuilt after the storm.
- **2024**: the lamps along the quay come back.
::: /timeline
````

A list of items is enough, as the first list says. The dates are the caller's text, so a plain reader keeps them,
and the line is the box's start border, on the right in a right-to-left document.

## Gaps left

- **A definition can't report a diagnostic of its own.** `columns` with `split=wide-middle` and no middle slot draws
  two columns at 1fr 2fr 1fr. Its signature can't say "only with `:: #middle`". Proposed for later: a signature
  `requires:` rule. Until then, the pack's README says it, and the column is the wrong width, not lost.
- **The processor keeps a pack's relative destinations as written** (a probe of `[x](./own.md)` in a pack's body).
  The spec makes "a pack's files reachable in the output" the processor's job. Decision 1 gives such destinations
  two bases, so the expanded tree must say which. Proposed: the processor writes every relative destination in the
  expanded tree relative to the processed document, and a pack's as the URL it serves the pack's files at. That is
  checked before the spec text is written, in case the processor or the host already resolves them elsewhere.
- **`stretch-link` with a card whose other text holds a link** works (the other links stay above), but a reader can't
  select the card's text by dragging over it. It is the usual trade-off of a whole-card link, and the card's own
  choice.

## Conformance cases

| Case | Decision | What it checks |
| --- | --- | --- |
| **new** `expand/param-destination-base` | 1 | a definition's `[{{title}}]({{href}})`, `![]({{rows.image}})` in `each`, and `![](./icon-{{name}}.svg)`: the first two the caller's base, the third the pack's |
| **new** `expand/md-fallback` | 2 | `md.fallback` as text and as alt; in a container's body, F014 |
| **new** `validate/stretch-link-W124` | 3 | the flag on a box, a component and a fence; without a link in the box, W124 |
| **new** `expand/inline-invocation-box` | 4 | an inline use of a definition with fence params: a `componentInline` box, the fence's params under the caller's; layout params left out |
| **new** `expand/md-slots` | 5 | `md.slots.middle` true and false, in a fence param and an `if` |
| **new** `validate/shadow-rotate-and-border-sides` | 6 | each `shadow` token, `rotate` at its bounds and past them (F009), each border side with each `border` value |
| **new** `expand/border-sides-cascade` | 6 | a fence's `border` under a caller's `border-inline-start`, and the reverse: per side, as `padding`'s |

**Existing cases.** No case expands an inline use of a `compmd` definition: the inline cases are `:value` and
parsing (`inline/*`), and `examples/spec-16-release-notes`'s `:badge[beta]{tone=info}` is imported and only parsed, so
decision 4 changes none. The cases with interpolated destinations (`expand/url-interpolation-W109`,
`expand/code-renderer-alt-interpolation`) are in documents, not definition bodies, so decision 1 leaves them as they
are. No case uses `md.fallback`, `md.slots`, `stretch-link`, `shadow`, `rotate` or a border side, as a name or a param.

## What changes

- **Spec:** §7.2 (inline invocations, decision 4; `md.fallback` and `md.slots`, decisions 2 and 5), §8.1 (the paths
  table: `md.fallback`, `md.slots`; alt text, decision 2), §8.6 *Relative paths* (decision 1), §9.2 (`stretch-link`,
  `shadow`, `rotate`; their tokens in §9.4), §10 (W124, decision 3), §12.3 (the inline box), the changelog
  under *Unreleased*.
- **Processor:** the bases of decision 1, `md.fallback`, `md.slots`, the inline box, the three params' validation.
- **Renderer:** the inline box, `stretch-link`'s CSS, `shadow`, `rotate` and the border sides.
- **Packs:** `link-card`, `gallery`, `figure`, `badge`, `columns`, `sticky-note`, `timeline`, each with its demo and
  tests, as their tasks say.
