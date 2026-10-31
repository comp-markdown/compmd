---
title: Packs
steps: [Idea, Draft, Review, Published]
md:
  import:
    - from: "@preview/minimermaid@0"
      names: [minimermaid]
---

# Diagrams from a pack

This document imports a pack, [`@preview/minimermaid`](#:~:text=@preview/minimermaid@0), as a document imports any pack: by its name and a version. Its one component, `minimermaid`, draws Mermaid's simplest flowcharts by hand, and it renders the ` ```mermaid ` blocks of a document that imports it. The tabs above the text show the pack's document, with the component's signature.

The pack was made with `npm create @compmd/pack minimermaid -- --renders mermaid`. Its component is one built module: a JavaScript file with everything it uses bundled in, which the playground serves next to the page. The sandbox loads it by its URL, and the browser runs it only if its file matches the hash the playground gives, so a module changed on its way runs nowhere.

```mermaid {alt="An idea becomes a draft, then it's reviewed, then published"}
graph LR; Idea --> Draft --> Review --> Published
```

A plain renderer, and GitHub, show the block as they always have; GitHub draws it with all of Mermaid.

## Branches, and labels

A step's label goes in brackets after its name, and a step can lead to several:

```mermaid {theme=neutral alt="A request is triaged, then fixed or declined; a fix is shipped"}
graph TD
  request[A request] --> triage[Triage]
  triage --> fix[Fix it]
  triage --> decline[Decline]
  fix --> ship[Ship]
```

## From data

A code block is never interpolated, so a diagram made from data is the same component written as a leaf, with its code as a param. This one is made from the `steps` key of the frontmatter:

:: minimermaid[The steps: Idea, Draft, Review and Published] {code=("graph LR; " + md.doc.steps.join(" --> ")) theme=dark}

> [!TIP]
> **Try it.** Add `Archived` to [`steps: [Idea, Draft, Review, Published]`](#:~:text=steps:%20%5BIdea), and the last diagram grows a step. Then misspell the import's name, [`@preview/minimermaid@0`](#:~:text=@preview/minimermaid@0): the pack isn't there, so every block shows its code and the leaf its fallback, with a diagnostic.
