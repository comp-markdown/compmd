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
view.on("error", ({ component, reason, code, message }) => report(component, code ?? reason, message));   // code: W118 (threw or failed), W121
view.components(next.hostComponents);         // when the document's components change
view.styles(["/sandbox/docs.css"]);           // the host's stylesheets, served from the frames' origin
view.theme(tokens);
view.scheme();                                // after the host switches its own theme, with a class
view.emit("ratings-changed", { product: "p-1" });   // an event to every component given `host`
view.destroy();
```

**`host`** (spec §7.4) is the same for a trusted component in the host's page and a permissive one in its frame, so one component works at either level: `host.call(name, ...args)` calls the host's function of that name and resolves with what it returned (the arguments and the result must be copyable, since a frame's cross a port), and `host.on(event, listener)` hears what `view.emit(event, data)` sends, until the function it returns is called. A name the host doesn't give fails the call. Strict components never get `host`, nor hear its events. `HostClient` is its type, for a component written in TypeScript.

**Light and dark.** Every frame takes the colour scheme of the page around it: the view reads its container's `color-scheme` (and the reader's preference, where it allows both) when it opens, at each render, when the preference changes and when printing starts and ends, and sends it to its frames, which set it on their documents. So frames stay transparent on a dark page, their text in its colours, and every component, framed or inline, gets it as `context.scheme` (spec §7.4), with an update when it changes. A page that switches its scheme itself sets `color-scheme` on the container, or above it, and calls `view.scheme()` (or renders).

**Where the page renders** is the host's choice, never the document's:

| | `page: "frame"` (the default) | `page: "host"` |
|---|---|---|
| The page | one strict frame per document | the host's page |
| Strict components | in the page's frame, sharing it | a frame per use, its slots inside it |
| Permissive components | their fallbacks, with an error `not-here` | a frame per use, with `host` if the configuration gives it |
| Trusted components | their fallbacks, with an error `not-here` | inline, in the host's page, with `host` when the host gives one |
| Math | typeset as MathML by Temml, in the page's frame | the same, Temml loaded into the host's page the first time a document has math |
| For | hosts of others' documents with strict packs | an app with components of its own, or permissive packs |

**Each component's level** comes from the processor (`HostComponent.trust`, set by the host's configuration, spec §6.4), and so does a built module's place (`module`), which `modules` turns into a URL and the file's hash: the frame loads it with `<script integrity>`, so a file that doesn't match never runs. A host whose page holds the file, already checked, as a resolver of registry packs does, gives it instead: `modules: (module) => ({ code: blob })`. The frames load it as a `blob:` module, sharing the `Blob`'s bytes, and with no network; the module must be one file ([packs from a registry](../docs/packs-from-a-registry.md), R2). The library packs a pack's components import (`HostComponent.libraries`) are asked of `modules` the same way, and must be given as code: each frame loads each library once, as a `blob:` module, and writes the components' imports of it as its URL, so the packs in a frame share one copy ([sharing library packs](../docs/sharing-library-packs.md)). A built module given by its URL can't import them. Source (JSX, TSX, Svelte, TypeScript) is compiled once per document in a hidden compile frame that runs no components.

**Frames only go down in trust.** The host's page holds permissive and strict frames; a permissive frame holds strict ones; a strict frame holds none. A permissive or trusted component in a strict container's slot shows its fallback, with `not-here` naming why; write such a container in CompMD, or make it the host's own.

## Asking before data from a new origin

A host that fetches remote data while a reader views a document must ask the reader the first time a document needs an origin the reader hasn't decided on, and remember the answer (spec §8.6). `allowOrigins` does it in the host's page, never in a frame:

```ts
import { allowOrigins, localAnswers } from "@compmd/sandbox";

const { remote } = sources(request);                     // the URLs a render needs
view.render(process(request).expanded);                  // meanwhile, a new origin is F028 and shows its fallback
const allowedOrigins = await allowOrigins(remote, { configured: config.allowedOrigins, document: title });
view.render(process({ ...request, allowedOrigins, remote: await fetchFrom(remote, allowedOrigins) }).expanded);
```

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

- **Unit tests** next to the code, in Node: the protocol's ends, its schema, the policies, compiling, the components a frame loads, a component's imports of library packs, `host` across a port and in the host's page, alike, the observer, the reports a parent takes and the bound on what it keeps.
- **[`test/`](test/)**, in Chrome and WebKit, with the frame documents served from a server of their own and probes that record every request: what a strict frame can't reach (T1 to T5), a permissive frame's origins and what it can't nest (T1, T6), a frame that ignores all but its parent (T7), built modules and their hashes (T8), built modules the host's page holds, library packs (one instance per version in a frame), a frame that hangs (T9, Chrome), one document's frames apart from another's (T4), both page modes, a host's page that is itself in a sandboxed frame, and the observer: the whole tree in both modes, a forged report (O6), an unasked one (O4, O7), a flood (O2), and the same messages with and without it (O3).
