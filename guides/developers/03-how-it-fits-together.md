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
| Expand | all of the above, and the time | the expanded tree: content only, ready to render |

The processor does no I/O. Your application puts everything in the request: `files` (the document and what it imports), `packs`, `remote` and `host` data, `inputs`, `now`. That's why it runs the same in a browser, a worker, a build or natively, and why opening a document never makes it fetch anything on its own.

`process(request)` runs every stage once. `OpenDocument` keeps the document open: each edit processes only the blocks it touches, and gives a *splice*, what changed in the expanded tree, for an editor's live preview.

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

## The plugin: your app's side

`@compmd/vite` does at build time what every app would otherwise write:

- it reads **`compmd.config.json`**: which packs get more than strict trust, which are ambient (every document gets their names), which are denied;
- it finds your **packs**, in `node_modules` and in folders of your app, and gives them to the processor as `request.packs`;
- it serves the sandbox's **frame documents** and the packs' **built modules** from your app, under `compmd/`, with the headers the frames need;
- it builds your **trusted components** with your app, and gives them to the sandbox as `sandbox.trusted`.

`virtual:compmd` hands both to your code, `request` for the processor and `sandbox` for the sandbox, so you spread them in and write no policy, path or import map yourself.

## Why it is split this way

- **The processor knows nothing of the browser**, so it can check documents in CI, render them at build time, or run in a native app.
- **The renderer knows nothing of trust**, so a host that renders documents it trusts completely can use it alone.
- **The sandbox carries the security**, in one place, tested against every threat it guards from, so no application writes a frame or a content security policy itself.
- **The plugin is the only part that knows your build**, and other hosts can do its job by hand, as [Part V](README.md#contents) will show.

Next: Part II, documents in your app *(to come)*.
