# 18. Components in code, in brief

Some components are written in code: in JSX for React, in Svelte, or in plain JavaScript. A pack's diagram is one; you may also get one from a friend who programs, or write one yourself. This chapter shows what such a component looks like to the document that uses it: where its code goes, how the document knows its params, what shows when it can't run, and what the sandbox lets it do. The newsletter gets a bar for each crop of the harvest, from a file, and a map of the plots, from the document itself. Writing the code is [the Developer's Guide](../developers/README.md)'s subject.

## The newsletter, with components in code

````md snippet=in-code/garden.comp.md
---
md:
  version: 1
  import:
    - from: ./harvest.jsx
      names: [harvest]
title: Elm Street Garden
issue: November 2026
harvest:
  - {crop: Beans, kilos: 42}
  - {crop: Leeks, kilos: 25}
  - {crop: Kale, kilos: 18}
plots:
  - {number: 1, gardener: Maya Okafor}
  - {number: 2, gardener: Tomás Reyes}
  - {number: 3, gardener: ""}
  - {number: 4, gardener: Priya Nair}
  - {number: 5, gardener: ""}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## This year's harvest

:: harvest[Beans 42 kg, leeks 25 kg, kale 18 kg] {rows=(md.doc.harvest) unit=kg}

## The plots

:: plot-map[Plots 1, 2 and 4 are taken; plots 3 and 5 are free] {plots=(md.doc.plots)}

```compmd component=harvest signature
params:
  rows: array
  unit: string = kg
```

```svelte component=plot-map runtime=svelte@5
<script>
  let { plots, compmd } = $props();
</script>

<div role="img" aria-label={compmd.fallback.text} style="display: flex; gap: 0.5rem">
  {#each plots as plot}
    <span style="padding: 0.5rem 0.75rem; border: 1px solid; opacity: {plot.gardener ? 1 : 0.4}">{plot.number}</span>
  {/each}
</div>
```

```compmd component=plot-map signature
params:
  plots: array
```
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/in-code) in the playground. Give plot 5 a gardener, and its square on the map no longer looks faded.

The harvest's bars are drawn by a React component in a file beside the document:

```jsx snippet=in-code/harvest.jsx
export default function Harvest({ rows, unit, compmd }) {
  const most = Math.max(...rows.map((row) => row.kilos));
  return (
    <figure role="img" aria-label={compmd.fallback.text} style={{ margin: 0 }}>
      {rows.map((row) => (
        <div key={row.crop} style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
          <span style={{ width: "4rem" }}>{row.crop}</span>
          <span style={{ width: `${(row.kilos / most) * 60}%`, height: "0.75rem", background: "currentColor" }} />
          <span>
            {row.kilos} {unit}
          </span>
        </div>
      ))}
    </figure>
  );
}
```

You don't need to read the code. What matters to the document is in the document: each component's name, its params, and its fallback.

## The same as any component

To the text that uses it, a component in code is written as every other: `:: harvest[…] {…}` is a leaf with a fallback and params, as `:: mermaid[…] {…}` is in chapter 16. Its params can be expressions (chapter 4), it takes the common params for its box (chapter 3), and it can sit in an `each` or an `if` (chapter 5). A component in code can be a container too, with slots, when its code places them.

Where its code lives is the only difference, and there are two places: a file the document imports, or a fence in the document itself.

## A component in a file

```text
md:
  import:
    - from: ./harvest.jsx
      names: [harvest]
```

An import whose `from` is a file of code, beside the document or in a folder below it, gives one component, named after the file: `./harvest.jsx` gives `harvest`, and `./PlotMap.svelte` would give `PlotMap`. Names aren't changed between cases; give it another with `as`, as for a pack (chapter 16).

The file's ending says its language:

| Ending | Written in | Runs on |
| --- | --- | --- |
| `.jsx`, `.tsx` | JSX or TSX, for React | React 19 |
| `.svelte` | Svelte | Svelte 5 |
| `.js`, `.ts` | plain JavaScript or TypeScript | nothing more: plain code |

A file keeps the code out of the text, so a reader of the plain file, on GitHub or anywhere else, sees only the leaf's line with its fallback.

## A component in a fence

````text
```svelte component=plot-map runtime=svelte@5
…the component's code…
```
````

A fenced code block whose first word is a language of code, followed by `component=` and a name, defines a component in the document. The page doesn't show the fence; a plain Markdown viewer shows it as code. The fence can go anywhere in the document, at its top level, and the name works everywhere in it.

`runtime=` says which framework the code is written for, and which version:

