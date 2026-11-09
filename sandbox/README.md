<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/sandbox

Runs a CompMD document's components in a browser at the trust levels of [spec §15](../compmd-spec-v1.md#15-security): strict code in frames with no network, permissive code in frames that reach only their pack's origins, and the host's own trusted code in its page. A host gives it trees and its configuration; it never writes a frame or a content security policy itself. The design is [running components in the sandbox](../docs/running-components-in-the-sandbox.md), its messages [docs/sandbox-protocol.md](../docs/sandbox-protocol.md).

**Website:** [compmd.hakanserce.com](https://compmd.hakanserce.com/), with the spec and a playground to try CompMD in.

```sh
npm install @compmd/sandbox @compmd/render @compmd/processor
```

It implements the draft of the [CompMD specification](https://compmd.hakanserce.com/spec/), version 1, and shares its version with `@compmd/render` and `@compmd/processor` ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.5.0:** where each line is (`view.lines()`, and the `layout` event); when the page has settled (`view.settled()`, the `settled` event, `data-compmd-settled` on the container), each frame reporting up the tree; Shiki and Temml shipped once, a host-mode page loading the frames' `highlight.js` and `math.js`, so the app's own bundle has neither. Breaking: the frames speak `@compmd/render`'s frame protocol version 4, so a host's page and the frame documents it serves are upgraded together.
- **0.4.0:** code blocks that show as code highlighted by Shiki, in light and dark (`shikiHighlight`, and CompMD's own grammar as `@compmd/sandbox/grammar`); where the page came from, through the view (`view.reveal`, `view.rectsAt`, `pointed`); a document's own images in frame mode, fetched by the host's page; the reader asked before data from a new origin, and the answer remembered (`loadData`); math typeset in host mode too; `host` for trusted components, as permissive ones get it, and `view.emit`; each failure naming its use, and `recovered`. Breaking: the frames speak `@compmd/render`'s frame protocol version 3, so a host's page and the frame documents it serves are upgraded together.
- **0.3.0:** the observer, for developer tools (`view.observe`).
- **0.2.1:** a frame the browser doesn't render (below the fold, or too small to see) sized to its content.
- **0.2.0:** the first version.

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # ESM and types into dist/, and the frame runtime and shared libraries into dist/frames/
pnpm test            # unit tests in Node, and browser tests in Chrome and WebKit (build first)
pnpm lint            # eslint and the type check
```

The browser tests drive the installed Chrome and Playwright's WebKit (`pnpm exec playwright install webkit`).

## Serving the frame documents

The frames load the sandbox's own documents, which the host serves, from its origin or another:

```ts
// at build time, in Node
import { buildFrames, buildLibraries, readLibraries, RUNTIMES } from "@compmd/sandbox/build";

const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root, out: "public/sandbox/libs", shared: RUNTIMES });
const libraries = await readLibraries("public/sandbox/libs");
const definition = await buildFrames({ out: "public/sandbox", imports, libraries });
// definition: { runtimes: [{ name: "react", version: "19.3.0", imports: […] }, …], libraries: [{ name: "d3", version: "7.9.0", imports: ["d3"] }] }

// The processor's request: it reports a runtime the frames don't have (W102) and a pack's unmet peerDependencies (W121).
const named = ({ name, version }) => ({ name, version });
request.policy = { ...request.policy, runtimes: definition.runtimes.map(named), libraries: definition.libraries.map(named) };
```

`buildFrames` writes `strict.html` and `permissive.html`, each with its policy and the import map, next to the frame runtime and the runtimes of the spec's bindings (React 19, React DOM, Svelte 5). `buildLibraries` adds libraries the host offers its documents' components; without it they have the runtimes alone. What component code may import is spec §7.6: these runtimes and libraries, and nothing else. `buildFrames` gives the frames' **library definition**, each runtime and library with its version and import names, whose names and versions are the processor's `policy.runtimes` and `policy.libraries`. It writes it to `library-definition.json` there too, which `libraryDefinition` reads, for a host that builds its frames ahead and makes requests later. A pack's build leaves out `RUNTIMES`' import names. The documents work wherever they are served. A host SHOULD serve them with the header `Content-Security-Policy: sandbox allow-scripts`, so they are sandboxed even when opened outside a frame.

**On a scheme of the host's own** (a desktop app's, such as Tauri's custom URI schemes), a host names where it serves the frames: `buildFrames({ out, source: "app://localhost/frames/" })`. A frame is sandboxed, so its origin is opaque, and WebKit (Safari, macOS and Linux webviews) reads `'self'` in it as the frame document's origin only over http and https: on any other scheme `'self'` matches nothing, and the frame can't load its own runtime. With `source`, the frames' policies name it beside each `'self'` (scripts, styles and fonts), and a permissive frame's second policy does too. It is a scheme, a host and a path, never a bare scheme, so the frames load nothing else the host serves on that scheme; `frameSource` checks one. Over http and https it isn't needed.

## A document's page

```ts
import { createSandbox } from "@compmd/sandbox";
import { react } from "@compmd/render/react";

const sandbox = createSandbox({
  base: "/sandbox/",                          // where the frame documents are served
  page: "host",                               // or "frame", the default
  trusted: { stars: react(Stars) },           // the host's own components; host mode only
  host: { rating: (product) => ratings.get(product) },   // what trusted components, and permissive packs given `host`, can call
  modules: (module) => ({ url: `/packs/${module.pack}@${module.version}/${module.path}`, integrity: hashes[module.path] }),
});

const view = sandbox.open(container, { components: processed.hostComponents });
view.render(processed.expanded);              // or view.splice(splices, change, { replace, inputs })
view.on("set", ({ input, value }) => rerender({ [input]: value }));
view.on("link", ({ url }) => confirmAndOpen(url));   // a reader's click; show the URL before opening it (spec §15)
view.on("error", ({ component, reason, code, message, use }) => report(use, component, code ?? reason, message));   // code: W118 (threw or failed), W121
view.on("recovered", ({ use }) => unreport(use));   // a use that failed draws again
view.components(next.hostComponents);         // when the document's components change
view.styles(["/sandbox/docs.css"]);           // the host's stylesheets, served from the frames' origin
view.theme(tokens);
view.scheme();                                // after the host switches its own theme, with a class
view.emit("ratings-changed", { product: "p-1" });   // an event to every component given `host`
view.reveal(offset);                          // scroll to what the text at a code-point offset drew, and mark it
const boxes = await view.rectsAt(offset);     // its boxes, in the host's viewport
view.on("pointed", ({ source }) => moveCaret(source));   // what the reader clicked came from there; asks for clicks
view.on("layout", async () => {
  for await (const { line, rects } of view.lines()) place(line, rects);   // where each line is, from the container's top left
});
view.destroy();
```

**`host`** (spec §7.4) is the same for a trusted component in the host's page and a permissive one in its frame, so one component works at either level: `host.call(name, ...args)` calls the host's function of that name and resolves with what it returned (the arguments and the result must be copyable, since a frame's cross a port), and `host.on(event, listener)` hears what `view.emit(event, data)` sends, until the function it returns is called. A name the host doesn't give fails the call. Strict components never get `host`, nor hear its events. `HostClient` is its type, for a component written in TypeScript.

**Failures and recoveries, by use.** An `error` names the use that shows its fallback in `use`, an opaque id, when it has one: each window gives each mounted use its own, and a frame's parent puts the frame's key in front, so ids from different frames never meet. When a use that failed (W118, by `fail` or a throw in `update`) draws again after its next update, `recovered` comes with the same `use`, so a host that lists failures removes the one it listed, and keeps another use's of the same component. A frame's code could send `recovered` itself; it only clears a listing.

**Light and dark.** Every frame takes the colour scheme of the page around it: the view reads its container's `color-scheme` (and the reader's preference, where it allows both) when it opens, at each render, when the preference changes and when printing starts and ends, and sends it to its frames, which set it on their documents. So frames stay transparent on a dark page, their text in its colours, and every component, framed or inline, gets it as `context.scheme` (spec §7.4), with an update when it changes. A page that switches its scheme itself sets `color-scheme` on the container, or above it, and calls `view.scheme()` (or renders).

**Where the page renders** is the host's choice, never the document's:

| | `page: "frame"` (the default) | `page: "host"` |
|---|---|---|
| The page | one strict frame per document | the host's page |
| Strict components | in the page's frame, sharing it | a frame per use, its slots inside it |
| Permissive components | their fallbacks, with an error `not-here` | a frame per use, with `host` if the configuration gives it |
| Trusted components | their fallbacks, with an error `not-here` | inline, in the host's page, with `host` when the host gives one |
| Math | typeset as MathML by Temml, in the page's frame | the same, the frames' own Temml loaded into the host's page from `math.js` where the frames are served, the first time a document has math |
| Code blocks | highlighted by Shiki, in the page's frame | the same, the frames' own Shiki loaded into the host's page from `highlight.js` where the frames are served, the first time a document has a code block |
| The document's images | fetched by the host's page and shown from `blob:` URLs; alt text until then, and for one it can't have | loaded as any image of the host's page |
| For | hosts of others' documents with strict packs | an app with components of its own, or permissive packs |

**Each component's level** comes from the processor (`HostComponent.trust`, set by the host's configuration, spec §6.4), and so does a built module's place (`module`), which `modules` turns into a URL and the file's hash: the frame loads it with `<script integrity>`, so a file that doesn't match never runs. A host whose page holds the file, already checked, as a resolver of registry packs does, gives it instead: `modules: (module) => ({ code: blob })`. The frames load it as a `blob:` module, sharing the `Blob`'s bytes, and with no network; the module must be one file ([packs from a registry](../docs/packs-from-a-registry.md), R2). The library packs a pack's components import (`HostComponent.libraries`) are asked of `modules` the same way, and must be given as code: each frame loads each library once, as a `blob:` module, and writes the components' imports of it as its URL, so the packs in a frame share one copy ([sharing library packs](../docs/sharing-library-packs.md)). A built module given by its URL can't import them. Source (JSX, TSX, Svelte, TypeScript) is compiled once per document in a hidden compile frame that runs no components.

**A document's images in frame mode.** The page's frame has no network, so its own Markdown and raw HTML images can't load there. The host's page fetches each image of the trees and splices it sent the frame, a relative `src` against its own address as in host mode, without credentials or a `Referer`, and gives the frame the file, which it shows from a `blob:` URL; the frame's policy stays as it is, so component code in it still loads nothing. Code in the frame could ask for an image too, so the host's page gives only those of the trees it sent. The alt text shows while an image loads, and stays when it can't be had: missing, not an image, or from another origin whose server doesn't allow the host's page to read it (no `Access-Control-Allow-Origin`), which an image element would still have shown. Remote images load without asking the reader, as in host mode.

**Where the page came from.** An editor or a review tool links what the page shows to the text that made it, as `@compmd/render`'s page does ([where the page came from](../render/README.md#where-the-page-came-from)), the same in both modes, from a tree processed with `origins: true`:

- **`view.reveal(offset)`** scrolls the page, as little as it takes, to what the text at `offset` drew, and marks it for 1.5 s with the class `compmd-revealed`, an outline in the theme's accent. An editor calls it when the reader moves the caret.
- **`view.rectsAt(offset)`** resolves with the boxes of what it drew, one per element, in the host's viewport, as `getBoundingClientRect()` gives them, for a host that draws its own marks over the page. In frame mode the frame measures them and the view adds where the frame is.
- **`pointed`** gives the range of the text that made what the reader clicked (`{start, end}`, in code points, `Source`): an editor moves its caret there. A page reports its reader's clicks only to a host that listens: the first `pointed` listener asks for them. The click goes on as usual, so a link still comes as `link`.
- **`view.lines(from?)`** is an async generator of each line of the text that draws something, from line `from`, with the boxes it shows in, `{line, rects}`, each rect `{x, y, width, height}` in CSS pixels from the view's container's top left corner, by `@compmd/render`'s rules ([where each line is](../render/README.md#where-each-line-is)), for a gutter of line numbers. Each line is measured as the loop reaches it, so a gutter that stops at the bottom of what shows measures no more; in frame mode the frame measures a batch at a time and the view adds where the frame is in the container. A use in a frame of its own (host mode) is placed as a whole, on its first line. **`layout`** says when to ask again: after a render or splice is painted, when the page or a frame in it changes size, when fonts load. In frame mode the page scrolls inside its frame, which the host doesn't see: its places are of the frame's page as if it weren't scrolled.

Offsets are of the text of the last render or splice, and the calls come after the renders and splices sent before them. In frame mode the frame's code may be untrusted: what it sends as a source or a box is checked to be numbers, but a frame's code could still report a click that wasn't, so a host only moves a caret with it. A click inside a use's own frame (a strict or permissive use in host mode, a nested use) isn't seen by the page around it, and gives nothing.

**When the page has settled.** A view's page keeps moving after `render()` returns: each frame takes its height once it has drawn and reported it, its own frames later still, and components that draw late (a diagram, data through `host`) later again. Tests, printing and scrolling to a link's target wait for it to stop:

```ts
view.render(processed.expanded);
await view.settled();                          // every use drawn, every frame sized: safe to click, print, scroll
view.on("settled", () => print());             // each time it settles again, after a render or a late drawing
```

- **Settled** is, after the latest render or splice: every use in the host's page and in every frame, at any depth, mounted or showing its fallback; every promise a component gave `busy` settled (spec §7.4); every frame's own frames settled and every frame's last reported size applied; the page's images and stylesheets loaded or failed, and its fonts loaded; then one animation frame in which nothing moved (`@compmd/render`'s `layout`, [where each line is](../render/README.md#where-each-line-is)).
- **`view.settled()`** resolves the next time that holds, at once if it does; a render or splice after it makes it wait for that one. **`settled`** comes each time the view settles again. The container has the attribute `data-compmd-settled` while it is settled, so a browser test waits for it without reaching into the app: `await page.locator("#page[data-compmd-settled]").waitFor()`.
- **What can't hold it forever:** a frame that hangs is stopped (W118) and counts as settled; a `busy` promise, a frame that never loads, and a frame that never says it settled count after 10 s (`BUSY_MS`); a frame that reports settled early, or never reports a size, moves the page later, which `layout` says. A frame's code can only make `settled` come early or late, never change what the page shows.
- **Not for drawing a gutter:** what moves the page comes as `layout`, which a gutter follows; `settled` is for waiting until it stops.

**Frames only go down in trust.** The host's page holds permissive and strict frames; a permissive frame holds strict ones; a strict frame holds none. A permissive or trusted component in a strict container's slot shows its fallback, with `not-here` naming why; write such a container in CompMD, or make it the host's own.

## Code blocks

Every code block that shows as code is highlighted, wherever a page renders (the page's frame, a use's frame, the host's page): one of a language no code renderer draws, and a code renderer's fallback, when its component is missing or fails (spec §7.7). It is `@compmd/render`'s `highlight`, given as `shikiHighlight()`, which a host that renders a page itself with `createPage` or `createRegion` can give too.

- **Shiki,** with its JavaScript regex engine, so a strict frame needs no WebAssembly and its policy stays as it is. Its core and engine load the first time a page has a code block; each language's grammar the first time a block of it shows. Until then, and for a block it can't highlight, the text shows as it is. A block is highlighted again only when its text changes.
- **Light and dark:** GitHub's themes, as the website's spec page. Each token's colour is `light-dark(light, dark)`, set through the CSSOM, so it follows the `color-scheme` every frame takes from the page around it, with nothing drawn again when it changes.
- **CompMD's syntax** shows in `md`, `markdown` and `compmd` blocks: component lines, inline components, param lists and `{{paths}}`, by a grammar injected into Markdown's, with a document's frontmatter, or a `compmd` definition's signature, as YAML. The website's spec page uses the same grammar, `@compmd/sandbox/grammar`.
- **The languages, by the fence names that mean them:** C (`c`), C++ (`cpp`, `c++`), C# (`csharp`, `c#`, `cs`), Java, Kotlin (`kotlin`, `kt`, `kts`), Scala, Swift, Objective-C (`objective-c`, `objc`), Go, Rust (`rust`, `rs`), Zig, Dart; Python (`python`, `py`), Ruby (`ruby`, `rb`), PHP, Perl, Lua, R, Julia (`julia`, `jl`), PowerShell (`powershell`, `ps`, `ps1`, `pwsh`), shell (`shellscript`, `bash`, `sh`, `shell`, `zsh`), Batch (`bat`, `batch`, `cmd`); Haskell (`haskell`, `hs`), Elixir, Erlang (`erlang`, `erl`), Clojure (`clojure`, `clj`), F# (`fsharp`, `f#`, `fs`), OCaml; JavaScript (`javascript`, `js`, `cjs`, `mjs`), TypeScript (`typescript`, `ts`, `cts`, `mts`), JSX, TSX, HTML, CSS, SCSS, Less, Svelte, Vue, Astro, GraphQL (`graphql`, `gql`); JSON, JSONC, JSON5, YAML (`yaml`, `yml`), TOML, INI (`ini`, `properties`), XML, CSV, SQL, Protocol Buffers (`proto`, `protobuf`); Dockerfile (`docker`, `dockerfile`), Makefile (`make`, `makefile`), CMake, HCL, Nix, Nginx; Diff, Markdown (`markdown`, `md`), CompMD (`compmd`), LaTeX, Mermaid (`mermaid`, `mmd`), MATLAB, Groovy, Solidity. A name is any case; any other language stays plain.

**One copy, the frames'.** Shiki and Temml are built once, with the frame runtime (`dist/frames/`): the frames load them, and so does a host-mode page, from `highlight.js` and `math.js` where the host serves the frame documents (`base`), which share their chunks with the runtime. So the host's own bundle has neither, and a page and its frames load the same files, which the browser caches once. A host that renders a page itself with `createPage` and gives it `shikiHighlight()` or `temmlMath()` bundles its own. A host's page with a content security policy allows scripts from where the frames are served. A library the sandbox comes to load next is added the same way: [libraries in the sandbox](../docs/libraries-in-the-sandbox.md) has the steps.

**What it costs.** Measured on the frame documents' build (`dist/frames/chunks/`, gzipped at level 9), of what a host serves: Shiki's core is 34.6 KB, its JavaScript engine 19.6 KB, the two themes 4.9 KB, and the 63 grammars 406 KB, about 465 KB in all. A page loads the core, the engine, the themes and the grammars of its blocks: about 75 KB for one with a JavaScript block. Most grammars are 1 to 15 KB; the largest are C++ (52.9 KB), PHP (27.9 KB), Objective-C (22.7 KB), JSX, TSX, JavaScript and TypeScript (about 16 KB each, each with what it embeds).

## Asking before data from a new origin

A host that fetches remote data while a reader views a document must ask the reader the first time a document needs an origin the reader hasn't decided on, and remember the answer (spec §8.6). `loadData` does it in the host's page, never in a frame: it asks about new origins, then fetches from the allowed ones, without credentials or a `Referer`, and gives what the request takes.

```ts
import { loadData } from "@compmd/sandbox";

const { remote, disallowed } = sources(request);          // the URLs a render needs: of allowed origins, and of others
view.render(process(request).expanded);                  // meanwhile, a new origin is F028 and shows its fallback
const loaded = await loadData([...remote, ...disallowed], { configured: request.allowedOrigins ?? [], document: title });
view.render(process({ ...request, ...loaded }).expanded); // { allowedOrigins, remote }: the origins are checked again
```

A host that fetches on its own uses `allowOrigins(urls, options)`, which asks and gives the allowed origins alone. With no reader to ask (a server, a build), give an `ask` that denies: nothing is fetched then but from the configured origins.

- **Configured origins** are never asked about. Every other origin is asked about once, however many renders need it at once, and the answer, allow or deny, is kept per origin for every document: in `localStorage` by default (`localAnswers()`), or in a store the host gives (`answers`), such as one on its server.
- **The question** is a plain `<dialog>` (`askWithDialog`): *This document wants to load data from api.example.com. Allow?*, with what the site learns, the values the reader picked, and the buttons *Allow* and *Don't allow*. A host asks in its own UI with `ask`, and `originQuestion(origin)` gives the words.
- **Changing an answer:** `localAnswers().list()` gives every answer, and `forget(origin)` makes the next render ask again, for a page where the reader reviews them.

## Observing a view

Developer tools see what a view's sandbox does ([the Developer Tools](../docs/devtools.md), decisions 5 and 6): its frames down to those its frames make, the messages on their ports, the components mounted in each window with how long they took, and their fallbacks.

```ts
const observation = view.observe((events) => tools.update(observation.events));
observation.events;    // the last 2,000 events of the host's page and of each of its frames, in the order they came
observation.dropped;   // what was dropped, by source
observation.stop();    // the sandbox observes while an observation listens
```

Only the host's page turns it on; nothing a document, a pack or a frame sends does. Each frame records its own part once its parent asks, and reports it up: an event with `by` is the report of that frame of the host's page, which code of its level may have changed, so the tools mark it so and nothing in the sandbox acts on it. Ids are paths, `"2/1"` the first frame of the second, and a parent puts its child's id in front of each it reports, so a frame can only report on frames under it. Long values in a message are kept by size (`{ $text: length, start }`, `{ $blob: size }`), and pings aren't copied.

## In an editor

An editor that renders a document's blocks in its own widgets (CodeMirror's) uses the host runtime directly, one use per widget:

