---
title: Data from the web
md:
  data:
    - from: https://registry.npmjs.org/@comp-markdown/mermaid/latest
      as: pack
      fallback: ./pack.json
---

# Data from the web, asked first

This document reads the latest release of the Mermaid pack from npm's registry, `registry.npmjs.org`. The playground isn't configured to fetch data from there, so the first time a document needs it, the playground asks you, and remembers your answer for every document.

Until you answer, and if you don't allow it, the page shows the document's `fallback` instead: `pack.json`, a copy saved beside it. The diagnostics say why (F028).

**{{md.data.pack.name}} {{md.data.pack.version}}**: {{md.data.pack.description}}

## What npm learns

Your browser asks npm for the release itself: npm learns your address and that a page asked for this pack's latest release, never the document or what it shows. A document whose `from` holds inputs would send their values too, which is why the question says so.

> [!TIP]
> **Try it.** Allow it, and the version and description above come from npm. Write another pack's name in place of [`@comp-markdown/mermaid`](#:~:text=@comp-markdown/mermaid/latest) in the `from`: the origin is the same, so you aren't asked again. Write a site the playground hasn't asked about, `https://api.github.com/repos/comp-markdown/compmd`, as `as: repo`, and you are asked about that one.
