+++
id = "01a1fe83-8af0-77e6-9d15-12fa5b05d653"
number = 46
title = "Component: figure"
status = "approved"
tasks = ["CMD-316"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T14:29:56Z
updated = 2026-11-21T14:32:09Z
+++

`@comp-markdown/figure`, item 5 of [the first components](docs/first-components.md), as [the no-code components](docs/no-code-components.md) write it (decisions 1 and 2):

```compmd component=figure {padding-block=s}
params:
  src: string
---
![{{md.fallback}}]({{src}})

{{md.fallback}}
```

- **The caption is the use's fallback,** `:: figure[Boats at the quay, the lamps just lit.] {src="images/harbour.jpg"}`, placed under the image and as its alt (`md.fallback`, CMD-360). A plain reader shows the caption where the figure is.
- **`src` is the caller's** (decision 1, CMD-362): a test puts the document in `posts/` and checks the image stays `images/harbour.jpg`, never a pack's file.
- **Print:** a `compmd` definition prints as it renders (spec §7.3, §14), so the figure needs nothing for paper. A test checks that its expanded tree is a `box` with an image and a paragraph, with no component left for a host to draw.

## The rest

A demo (two figures in a short post, one 20rem wide); tests of the box and its params under the use's, the image's alt and the caption, the caller's path, print, the plain reading and the demo. The pack is in the workspace and `compmd-packs/versions.json` at 0.1.0.

## To check

The caption is a plain paragraph, the same size as the text. A smaller or muted caption would need a text-size or colour param on `box`, which the common params don't have.
