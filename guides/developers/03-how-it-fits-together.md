# 3. How it fits together

The quick start used four packages in a few lines. This chapter is what each does, where it runs, and why it is split that way, so the parts after it make sense.

## The processor: text in, trees out

`@compmd/processor` takes a **request**, a document and everything it needs, and gives back what a page needs. It runs in stages, each taking only what it uses:

| Stage | Needs | Gives |
|---|---|---|
| Parse | the document's text | the parse tree; syntax diagnostics |
| Resolve | the files it imports, its packs | every component name bound to a definition |
| Validate | signatures | params checked and typed |
| Load | its data, the inputs' values | the data |
| Evaluate | all of the above, and the time | the expanded tree: content only, ready to render |

The processor does no I/O. Your application puts everything in the request. That's why it runs the same in a browser, a worker, a build or natively, and why opening a document never makes it fetch anything on its own. Only `path` and `now` are required:

| Field | Holds | The stage that reads it |
| --- | --- | --- |
| `path` | the document's path in `files` | all |
| `files` | the files of the document's folder by path: the document, what it imports, its data files | parse, resolve, load |
| `packs` | the packs by name, each with its releases and their files | resolve |
| `ambient` | packs every document gets without importing them | resolve |
| `policy` | your packs' trust, which packs a document may import, which are denied | resolve |
| `allowedOrigins` | the origins remote data may come from | validate |
| `remote` | remote data, fetched beforehand, by URL | load |
| `host` | your app's own data, for `host:` sources | load |
| `inputs` | the inputs' values, by name | load, evaluate |
| `now`, `tz` | the time the page is made, RFC 3339, and the time zone of a document without one | evaluate |
| `origins` | whether expanded nodes say where in the text they come from, as a live preview needs | evaluate |

`virtual:compmd`'s `request` fills `packs`, `ambient` and `policy` from your configuration; the rest is the document's. `sources(request)` lists what a render needs that the request lacks, remote URLs, `host:` sources and packs, so you can get them first. [Processing documents](04-processing-documents.md) and [data and inputs](06-data-and-inputs.md) go through them.

Parsing needs the text alone, so an editor or a linter can parse any document, even one whose packs and data it doesn't have.

`process(request)` runs every stage once. `OpenDocument` keeps the document open: each edit processes only the blocks it touches, and gives a *splice*, what changed in the expanded tree, for an editor's live preview.

## Diagnostics

Every stage reports what it finds wrong with the document as diagnostics, and none stops the others: a broken component renders its fallback, and the rest of the page is there. A request that isn't valid is the only thing that throws. Each diagnostic is the spec's machine form:

| Field | Is |
| --- | --- |
| `code` | `F…` for an error, which changes what renders; `W…` for a warning, which doesn't |
| `severity` | `error` or `warning`, from the code |
| `message` | a sentence for the writer; not stable, so match on `code` |
| `range` | where: 1-based lines and columns, in code points |
| `file` | the file it is in, when it isn't the processed document, such as a pack's document |
| `related` | other places it involves, such as the opener of a container never closed |
| `stage` | the stage that found it: `parse`, `resolve`, `validate`, `load` or `evaluate` |
| `fix` | when there is one, the edit that fixes it, for a quick fix |

The processor gives them in stage order, with its result. Two come later, from the sandbox, since only running a component's code finds them: `W118` for a component that threw or failed, and `W121` for code that can't load (the processor reports `W121` itself only for a pack whose `peerDependencies` your frames can't meet). They arrive through the view's `error` event, `view.on("error", ({ component, code, message }) => …)`. The [diagnostics appendix](22-the-diagnostics.md) lists every code.

## The expanded tree, and the components

The expanded tree is content: headings, paragraphs, lists, boxes, input components, and the components written in code, by name and params. It holds no code. The code of each component comes beside it, in `components`: a fence's or a file's text to compile, or a built module's place in its pack, with the trust its pack is given.

## The renderer and the sandbox: trees in, a page out

`@compmd/render` turns a tree into a live page and updates it in place: a new tree, or a splice, changes only what changed, and components keep their state.

`@compmd/sandbox` decides **where** each component's code runs, by the trust its pack is given:

| Trust | Where it runs | What it can reach | Who |
|---|---|---|---|
| **strict** (the default) | a frame with an opaque origin and no network | its params, its slots, its box | any pack, any document |
| **permissive** | a frame of its own | that, plus the network to origins you list, and `host` if you give it | packs your configuration lists |
| **trusted** | your page | what your page can | your app's own components, built with it |

A document can never raise a pack's trust: only your configuration can. So a document from anyone, and any pack it imports, can't reach your page, your session or your data.

The sandbox renders a document in one of two modes:

- **Frame mode** (the default, as in the quick start): the whole page in one strict frame. Simple, and right for documents from anyone.
- **Host mode**: the page in your page, your trusted components inline, every other component in a frame of its own. Right for an app with components of its own.

| | Frame mode, `page: "frame"` | Host mode, `page: "host"` |
| --- | --- | --- |
| The page | one strict frame per document | your page |
| Strict components | in the page's frame | a frame per use |
| Permissive components | their fallbacks, with an error `not-here` | a frame per use, with `host` if your configuration gives it |
| Trusted components | their fallbacks, with an error `not-here` | inline, in your page |

Frames only go down in trust: your page holds permissive and strict frames, a permissive frame holds strict ones, and a strict frame holds none. So a permissive or trusted component in the slot of a strict container shows its fallback too. Which level each component gets comes with it from the processor, in `components`, set by your configuration; the sandbox only carries it out. [Components and trust](08-components-and-trust.md) and [the sandbox, closer](11-the-sandbox-closer.md) say more.

## The plugin: your app's side

`@compmd/vite` does at build time what every app would otherwise write:

- it reads **`compmd.config.json`**: which packs get more than strict trust, which are ambient (every document gets their names), which are denied;
- it finds your **packs**, in `node_modules` and in folders of your app, and gives them to the processor as `request.packs`;
- it builds the **bundle**, the sandbox's frames and your packs' files, and serves it from your app under `compmd/`, with the headers the frames need, or writes it into a folder a desktop app's plugin serves ([a Tauri app](19-a-tauri-app.md));
- it builds your **trusted components** with your app, and gives them to the sandbox as `sandbox.trusted`.

`virtual:compmd` hands both to your code, `request` for the processor and `sandbox` for the sandbox, so you spread them in and write no policy, path or import map yourself.

## Why it is split this way

- **The processor knows nothing of the browser**, so it can check documents in CI, render them at build time, or run in a native app.
- **The renderer knows nothing of trust**, so a host that renders documents it trusts completely can use it alone.
- **The sandbox carries the security**, in one place, tested against every threat it guards from, so no application writes a frame or a content security policy itself.
- **The plugin is the only part that knows your build**, and other hosts can do its job by hand, as [other hosts](18-other-hosts.md) shows.

Next: Part II, [processing documents](04-processing-documents.md).