| Language | `runtime` | When left out |
| --- | --- | --- |
| `jsx`, `tsx` | `react@19` or `preact@10` | `react@19` |
| `svelte` | `svelte@5` | `svelte@5` |
| `js`, `ts` | none: plain code takes no `runtime` | |

Writing `runtime` keeps the component meaning the same when a later version of CompMD moves the default to a new version of the framework. A runtime the app doesn't have, such as `react@18`, or any `runtime` on a `js` or `ts` fence, is `W102`: the component shows its fallback.

A fence in the document is handy while you try a component out, or for one only this document uses. One used by several documents is better in a file, or in a pack.

## Its signature

CompMD can't read the code to learn which params a component takes, so a *signature fence* says it, after the language `compmd` and the word `signature`:

````text
```compmd component=harvest signature
params:
  rows: array
  unit: string = kg
```
````

It lists the params with their types and defaults, and its slots, the same way as the signature of a definition written in CompMD (Part IV). A signature fence attaches to the component of its name, whether the code is in a fence of the document or in a file it imports. The page doesn't show it.

With a signature, the component's uses are checked like any other's: a param it doesn't take is `F008`, a value of the wrong type `F009`, a required param left out `F010`, and a default fills in what you leave out (`unit` is `kg` unless you say otherwise). Without one, the params go through unchecked, so a misspelt name is never caught; for a component in a fence, that is `W105`, which a signature fence answers. A pack always has signatures for its components, which is why the playground can say what a pack's component takes.

## Its fallback

A component in code needs something to run it: an app that shows CompMD in a browser, with the right framework. Wherever that isn't there, the leaf's fallback shows instead:

| Where | What shows |
| --- | --- |
| a plain Markdown viewer, GitHub | the leaf's line, with its fallback |
| an app without the framework or its version (`W102`) | the fallback |
| an app that makes something other than a web page, such as a PDF (`W102`) | the fallback |
| an app where the code fails to draw (`W118`) or can't load (`W121`) | the fallback |
| an app that runs it | the component |

So the fallback should say what the component shows, as a sentence: *Beans 42 kg, leeks 25 kg, kale 18 kg* tells a reader everything the bars do. A component that draws a picture also gets the fallback as text for screen readers, as both components above use it.

## What code in a document can do

A component in code runs in a *sandbox*, as a pack's does (chapter 16). How much it may do is its *trust level*, which only the app that shows the document sets, never the document:

| Level | Reaches | For |
| --- | --- | --- |
| strict sandbox | only its params, its slots and its own box | the default: code in documents, and every pack the app gives no other level |
| permissive sandbox | the same, plus the internet addresses the app lists for its pack, and what the app gives it | packs the app chooses, such as a map that loads its tiles |
| trusted | what the app's own code reaches | components built into the app itself |

Code you write in a document, in a fence or a file beside it, always runs in the strict sandbox. For a writer, that means:

- **It sees only what you give it.** Its params and its slots, nothing else of the page or of the reader. Give it the values it needs as params, as the newsletter gives `harvest` its rows.
- **It has no internet.** It can't fetch a forecast or a picture. Data it needs comes through the document's data (Part II), into a param.
- **It can't open links or windows on its own.** The app may offer to open a link it shows, after showing the reader where it goes.
- **It can't harm the reader.** The worst it can do is draw badly in its own box, so a document with code in it is safe to open from anyone.

## Sharing a component in code

A document that offers its components to others lists them under `md.export`, as a pack does. A component in code that is exported needs a signature fence, `F029` otherwise, since documents that import it must be checked where its code may not run. A component in a fence that is exported should also name its `runtime`: without one, it is `W117`, since its meaning would change when the default moves. Plain JavaScript takes no `runtime`, and has no `W117`.

## Writing one

The code itself is [the Developer's Guide](../developers/README.md)'s: what a component is given, and how it draws.

- [Components and trust](../developers/08-components-and-trust.md): the trust levels, and who decides them.
- [Your app's own components](../developers/10-your-apps-own-components.md): a component in React or Svelte, and its signature.
- [Your first pack](../developers/13-your-first-pack.md) and [signatures and params](../developers/14-signatures-and-params.md): a pack of components, with their types, defaults and slots.
- [React, Svelte and libraries](../developers/15-react-svelte-and-libraries.md): the frameworks, and the libraries a component may use.
- [The sandbox, closer](../developers/11-the-sandbox-closer.md): how the sandbox is built.

Next: the appendices, starting with [the syntax at a glance](19-the-syntax-at-a-glance.md), everything a writer types in one place.
