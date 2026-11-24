# @comp-markdown/wireframe

A [CompMD](https://compmd.dev/) pack that draws wireframes from ```` ```wireframe ```` blocks: a screen written as an
indented outline, one part per line, drawn as HTML and CSS, light and dark, in the host's strict sandbox. The language
is [`LANGUAGE.md`](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/wireframe/LANGUAGE.md), published with the
pack; the design is [docs/wireframe-pack.md](https://github.com/comp-markdown/compmd/blob/main/docs/wireframe-pack.md).

## Use it in a document

````md
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

```wireframe {alt="The sign-in screen: email, password, and a Sign in button"}
window "Sign in":
  header:
    text "Welcome back" bold
  panel:
    input placeholder="Email" id=email
    input placeholder="Password" type=password
    row justify=between:
      checkbox "Remember me"
      button "Sign in" primary
annotation "Shows an error under the field that failed" target="email" position=right
```
````

- **A block** of ```` ```wireframe ```` is drawn; its `alt` is the drawing's text alternative. A reader without the
  pack, and GitHub, see the code.
- **A leaf** takes the source as its `code` param, for a screen made from data or inputs: a code block is never
  interpolated.
- **The parts** (`window`, `row`, `col`, `grid`, `panel`, `section`, `tabs`, `list`, `tree`, `menubar`, `text`,
  `button`, `input`, `combo`, `checkbox`, `radio`, `toggle`, `progress`, `image`, `icon`, `annotation` and the rest)
  are in the language; a window is a desktop window, or a phone, a tablet or a browser with `device=`.
- **The look** is a sketch's: greys from the host's tokens (`--compmd-border-strong`, `--compmd-surface-subtle`, …),
  built from the text colour when the host has none, so it suits light and dark; a page's hand-written font when it
  sets `--compmd-wireframe-font`. It prints as it shows.
- **A source with a mistake** shows its code, and the host reports the first mistake as W118, with its line and column
  in the block (`line 2, column 3: no part `butn`: did you mean `button`?`).

## Size

`dist/wireframe.js` is 28 KB, 9 KB gzipped (measured 2026-11-17): the parser, the drawing and its stylesheet, with no
library.

## Licences

The pack is MIT. It bundles nothing of anyone else's.

The language is based on [Wireloom](https://github.com/StardockCorp/Wireloom)'s syntax as its examples showed it on
2026-11-17 (MIT, Stardock); its grammar, its parser and its HTML and CSS drawing are this pack's own.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/wireframe dev     # the test page, demo.comp.md in host mode, each wireframe in a strict frame, with a light and dark switch
pnpm --dir compmd-packs/wireframe test    # the build, the parser and the drawing in happy-dom, and the test page in Chrome and WebKit
pnpm --dir compmd-packs/wireframe lint    # the type check
```

The build, its checks and the test page are [`@compmd/pack`](https://github.com/comp-markdown/compmd/blob/main/npm-packages/pack/README.md)'s,
as for any pack.

## Releases

The pack has its own version, for its language and its component's params.

- **0.1.0:** the first version, published by `release-wireframe.yml`. (`0.0.0` is a placeholder with no code.)
- **0.1.1:** a part or an attribute named after what every object inherits (`constructor`, `toString`, …) is a
  mistake, as any unknown name is; the first version staged by `release-packs.yml`.
