---
md:
  export:
    pack: "@comp-markdown/figure"
    version: "0.1.0"
    names: [figure]
---

# @comp-markdown/figure

`figure` is an image with its caption under it: a post's photo, a newsletter's chart, a recipe's dish. The caption is written once, and is the image's text alternative too.

```md
:: figure[Boats at the quay, the lamps just lit.] {src="images/harbour.jpg"}
```

The caption is the text in `[…]`, the use's fallback: a plain Markdown reader shows it where the figure is, and the image's alt is its text. The image is your document's: `images/harbour.jpg` is next to the document that uses the figure, wherever the figure comes from. Its box has a little space above and below; a use changes it as any box's, `{width=20rem}`. It prints as it shows.

```compmd component=figure {padding-block=s}
params:
  src: string
---
![{{md.fallback}}]({{src}})

{{md.fallback}}
```