```ts
import { HostRuntime } from "@compmd/sandbox";

const runtime = new HostRuntime({ base: "/sandbox/", trusted }, { set, link, error });
await runtime.set(processed.hostComponents);
const use = runtime.mountUse(widget, node, inputs);   // inline if trusted, else in a frame of its level
use.update(nextNode, inputs);
use.destroy();
```

`runtime.mounts` is a `components` map for `@compmd/render`'s regions. A frame whose element an editor removes or moves loads again, so a component's state belongs in its params. The host's page may itself be a sandboxed frame: a frame believes only its parent.

## The protocol

`@compmd/sandbox/protocol` is the frame protocol alone, with no DOM: its messages, and each end's rules over a port (`serveParent` in a frame, `connectChild` in its parent, `connectCompiler`). `protocol.schema.json` describes the messages, for an implementation in another language or runtime.

## What a browser can't stop

[The design's threat model](../docs/running-components-in-the-sandbox.md#threat-model) has the list: WebRTC, which no content security policy covers; styles, fonts and prefetches to the frames' own origin, which can carry a query string there; and a frame that spins, which Chrome stops but WebKit can't, since it runs these frames in the page's process.

## Tests

- **Unit tests** next to the code, in Node: the protocol's ends, its schema, the policies, the images the host's page gives a page frame, the highlighter (every language's grammar loads and draws with the JavaScript engine), compiling, the components a frame loads, a component's imports of library packs, `host` across a port and in the host's page, alike, the observer, the reports a parent takes and the bound on what it keeps.
- **[`test/`](test/)**, in Chrome and WebKit, with the frame documents served from a server of their own and probes that record every request: what a strict frame can't reach (T1 to T5), a permissive frame's origins and what it can't nest (T1, T6), a frame that ignores all but its parent (T7), built modules and their hashes (T8), built modules the host's page holds, library packs (one instance per version in a frame), a frame that hangs (T9, Chrome), one document's frames apart from another's (T4), both page modes, a host's page that is itself in a sandboxed frame, a document's images in both modes, where the page came from in both modes (boxes, `reveal`, clicks only once asked), code blocks highlighted in both modes, light and dark, a code renderer's fallback among them, and the observer: the whole tree in both modes, a forged report (O6), an unasked one (O4, O7), a flood (O2), and the same messages with and without it (O3).
