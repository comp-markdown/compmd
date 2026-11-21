---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
  export:
    pack: "@comp-markdown/link-card"
    version: "0.1.0"
    names: [link-card]
---

# @comp-markdown/link-card

`link-card` is a card that is a link: a title, a line about where it goes, and a click anywhere on it follows the link. For a post's "read next", a list of a newsletter's stories, a page of a trip's walks.

```md
:: link-card[The harbour at dusk] {href="harbour.comp.md" description="A walk along the old quay, an hour before the boats come in."}
```

The title is the text in `[…]`, the use's fallback, and the link's text: a screen reader names the link by it, and a plain Markdown reader shows it in the use's line. `href` is your document's: `harbour.comp.md` is next to the document that uses the card, wherever the card comes from. `description`, which a use may leave out, is the line under the title. The title is a heading one level below the heading above the card.

Its box is a `card`'s: a subtle border, rounded corners and some padding. A use changes it as any box's, `{width=20rem}`; cards in a `grid` make a row of them that wraps.

```compmd component=link-card {stretch-link}
params:
  href: string
  description:
    type: string?
    doc: The line under the title
---
::: card {fill}
# [{{md.fallback}}]({{href}})

::: if {when=(description != null)}
{{description}}
::: /if
::: /card
```
