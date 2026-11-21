---
md:
  export:
    pack: "@comp-markdown/badge"
    version: "0.1.0"
    names: [badge]
---

# @comp-markdown/badge

`badge` is a word or two in a sentence, in a small tinted box: "new", "beta", a version, a status.

```md
The new :badge[beta]{tone=accent} search, from version :badge[2.1].
```

The badge's text is the text in `[…]`, the use's fallback, so a plain Markdown reader shows it, in the sentence as written. `tone` is its tint, from the page's colour tokens: `subtle` (the default), `strong` or `accent`. It is as tall as the text around it, with rounded ends; a use changes its box as any box's, `{border=subtle}`. It prints as it shows.

```compmd component=badge {padding-inline=s border-radius=full background=(tone)}
params:
  tone:
    type: subtle | strong | accent
    default: subtle
    doc: The badge's tint, from the page's colour tokens
---
{{md.fallback}}
```
