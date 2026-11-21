# @comp-markdown/badge

A [CompMD](https://compmd.dev/) pack with one component, `badge`: a word or two in a sentence in a small tinted box, "new", "beta", a version, a status. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 6 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The new :badge[beta]{tone=accent} search, from version :badge[2.1].
```

- **The text** is the text in `[…]`.
- **`tone`** is its tint, from the page's colour tokens: `subtle` (the default), `strong` or `accent`. It can come from an expression: `{tone=(stop.status == "open" ? "accent" : "subtle")}`.
- **Its box** is as tall as the text around it, with rounded ends. A use changes it as any box's: `{border=subtle}`, `border-radius`.
- **On paper** it prints as it shows.
- **In a plain Markdown reader,** and on GitHub, the sentence shows as written, the badge's text in it.

## Your own badges, defined once

A project that uses a few badges everywhere (`bug`, `feature`, `chore`) defines them once, each a `badge` with its look fixed, in a file its documents import. `labels.comp.md`:

````md
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
  export:
    names: [bug, feature, chore]
---

```compmd component=bug
---
:badge[**{{md.fallback}}**]{background="#ffebe9" border="1px solid #cf222e"}
```

```compmd component=feature
---
:badge[**{{md.fallback}}**]{background="#dafbe1" border="1px solid #1a7f37"}
```

```compmd component=chore
---
:badge[*{{md.fallback}}*]{background=subtle border=strong border-radius=s}
```
````

A document imports them and writes each with nothing to configure:

```md
---
md:
  import:
    - from: ./labels.comp.md
      names: [bug, feature, chore]
---

- :feature[new] Saved searches, from the search page.
- :bug[bug] The tide table showed yesterday's tides after midnight.
- :chore[chore] The photos are smaller, so the pages load faster.
```

Each badge's text is still the text in `[…]`, so a plain Markdown reader shows `new`, `bug` and `chore` in the lines. Published as a pack of its own, the same file serves several sites.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/badge dev     # the test page: demo.comp.md, each use as an example
pnpm --dir compmd-packs/badge test    # the publish check, and the badge's expanded tree
pnpm --dir compmd-packs/badge lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
