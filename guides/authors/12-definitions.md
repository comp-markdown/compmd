# 12. Definitions

When the same shape comes back again and again, such as a card for each plot, you can write it once as a component of your own and use it like a built-in one. This chapter writes two: a card for a plot, used as a leaf, and a gardener's name, used inside a sentence. They are written in CompMD itself, so they need no code and work wherever the document is shown.

## The newsletter, with components of its own

````md snippet=definitions/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=plot-card {padding=s border=subtle border-radius=m background=(gardener == "" ? "accent" : "none")}
params:
  number: number
  gardener: string
  crop: string
---
# Plot {{number}}

:value[Maya Okafor, growing garlic.]{of=(gardener == "" ? "Free for next year." : gardener + ", growing " + crop + ".")}
```

```compmd component=gardener
params:
  name: string
  plot: number
---
**{{name}}** (plot {{plot}})
```

## The plots

::: grid {min=12rem gap=m}

::: each {in=(md.doc.plots) as=p}

:: plot-card[Plot {{p.number}}] {number=(p.number) gardener=(p.gardener) crop=(p.crop)}

::: /each

::: /grid

## Thanks

To :gardener[Maya Okafor (plot 1)]{name="Maya Okafor" plot=1} for turning the compost, and to :gardener[Priya Nair (plot 4)]{name="Priya Nair" plot=4} for mending the gate.
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/definitions) in the playground.

The page shows four cards, one per plot, each with its number as a heading and a line about it. Plot 3 has no gardener, so its card has the accent background and says *Free for next year.* The thanks name Maya and Priya in bold, with their plots.

## A definition

A component is defined in a fenced code block, the kind you'd write for code, whose first line says it is a definition:

````text
```compmd component=plot-card {padding=s border=subtle}
````

- `compmd` is the language the component is written in: CompMD itself. Components written in code use other languages, and are [the Developer's Guide](../developers/README.md)'s.
- `component=plot-card` names it. The name is how you write it in the document: `:: plot-card[…] {…}`.
- The params in braces, which may be left out, style the component's box (below).

Inside the fence come three parts:

1. The **signature**: what the component takes. Here `params` lists three params and the type of each value. Chapter 15 has every type; `string` (text) and `number` are the ones you'll use most.
2. A line of three dashes, `---`.
3. The **body**: CompMD, as you'd write it anywhere in a document. It is what each use of the component shows.

The fence itself doesn't show on the page. A plain Markdown viewer, such as GitHub's, shows it as a code block, so a reader there can see how the component is made.

## Params in the body

In the body, a param is a name of its own: `{{number}}` inserts the plot's number, as `{{md.doc.title}}` inserts the frontmatter's title. In an expression, write it without braces: `gardener == "" ? "accent" : "none"`.

A param whose name has a `-`, such as `last-year`, is written with `_` in the body: `{{last_year}}`, because expressions can't use `-` in names.

The body also sees the frontmatter of the document the definition is in, as `md.doc`, but not the reader's inputs (chapter 7): `md.input` in a body is a diagnostic (`F014`). A component gets an input's value as a param instead (chapter 15).

## Using a component

You write your component as any component: as a leaf, or inline, or, once it has slots (chapter 13), as a container. Its params go in braces, with the values its signature says:

```text
:: plot-card[Plot 1] {number=1 gardener="Maya Okafor" crop=garlic}
```

A bare value is read by its type: `number=1` is the number 1, because the signature says `number`. A value can be an expression too, as in the `each` above: `number=(p.number)`.

The brackets hold the fallback, as for any leaf: the text a plain viewer shows, and the page shows if the component can't be drawn (chapter 15). Write what the card says: *Plot 3*, or *Plot 1: Maya Okafor, garlic*.

An **inline** use, `:gardener[Maya Okafor (plot 1)]{…}`, puts the body's text in the sentence. That works when the body is one paragraph, as `gardener`'s is; a body of several blocks can't go in a sentence, and an inline use of it shows its fallback.

The processor checks every use against the signature, and says what's wrong:

| Written | Diagnostic |
| --- | --- |
| `:: plot-crad[…]` | `F002`: no such component, *did you mean `plot-card`?* |
| `colour=red` | `F008`: `plot-card` has no param `colour` |
| `number=one` | `F009`: `number` takes a number |
| no `number` | `F010`: `plot-card` needs `number`, and shows its fallback |
| `::: plot-card` … `::: /plot-card` | `F018`: `plot-card` declares no slots, so it takes no body |

## The component's box

Each use of a component as a block is one box, as each `::: box` is, and the body goes in it. The params after the name on the fence style that box: they take the common params of chapter 3, and `box`'s own, such as `display` and `gap` (Part III). They can be expressions of the component's params: `background=(gardener == "" ? "accent" : "none")` makes a free plot's card stand out.

The params of a use style the same box, and win over the fence's. `:: plot-card[…] {… padding=m}` gives that card more room, and keeps the fence's border and background.

A component that should add no box of its own, such as a name inside a list of cards, has `display=contents` on its fence, and nothing else. Its blocks then sit directly in whatever holds it. Such a component has no box to style, so a box param on its use is ignored, with a warning (`W120`).

## Headings in a body

The card's title is written `# Plot {{number}}`, yet on the page it is a third-level heading. Headings in a body are relative to where the component is used: `#` is one level below the heading above the use, `##` two below, and so on. The cards are under *The plots*, a second-level heading, so their titles are third-level. Used under the document's title, they would be second-level. So the page's outline stays right wherever a component goes. Chapter 15 shows how a component keeps its headings as written.

## Where definitions go

A definition goes anywhere at the top level of the document, not inside a container, and its name is known to the whole document, above and below it. These are near the top, so a reader of the text sees them before their uses; at the end, they would keep the newsletter's text together. Inside a container, such as a `box`, it isn't a definition, and the component's uses are `F002`.

When several documents use the same components, keep the definitions in a file of their own and import them: chapter 15 shows how.

## Components that use components

A body can use any component the document can name: built-in ones, as `plot-card` uses `value`, ones from packs, and your own. So a `gardener-list` could use `gardener` for each name. What a component can't do is use itself, directly or through another component: that would never end, and is `F019`.

## Names

A component's name is made of letters, digits, `-` and `_`, and starts with a letter. Like the built-in ones, names are usually lower case words joined by `-`: `plot-card`, `work-day`.

- Two definitions with one name are `F013`, and the first one is used.
- `if`, `each`, `value` and `box` are CompMD's own, and can't be defined (`F013`).
- A definition named like another built-in component, such as `row` or `grid`, replaces it in this document, with a warning (`W112`), so check that's what you mean.

Next: [slots](13-slots.md), for components with a body you write, in parts.
