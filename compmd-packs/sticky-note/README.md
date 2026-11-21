# @comp-markdown/sticky-note

A [CompMD](https://compmd.dev/) pack with one component, `sticky-note`: a note pinned on the page, tinted, tilted a little, with a shadow, for a reminder beside a walk's directions, a tip in a recipe, a correction in a newsletter. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 7 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=4rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
```

- **The note** is what you write in it.
- **Where it sits:** without a position, where it is written. `inset-*` pins it in its nearest box, over what is there (spec §9.2); the box is as tall as what stays in its flow. Written from `sm` up, as above, it is pinned where its box is wide and in the flow on a phone.
- **Its box** is 14rem wide, never wider than its column, tinted `accent`, tilted `-2deg`, with a shadow. A use changes it as any box's: `{rotate=3deg background=subtle width=10rem}`.
- **On paper** it prints as it shows.
- **In a plain Markdown reader,** and on GitHub, a note is its text.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/sticky-note dev     # the test page: demo.comp.md, each use as an example
pnpm --dir compmd-packs/sticky-note test    # the publish check, the expanded tree, and the page in Chrome and WebKit
pnpm --dir compmd-packs/sticky-note lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
