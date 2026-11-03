# 16. Using a pack

A *pack* is a set of components someone else wrote and published, for any document to use: diagrams, charts, maps, ratings. This chapter gives the newsletter a diagram of the garden's crop rotation, drawn by a pack from npm, and says what happens when a pack can't be had.

## The newsletter, with a diagram

```md snippet=a-pack/garden.comp.md
---
md:
  version: 1
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
title: Elm Street Garden
issue: November 2026
rotation: [Beans, Cabbages, Roots, Potatoes]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Next year's beds

Each bed moves one step along the rotation every spring, so nothing grows where it grew last year, and the soil gets a rest from each crop.

:: mermaid[Beans, then cabbages, then roots, then potatoes, then beans again] {code=("graph LR; " + md.doc.rotation.join(" --> ") + " --> " + md.doc.rotation[0])}

Your bed's crop next year is the one after this year's.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/a-pack) in the playground. The first time, the diagram's text shows for a moment while the pack arrives; then the diagram is drawn. Add `Onions` to the rotation, and it grows a step.

## Importing a pack

A document says which packs it uses in its frontmatter, under `md.import`, one entry per pack:

```md
---
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
---
```

- **`from`** is the pack's name, `@comp-markdown/mermaid`, then `@` and the version you want. A pack's name always starts with `@` and has a slash in it: `@who/what`, the name it is published under.
- **`names`** lists the components you use from it. Each is then written as any component is, `:: mermaid[…] {…}`, as if it were built in.

Two packs may each have a component of the same name. Give one another name with `as`, and write it by that name:

```md
---
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names:
        - name: mermaid
          as: diagram
---
```

`names: "*"` takes every component the pack offers, under its own names. It's quicker, but a component you write and one the pack adds later may share a name: listing the names you use keeps your document meaning the same.

## Which version

A pack publishes new versions as it improves, and the number after `@` says which ones your document takes:

| Written | Takes |
| --- | --- |
| `@1` | the newest version whose number starts with 1 (1.0, 1.4.2, …), never a 2 |
| `@^0.1.1` | 0.1.1, or a newer 0.1 version |
| `@0.1.1` | exactly 0.1.1 |

The newest version that matches is used, so a document written with `@1` gets the pack's fixes without a change. A pack raises its version's first number when it changes how it is used, so `@1` never takes a version 2 that might draw your document differently. Before version 1, a pack is still settling, and its second number plays that part: `@^0.1.1` stays with 0.1.

A document that must never change, such as a published report, *pins* its pack: an exact version, with the pack's fingerprint as `integrity`. If the pack were ever changed, the fingerprint wouldn't match, and the document would refuse it (`F035`) rather than show something you didn't write. You don't work fingerprints out yourself: tools write them, from the one npm publishes for each version.

```md
---
md:
  import:
    - from: "@comp-markdown/mermaid@0.1.1"
      integrity: sha512-EwevqkkUdxOsFLHV85W74wEL+LoyZhaZkmnj8bJax3zLG48uuiGrezFTixCdlS6VQC/GHVU8Ra0vVXjeFftc7Q==
      names: [mermaid]
---
```

## Where the pack comes from

Your document says which pack it wants; the app that shows it gets the pack. An app gives its documents the packs it was built with, and it may also fetch any pack published on npm the first time a document asks for it, as the playground does. Each app decides which packs its documents may use.

A pack the app can't give is a diagnostic, `F026`, that says why: it isn't published, no version matches the one you wrote, or the app doesn't allow it. The document still shows: each use of the pack's components shows its fallback, the text in brackets. That's why a fallback should say what the component shows (chapter 3): *Beans, then cabbages, then roots, then potatoes* reads well where no diagram can be drawn. A reader with a plain Markdown viewer sees the same text.

## While a pack arrives

A pack fetched from npm takes a moment the first time; after that, the reader's browser keeps it. Until it arrives, the page shows everything else, and the pack's components show their fallbacks. When it's there, the page draws them, without a reload.

## What a pack can do

A pack's components run in a *sandbox*: a box of their own, cut off from everything else. There they see only what your document gives them, their params and their slots, and draw in their place on the page. They can't reach the internet, read the page around them, or see who is reading.

So you can use a pack nobody you know has checked: the worst it can do is draw badly in its own box. An app may trust some packs with more, such as a map that needs to load its tiles, but only the app decides that, never the document: no line in a document can give a pack more than the app does.

## What npm learns

An app with a server of its own fetches packs from there, and npm learns nothing about its readers. An app without one, such as the playground, lets the reader's browser fetch from npm. npm then learns the reader's address and the names and versions of the packs, as a site learns who loads its pictures; never the document, or what it says. Such an app says so: the playground does, in its About.

## Finding packs

Packs on npm carry the keyword `compmd-pack`, so [searching npm for it](https://www.npmjs.com/search?q=keywords%3Acompmd-pack) lists them. A pack's page there, its README, says which components it has and what each one's params are.

While you write, the playground helps too: its Components tab lists every component your document can name, with where each comes from, and a param a component doesn't take is a diagnostic that says which ones it does (chapter 3).

## What's in a pack

A pack is a CompMD document that *exports* components: it lists, in its own frontmatter, the components it offers. Some are written in CompMD itself, as you'll write your own in Part IV; others are written in code, such as this diagram, which draws with a library called Mermaid. To your document they're all the same: a name, params, and a fallback.

Next: [code blocks a pack draws](17-code-blocks-a-pack-draws.md): a pack like this one can also draw a document's ` ```mermaid ` code blocks.
