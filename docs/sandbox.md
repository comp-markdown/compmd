# The sandbox

**Status: accepted (2026-10-30).** How `@compmd/sandbox` runs component code in a browser at the three trust levels of spec §15. Part of [components from apps and packs](components-from-apps-and-packs.md), which decided *that* the sandbox is a package (decision 4) and *what* each level may reach (Q1); this design decides *how*. The messages between its windows are their own design: [the sandbox's frame protocol](sandbox-protocol.md).

## Proposed decisions

1. **A frame runs one trust level, and the frames form a tree that only goes down in trust.** Each frame renders a part of the tree, the page or one use with its slots, and runs code of one level only. A component of another level in it gets a frame of its own where it is used, but only a *lower* level: the host's page holds permissive and strict frames, a permissive frame holds strict ones, and a strict frame holds none. A permissive or trusted use that would be under a strict one shows its fallback, with a warning (Q2).
2. **A frame believes its parent and nothing else.** It takes its level, its code, its params and its slots from the window that made it, which is never less trusted than itself. This holds however deep the host's page itself is: a top-level page, or a page that is itself in a sandboxed frame, as an application whose editor runs in a sandbox of its own.
3. **Two frame documents, whose policies keep the tree.** `strict.html` has a fixed policy, with no network and no nested frames, which no message changes. `permissive.html` starts without its network directives, and its parent adds the pack's origins as a second policy, which can only narrow the first; it may nest `strict.html`, named by its full URL, and nothing else. So strict code can only ever make frames with no network.
4. **The host chooses where the page renders** (`page: "frame" | "host"`). In **frame mode** (the default) the page is one strict frame per document, and strict code shares it; permissive and trusted uses show their fallbacks. In **host mode** the page renders in the host's page: trusted components mount inline, and each use of a strict or permissive component is a frame of its own, with its slots inside it.
5. **`host` goes to the permissive frames the host's page makes**, of packs the configuration gives `host` (`{"level": "permissive", "network": […], "host": true}`). Each frame's parent made it, so there is no frame to fake.
6. **Layers a host can use apart:** the protocol (messages and rules, no DOM), the frame runtime (what runs in a frame), the host runtime (mount one use in a target the host gives, inline or in a frame), and defaults (the two frame documents, the import map, a compile frame, `createSandbox` with the two modes). A host with frame documents, libraries and compiling of its own uses the first three; a simple host uses all four.
7. **A built module loads by its URL** with `<script type="module" integrity>`, so the browser checks the file's hash the host gives, and runs nothing that doesn't match. **Source** is compiled once per document, by default in a compile frame that runs no components.
8. **Every threat in the threat model is a browser test,** in Chrome and WebKit. Where a browser can't keep a rule (WebRTC, below), the design says so, and the test records what the browser does.

## Today

The playground (`editor/`) renders each document in one iframe, `sandbox="allow-scripts"`, so its origin is opaque. Its content security policy allows scripts from the site, `blob:` modules and the import map by its hash, and no requests (`default-src 'none'`). The editor sends each host-language component's source on the frame's port; the frame compiles it (sucrase, Svelte's compiler), loads it as a `blob:` module and mounts it through its binding. Every component runs in this one frame, with no network, whatever its pack. There are no trust levels, no built modules and no `host`.

## Terms

- **The host's page**: the window where `@compmd/sandbox`'s host runtime runs, with the renderer in host mode and trusted components. It is the application's page, or a frame the application sandboxes itself, such as one that holds an editor.
- **A frame**: an iframe with `sandbox="allow-scripts"`, so it has an opaque origin: no cookies, no storage, no access to its parent's DOM or another frame's. Its parent is the window that made it.
- **A use**: one component node in the expanded tree, as `:: chart {…}` is once per line it is written on, and once per element of an `each`.
- **A part**: what a frame renders, the whole page or one use with its slots.

## The examples

The five examples of [components from apps and packs](components-from-apps-and-packs.md#the-examples), in one document:

````md
:: stars {value=4}                        ← the app's own, trusted
:: rating-summary {product=p}             ← the app's own Svelte, permissive with `host`
::: bar-chart {rows=(md.data.sales)}      ← @acme/charts, unlisted: strict
  :: sparkline {rows=(md.data.trend)}     ← @acme/charts, in bar-chart's slot
::: /bar-chart
```mermaid                                ← @kit/mermaid, a built module, strict
graph LR; Draft --> Review
```
:: map {place="Kyoto"}                    ← @acme/maps, permissive, tiles.example.com
````

And two hosts: a **site** that renders others' documents in a top-level page, and an **application** whose editor runs in a sandboxed frame of its own and renders a document's blocks in the editor's widgets (CodeMirror's), with its own components trusted there.

## Threat model

| Code | Where it comes from | It must not reach |
|---|---|---|
| The host's page, the sandbox's host runtime, trusted components | the application, built with it | (trusted) |
| Permissive code | a pack the host lists as permissive | other origins than its pack's; the host's page and session; other frames' content; `host`, unless the configuration gives it |
| Strict code | the document, unlisted packs, packs listed as strict | any network; the host's page and session; other documents' frames; code of a higher level |
| The document | anyone | a higher level for any of its code (spec §15) |

| # | Threat | Stopped by |
|---|---|---|
| T1 | Strict code sends what it sees: `fetch`, images, fonts, styles, sockets, beacons, prefetch | `strict.html`'s fixed policy, with no network |
| T2 | Strict code reads the host's session: cookies, storage, the host's DOM | the opaque origin (`sandbox` without `allow-same-origin`) |
| T3 | Strict code navigates, opens a window or submits a form | `sandbox` without `allow-top-navigation`, `allow-popups` or `allow-forms` |
| T4 | Strict code of one document reads another's | one frame per document in frame mode, one per use in host mode; frames can't reach each other's DOM |
| T5 | Strict code makes a frame with a network, or gives code to one | `strict.html` nests no frames (`frame-src 'none'`); a `srcdoc` frame inherits its policy |
| T6 | Permissive code widens its own network by nesting a frame it narrows itself | `permissive.html` nests only `strict.html` |
| T7 | Code gets `host` that the configuration didn't give it | only the host's page gives `host`, to frames it made, of packs given `host` |
| T8 | A module changed on the server or a cache after the host checked its pack | the file's hash (decision 7) |
| T9 | A component runs forever and freezes the page | each parent pings its frames; a frame that doesn't answer in time is removed and its use shows its fallback (W118, spec §15 rule 5) |
| T10 | A component draws a fake prompt over the host's interface | frames are placed as the document's content, inside the page's box, never over it |

**What a browser can't stop**, and so this design can't either:

- **WebRTC.** A content security policy doesn't cover `RTCPeerConnection` in current browsers, and a STUN request can carry data to any server. The frame removes the WebRTC constructors before any component runs; strict code may still get fresh ones from a nested `srcdoc` frame, if a browser doesn't apply `frame-src 'none'` to it. The browser tests record what each browser does; a host that must close it serves the sandbox's files from a separate site and turns WebRTC off there with a `Permissions-Policy`, where a browser supports one.
- **Requests to the host's own files.** A frame loads scripts, styles and fonts from the origin where the host serves the sandbox and its packs, and a browser lets a prefetch go there too, so such a request can carry data in its query string to the host's own server, without credentials. The browser tests record exactly these and nothing else. Strict code can't reach anyone but the host this way. A host that minds serves those files from a separate origin with no logs.
- **Messages between frames.** Any frame can post a message to any window it can name, and frames can name each other through `top.frames`. A strict frame can only hand data to another frame, never send it out, and a frame acts only on its parent's messages; a permissive pack whose code listens for other windows' messages and sends them on is one the host trusted with its network.

## Q1. Where the page renders

| | F1. Frame mode | F2. Host mode |
|---|---|---|
| The page | one strict frame per document | the host's page |
| Strict uses | in the page's frame, sharing it | a frame per use, its slots inside it |
| Permissive uses | the fallback, with a warning | a frame per use, with `host` if the configuration gives it |
| Trusted uses | the fallback, with a warning | inline, in the host's page |
| Our example | `bar-chart`, `sparkline`, `mermaid` in the page's frame; `stars`, `rating-summary`, `map` fall back | `stars` inline; frames for `bar-chart` (with `sparkline` in it), `mermaid`, `map`, `rating-summary` |
| Frames for 10 charts | 1 | 10 |
| For | the playground, published sites, any host of others' documents with strict packs only | an app with components of its own, or a host with permissive packs |

**Both**, and the host chooses (decided in T-298's conversation). The document can't choose, since it must not decide how its own code is isolated, and the sandbox doesn't switch by itself: one edit adding `:: stars` would move the whole page.

## Q2. Why the tree only goes down, and what that costs

**Why.** A browser frame can tell where a message came from by its window (`event.source`) and its origin. A frame always knows its parent's window, and the frame that made it is its parent. It can't know which ancestor is the host's page: the host's page may be `top` (a site) or a frame several levels down (an application's sandboxed editor), and every frame in between has an opaque origin, as the host's page itself may. So a frame can believe its parent only, and a parent must never be less trusted than its child: otherwise strict code could make a frame and give it the network it chooses.

**Options considered:**

| | A tree that only goes down (recommended) | A star: every frame believes the host's page | Permissive under strict, by a secret |
|---|---|---|---|
| How a frame knows who to believe | its parent | the only window whose origin isn't opaque | a token the host's page puts in the nested frame's URL |
| A host's page in a sandboxed frame | works | fails: its origin is opaque too, so strict code can pose as it | — |
| Strict code makes a frame | it can't: `strict.html` nests none | it can, and the frame waits for "the host's page", which strict code can pose as | — |
| Rejected because | — | it rests on the host's page having a real origin, which an application's sandboxed editor doesn't | the strict frame makes the nested frame, so it sees the URL |

**What we lose,** with the examples:

| A document writes | With a tree that only goes down |
|---|---|
| `:: map` at the top level, in host mode | works: the host's page makes its frame |
| `:: map` in a `box`, `grid`, `each` or a `compmd` definition such as a card written in CompMD | works: these run no code, so the host's page renders them and makes the map's frame |
| `:: map` in the slot of a trusted component (the app's own `::: panel`) | works: the trusted component's slots are rendered in the host's page |
| a strict `:: legend` inside `::: map` | works: a permissive frame may hold a strict one |
| **`:: map` inside `::: tabs` from an unlisted pack** (strict, written in Svelte) | **the map shows its fallback**, with a warning: the tabs' frame is strict and can't hold it |
| **`:: rating-summary` inside an unlisted `::: accordion`** | **its fallback**, with a warning, as above |
| **`:: map` inside a listed permissive `::: dashboard-panel`** (another permissive pack) | **its fallback**: a permissive frame nests only strict ones, so one pack can't give another pack's frame its network |
| **any permissive or trusted use in frame mode** | **its fallback**: the whole page is strict |
| `:: map` in the playground (frame mode) | its fallback, unless the playground uses host mode for documents with permissive packs |

So the cost is a permissive or trusted component in the slot of a host-language container that isn't trusted. Layout (`box`, `grid`) and containers written in CompMD don't count, since they run no code. **What an author or host can do:** write such a container in CompMD, or, when it is the app's own, list its pack as trusted. Listing it as permissive doesn't help, since only the host's page holds permissive frames. The warning names the container, so the author sees why.

## Q3. The frame documents and their policies

Two static documents, which the package builds and the host serves where it likes (`base`); the host's page makes frames of them, never anyone else's:

| | `strict.html` | `permissive.html` |
|---|---|---|
| First policy, in a `<meta>` | `default-src 'none'`; scripts from `base` and the host's pack files, `blob:` modules and the import map by its hash; styles from `base` and inline; `img-src data: blob:`; `font-src` from `base`; `frame-src 'none'`; no objects, forms or `<base>` | the same, without the network directives (`connect-src`, `img-src`, `font-src`, `media-src`, `default-src`), and `frame-src` naming `strict.html` by its full URL |
| Second policy | none: nothing to narrow | from its parent, before any component loads: `default-src 'none'`, the first's script and style sources, and the pack's origins in `connect-src`, `img-src`, `font-src` and `media-src` |
| May nest | nothing | `strict.html` |

A browser enforces every policy a document has, so the second can only narrow the first. Until it arrives, only the sandbox's own script runs in a permissive frame, and that script loads no component until it has added it. `frame-src` needs `strict.html`'s full URL, so the host builds the frame documents for its origin (`buildFrames({ base })`, which `@compmd/vite` runs at build); a host that serves its own frame documents builds its own.

**Options considered:** one `frame.html` narrowed for every level (rejected: strict code could make one and narrow it to a network it chooses); a page per frame written by its parent (`srcdoc`; rejected: a `srcdoc` frame inherits its parent's policy, so a permissive frame's strict child would have its network); a policy per request from the host's server (rejected: the frame's URL would say its origins, and the frame's maker chooses the URL).

The host SHOULD also serve both with the header `Content-Security-Policy: sandbox allow-scripts`, so they are sandboxed even when opened outside a frame.

## Q4. Loading code

**Source (S1)**, from a document or a pack's component files:

| | C1. Each frame compiles | C2. A compile frame per document (recommended default) | C3. The host's page compiles |
|---|---|---|---|
| Compiles each component | once per frame | once | once |
| Loads the compilers (sucrase 43 KB, Svelte's 221 KB, gzipped) | in every frame that needs one | once | once |
| Ten Svelte charts in host mode | ten loads of Svelte's compiler | one | one |
| Risk | none new | a compiler bug that runs code could change other components' compiled text, of this document only | the compilers parse untrusted text in the host's realm |

The compile frame is a hidden `strict.html` frame of the host's page that runs no components: the host's page sends it each component's source and gets back the compiled text, which it gives each frame as its code. A host that compiles and caches code itself (layer 3 without the defaults) gives the sandbox compiled code instead.

**Built modules (S2).** The host gives the sandbox, for each module, its URL and its file's hash:

```ts
modules: (module) => ({ url: `/packs/${module.pack}@${module.version}/${module.path}`, integrity: hashes[…] })
```

| | M1. Script integrity (recommended) | M2. The import map's `integrity` | M3. The host's page fetches |
|---|---|---|---|
| How | the frame adds `<script type="module" src integrity>`, then imports the URL, which is the module the browser checked | the frame's import map lists each module's hash | `fetch(url, {integrity})` in the host's page, and the text sent to the frame as a `blob:` module |
| Needs | nothing more | an import map per frame, so a page per frame (rejected in Q3) | the module as one file: its own imports can't resolve from `blob:` |
| Cached | by URL, compiled code too | by URL | the download only; parsed in every frame |

A module's own imports (its chunks) load from the same pack files, checked only by the host when it got the pack: a pack's module SHOULD be one file. Bare imports (`react`) resolve through the import map, as compiled source's do.

**The import map** holds the libraries every frame shares: by default the runtimes of the spec's bindings (React 19 and React DOM, Svelte 5). `buildFrames` takes the other libraries a host offers (the playground adds d3 for its charts). Libraries from the registry for packs are S3, still open.

## Q5. A part in a frame

A frame renders its part with `@compmd/render`: the page, or one use and its slots. A use's slots are expanded nodes, rendered inside the frame, so a sparkline in a bar chart's slot runs in the bar chart's frame, at its level. A strict component in a permissive use's slot gets a nested `strict.html` frame there; anything higher shows its fallback (Q2).

This needs one thing of `@compmd/render` that it doesn't have: **a component that renders its slots itself.** Today the renderer renders a component's slots and gives it their elements. The sandbox's mount for a framed use takes the slots' nodes instead, and the frame renders them. In `@compmd/render`: a mount marked as taking its slots as nodes, which only the sandbox uses.

A frame's box is the use's outer box (spec §9.2), laid out by the parent as today. The frame fills its width and reports its height, which the parent sets; an inline use's frame reports its width too.

## Q6. Hosts that render in an editor

An application whose editor renders a document's blocks in the editor's widgets (CodeMirror's), in DOM the editor owns, renders block by block with `@compmd/render`'s regions, not a page. It uses the host runtime directly:

```ts
import { mountUse } from "@compmd/sandbox";

// in a widget's toDOM, for each use the region renders
const handle = mountUse(target, node, { component, inputs, onSet, onError });
handle.update(node, inputs);
handle.destroy();
```

`mountUse` mounts a trusted use inline and makes a frame of its level for any other, as host mode does; a region's `components` map can be built from it.

- **Widgets come and go.** An editor unmounts widgets outside its viewport and builds them again, and a frame whose element is removed or moved loads again, so its component's state is lost and the frame boots again. The component's state belongs in its params or the host's store (spec §7.4), and the cost of booting again is the host's to keep down: keep the widget's DOM while it is near the viewport, or move frames with `moveBefore` where a browser has it, which keeps them alive.
- **The host's page may be sandboxed itself.** An editor that runs in a sandbox of its own is the host's page here: its frames believe it as their parent (decision 2), its trusted components are the application's code in that sandbox, and its `host` is what the application gives that sandbox.

## Q7. The API

Four layers (decision 6):

| Layer | What it is | Who uses it |
|---|---|---|
| **Protocol** | the messages and each end's rules, over `@compmd/render`'s `Port`; no DOM | every host, and the other layers; [its own design](sandbox-protocol.md) |
| **Frame runtime** | what runs in a frame: applying its level, loading code, rendering its part, answering pings | the package's frame documents, or a host's own |
| **Host runtime** | `mountUse(target, node, …)`: a use inline or in a frame of its level | host mode, editors |
| **Defaults** | `strict.html`, `permissive.html`, the import map, the compile frame (`buildFrames`), and `createSandbox` | simple hosts, the playground |

```ts
import { createSandbox } from "@compmd/sandbox";

const sandbox = createSandbox({
  base: "/compmd/sandbox/",               // where the host serves the built frame documents
  page: "host",                           // or "frame", the default
  trusted: { stars: react(Stars) },       // trusted components by name; host mode only
  host: appApi,                           // what permissive components given `host` get
  modules: (module) => ({ url, integrity }),
  stylesheets: ["/styles/docs.css"],      // the host's stylesheets, in every frame
  timeout: 5000,                          // ms a frame may not answer before it is stopped
});

const view = sandbox.open(container, { components: processed.hostComponents });
view.render(processed.expanded);          // or view.splice(splices, change, options), as @compmd/render's
view.components(next.hostComponents);     // when the document's components change
view.on("set", ({ input, value }) => rerender({ [input]: value }));
view.on("link", ({ url }) => confirmAndOpen(url));   // only a reader's own click on a link
view.on("error", (error) => report(error));
view.destroy();
```

- **Each component's level** comes from the processor: `HostComponent.trust`, and `module` for a built one. The sandbox never reads the configuration itself, so a level is decided in one place. The processor's `Trust` gains `host` for permissive packs (decision 5).
- **`trusted`** gives the code of components the host implements; a trusted component not in it shows its fallback.
- **The playground** keeps what is its own (the host stylesheets it offers, links to the text, timings) on top of this API.

## Q8. The browser tests

Vitest's browser mode, with the installed Chrome and WebKit through Playwright (decided in T-298's conversation; WebKit, since an application's web view is WebKit on macOS and Linux). The tests serve the built frame documents, as a host would, from a top-level page and from a page that is itself in a sandboxed frame, and run documents with components that try each threat:

| Test | Threat |
|---|---|
| a strict component's `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `sendBeacon`, image, font, stylesheet and `<link rel=prefetch>` to another origin and to the host's origin all fail | T1 |
| a permissive component reaches its origins and no other; a strict component in its slot reaches none | T1, decision 1 |
| `document.cookie`, `localStorage`, `indexedDB` and `parent.document` throw or are empty | T2 |
| `top.location`, `window.open`, a form's submit and a link's click without the reader do nothing | T3 |
| a component in one document can't read another document's frame | T4 |
| strict code can't make a frame of `permissive.html`, `strict.html`, `blob:` or `data:`, and a `srcdoc` frame it makes has no network | T5 |
| a permissive component can't make a `permissive.html` frame | T6 |
| a frame given a second policy, code or `host` by a window that isn't its parent ignores it | T7, decision 2 |
| a module whose file doesn't match its hash doesn't run, and its use shows its fallback | T8 |
| a component that loops forever is stopped within the timeout, and the host's page stays responsive | T9 |
| a permissive use inside a strict container shows its fallback, with the warning | Q2 |
| `RTCPeerConnection` in a strict frame, and in a `srcdoc` frame it makes: recorded, not asserted | WebRTC |

## Q9. Costs

Measured when built, not budgeted here:

- A frame: its document, the sandbox's script and the runtime it needs, from the HTTP cache after the first; then parsing and running them, once per frame.
- Host mode with ten charts: ten frames, each with React or Svelte. Frame mode: one.
- A frame loaded again when an editor builds its widget again.
- A built module: once per URL, then cached; Mermaid's 976 KB load only for a document that draws a diagram.

## What it means for the spec

Nothing changes in the rules: §15 already allows a sandbox per use, trusted code only where the processor's own code runs, and `host` only where the processor gives it, and §6.4 already checks a built module's files against the pack's integrity when the processor gets them. §15's informative note on browsers gains what this design found: frames that only go down in trust, a policy a frame can't widen, and that WebRTC isn't covered by a content security policy.

## Not in this design

- Serving the sandbox's files from a separate site, for process isolation and WebRTC: an option for later, which the two frame documents leave room for.
- A sandbox outside a browser, which spec §15 also asks for: for a host that runs component code where there is no browser, such as a site's build that renders components into static HTML, or a processor that runs components to report W118. An application that renders documents in its own web view (a desktop app) needs only this one. The protocol layer is what such a sandbox would implement.
- Caching compiled code across visits.
- S3, libraries from the registry.
