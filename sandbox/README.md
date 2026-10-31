<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/sandbox

Runs a CompMD document's components in a browser at the trust levels of [spec §15](../compmd-spec-v1.md#15-security): strict code in frames with no network, permissive code in frames that reach only their pack's origins, and the host's own trusted code in its page. A host gives it trees and its configuration; it never writes a frame or a content security policy itself. The design is [docs/sandbox.md](../docs/sandbox.md), its messages [docs/sandbox-protocol.md](../docs/sandbox-protocol.md).

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
import { buildFrames, buildLibraries, RUNTIMES } from "@compmd/sandbox/build";

const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root, out: "public/sandbox/libs", shared: RUNTIMES });
await buildFrames({ out: "public/sandbox", imports });
```

`buildFrames` writes `strict.html` and `permissive.html`, each with its policy and the import map, next to the frame runtime and the runtimes of the spec's bindings (React 19, React DOM, Svelte 5). `buildLibraries` adds libraries the host offers its documents' components; without it they have the runtimes alone. The documents work wherever they are served. A host SHOULD serve them with the header `Content-Security-Policy: sandbox allow-scripts`, so they are sandboxed even when opened outside a frame.

## A document's page

```ts
import { createSandbox } from "@compmd/sandbox";
import { react } from "@compmd/render/react";

const sandbox = createSandbox({
  base: "/sandbox/",                          // where the frame documents are served
  page: "host",                               // or "frame", the default
  trusted: { stars: react(Stars) },           // the host's own components; host mode only
  host: { rating: (product) => ratings.get(product) },   // what permissive packs given `host` can call
  modules: (module) => ({ url: `/packs/${module.pack}@${module.version}/${module.path}`, integrity: hashes[module.path] }),
});

const view = sandbox.open(container, { components: processed.hostComponents });
view.render(processed.expanded);              // or view.splice(splices, change, { replace, inputs })
view.on("set", ({ input, value }) => rerender({ [input]: value }));
view.on("link", ({ url }) => confirmAndOpen(url));   // a reader's click; show the URL before opening it (spec §15)
view.on("error", ({ component, reason, message }) => report(component, reason, message));
view.components(next.hostComponents);         // when the document's components change
view.styles(["/sandbox/docs.css"]);           // the host's stylesheets, served from the frames' origin
view.theme(tokens);
view.destroy();
```

**Where the page renders** is the host's choice, never the document's:

| | `page: "frame"` (the default) | `page: "host"` |
|---|---|---|
| The page | one strict frame per document | the host's page |
| Strict components | in the page's frame, sharing it | a frame per use, its slots inside it |
| Permissive components | their fallbacks, with an error `not-here` | a frame per use, with `host` if the configuration gives it |
| Trusted components | their fallbacks, with an error `not-here` | inline, in the host's page |
| For | hosts of others' documents with strict packs | an app with components of its own, or permissive packs |

**Each component's level** comes from the processor (`HostComponent.trust`, set by the host's configuration, spec §6.4), and so does a built module's place (`module`), which `modules` turns into a URL and the file's hash: the frame loads it with `<script integrity>`, so a file that doesn't match never runs. Source (JSX, TSX, Svelte, TypeScript) is compiled once per document in a hidden compile frame that runs no components.

**Frames only go down in trust.** The host's page holds permissive and strict frames; a permissive frame holds strict ones; a strict frame holds none. A permissive or trusted component in a strict container's slot shows its fallback, with `not-here` naming why; write such a container in CompMD, or make it the host's own.

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

[docs/sandbox.md](../docs/sandbox.md#threat-model) has the list: WebRTC, which no content security policy covers; styles, fonts and prefetches to the frames' own origin, which can carry a query string there; and a frame that spins, which Chrome stops but WebKit can't, since it runs these frames in the page's process.

## Tests

- **Unit tests** next to the code, in Node: the protocol's ends, its schema, the policies, compiling, the components a frame loads, `host` across a port.
- **[`test/`](test/)**, in Chrome and WebKit, with the frame documents served from a server of their own and probes that record every request: what a strict frame can't reach (T1 to T5), a permissive frame's origins and what it can't nest (T1, T6), a frame that ignores all but its parent (T7), built modules and their hashes (T8), a frame that hangs (T9, Chrome), one document's frames apart from another's (T4), both page modes, and a host's page that is itself in a sandboxed frame.
