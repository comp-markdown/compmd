# The sandbox

**Status: proposed (2026-10-30).** How `@compmd/sandbox` runs component code in a browser at the three trust levels of spec §15. Part of [components from apps and packs](components-from-apps-and-packs.md), which decided *that* the sandbox is a package (decision 4) and *what* each level may reach (Q1); this design decides *how*.

## Proposed decisions

1. **A frame runs one trust level.** Each frame renders a part of the tree, the page or one component with its slots, and runs only code of one level. A component of another level inside it gets a frame of its own, nested where it is used. A frame is as trusted as the least trusted code in it, so whatever a frame says is believed only as far as that.
2. **The host chooses where the page renders** (`page: "frame" | "host"`). In **frame mode** (the default) the page is one strict frame per document: strict code shares it, permissive uses are nested frames, and a trusted component shows its fallback. In **host mode** the page renders in the host's page: trusted components mount inline, and each use of a strict or permissive component is a frame of its own, with its slots rendered inside it.
3. **Every frame gets everything from the host's page: its level, its code, its params and slots; its parent only places it.** The host's page is the only window whose origin is not opaque, so a frame can tell its messages apart from anything strict code sends. So strict code can't give a frame code, raise its level, widen its network or choose what a permissive component sends. The messages are their own design: [the sandbox's frame protocol](sandbox-protocol.md).
4. **`host` goes only to frames the host's page made itself,** whose windows it holds. A permissive use nested in a strict frame (any permissive use in frame mode) gets its network, but not `host`: strict code can make a frame that claims to be it, and the host's page can't tell the two apart. The configuration says which permissive packs get `host`; their uses show fallbacks there, with a warning.
5. **One frame document, `frame.html`, served by the host** with the package's other files. Its content security policy is fixed and allows no network until the host's page sends the frame its level: the host's page then adds the policy of that level, which can only narrow what the first allows.
6. **Source is compiled once per document, in a compile frame** that runs no component code, and every frame loads the compiled text. **A built module loads by its URL** with `<script type="module" integrity>`, so the browser checks the file's hash the host gives, and runs nothing that doesn't match.
7. **Every threat in the threat model is a browser test.** Where a browser can't keep a rule (WebRTC, below), the design says so, and the test records what the browser does.

## Today

The playground (`editor/`) renders each document in one iframe, `sandbox="allow-scripts"`, so its origin is opaque. Its content security policy allows scripts from the site, `blob:` modules and the import map by its hash, and no requests (`default-src 'none'`). The editor sends each host-language component's source on the frame's port; the frame compiles it (sucrase, Svelte's compiler), loads it as a `blob:` module and mounts it through its binding. Every component runs in this one frame, with no network, whatever its pack. There are no trust levels, no built modules and no `host`.

## Terms

- **The host's page**: the application's own page, where `@compmd/sandbox`'s host side runs. Its origin is the application's.
- **A frame**: an iframe with `sandbox="allow-scripts"`, so it has an opaque origin: no cookies, no storage, no access to the host's page or to another frame's DOM. Its parent is the window it is nested in.
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

## Threat model

| Code | Where it comes from | It must not reach |
|---|---|---|
| The host's page, the sandbox's host side, trusted components | the application, built with it | (trusted) |
| Permissive code | a pack the host lists as permissive | other origins than its pack's; the host's page and session; other frames' content; `host`, unless the host gives it |
| Strict code | the document, unlisted packs, packs listed as strict | any network; the host's page and session; other documents' frames; code of a higher level |
| The document | anyone | a higher level for any of its code (spec §15) |

What each threat needs, and what stops it:

| # | Threat | Stopped by |
|---|---|---|
| T1 | Strict code sends what it sees: `fetch`, images, fonts, styles, sockets, beacons, prefetch | the frame's policy, with no network (decision 5) |
| T2 | Strict code reads the host's session: cookies, storage, the host's DOM | the opaque origin (`sandbox` without `allow-same-origin`) |
| T3 | Strict code navigates, opens a window or submits a form | `sandbox` without `allow-top-navigation`, `allow-popups` or `allow-forms` |
| T4 | Strict code of one document reads another's | one frame per document in frame mode, one per use in host mode; frames can't reach each other's DOM |
| T5 | Strict code gives a permissive frame code, or a wider network | decision 3: a frame takes code and its policy only from the host's page |
| T6 | Strict code gets `host` by posing as a permissive frame | decision 4 |
| T7 | Strict code loads a frame document that isn't the sandbox's, or one without the sandbox's policy | the frame policy's `frame-src`, which allows `frame.html` only; nested frames keep the `sandbox` flags of their parents |
| T8 | A module changed on the server or a cache after the host checked its pack | the file's hash (decision 6) |
| T9 | A component runs forever and freezes the page | the host's page pings each frame it made; a frame that doesn't answer in time is removed and its uses show fallbacks (W118, spec §15 rule 5) |
| T10 | A component draws a fake prompt over the host's interface | frames are placed as the document's content, inside the page's box, never over it |

**What a browser can't stop**, and so this design can't either:

- **WebRTC.** A content security policy doesn't cover `RTCPeerConnection` in current browsers, and a STUN request can carry data to any server. The frame removes the WebRTC constructors before any component runs; strict code can still get fresh ones from a nested `srcdoc` frame, which no policy blocks. The browser tests record whether this works; a host that must close it serves the sandbox's files from a separate site and turns WebRTC off there with a `Permissions-Policy`, where a browser supports one.
- **Requests to the host's own files.** A frame may load scripts from the paths where the host serves the sandbox and its packs, and a request to them can carry data in its query string to the host's own server, without credentials. Strict code can't reach anyone but the host this way. A host that minds serves those files from a separate origin with no logs.
- **Messages between frames.** Any frame can post a message to any other window it can name, and frames can name each other through `top.frames`. A strict frame can only hand data to another frame, though, never send it out; a permissive pack whose code listens for such messages and sends them on is one the host trusted with its network.

## Q1. Where the page renders

| | F1. Frame mode | F2. Host mode |
|---|---|---|
| The page | one strict frame per document | the host's page |
| Strict uses | in the page's frame, sharing it | a frame per use, its slots inside it |
| Permissive uses | a nested frame per use, network but no `host` | a frame per use, with `host` |
| Trusted uses | the fallback, with a warning | inline, in the host's page |
| Our example | `bar-chart`, `sparkline`, `mermaid` in the page's frame; `map`, `rating-summary` nested (no `host`); `stars` falls back | `stars` inline; frames for `bar-chart` (with `sparkline` in it), `mermaid`, `map`, `rating-summary` |
| Frames for 10 charts | 1 | 10 |
| For | the playground, published sites, any host of others' documents | an app with components of its own |

**Both**, and the host chooses (decided in T-298's conversation). The document can't choose, since it must not decide how its own code is isolated, and the sandbox doesn't switch by itself: one edit adding `:: stars` would move the whole page.

## Q2. Who tells a frame what

A frame talks only to the host's page, and the host's page is the only window it believes; a nested frame tells its parent nothing but its size. Every message, who may send it, and what each end checks: [the sandbox's frame protocol](sandbox-protocol.md).

**Why this holds** (T5, T6): strict code can make a nested frame, but only of `frame.html` (T7), which waits for the host's page. A frame that claims a use gets that use's level, code and params, which its document gives it anyway, and strict code can't give it params of its own. It can't get `host`: the host's page gives `host` only on a port to a window it holds itself (`iframe.contentWindow === event.source`).

**Options considered for `host` in frame mode:**

- **H1. Network only (recommended):** a permissive use in frame mode gets its origins, and no `host`.
- **H2. Neither:** permissive in frame mode is strict. Simpler, but `map` in the playground couldn't load its tiles.
- **H3. A secret in the frame's URL:** the host's page puts a token in the nested frame's URL. The strict frame makes that frame, so it sees the URL: rejected.

**H1, challenged with the examples, in frame mode:**

| Use | What happens | Holds? |
|---|---|---|
| `stars` (trusted) | its fallback, with a warning | yes: the app's page isn't there |
| `rating-summary` (permissive, needs `host`) | mounts with `host` absent, and shows nothing useful | **no**: the sandbox can't tell that it needs `host`, so neither the reader nor the host is told why |
| `bar-chart`, `sparkline` (strict) | in the page's frame | yes |
| `mermaid` (strict, a built module) | in the page's frame, its file checked | yes |
| `map` (permissive) | a nested frame, its tiles from `tiles.example.com`, its params from the host's page | yes |
| `map`, while a malicious `bar-chart` reads the sales data it is given | with params from its parent, `bar-chart` could make a second `map` frame and give it the sales data as its place, which `map` sends to `tiles.example.com` | **only with params from the host's page**, which is why decision 3 sends them so: then strict code can only make a frame that gets the real `map`'s params |
| ten `map`s in an `each` | ten nested frames | yes, at ten frames' cost |

So H1 needs one more thing: **the configuration says which permissive packs get `host`** (`{"level": "permissive", "network": […], "host": true}`, on the processor's `Trust`). A use of such a pack in frame mode shows its fallback, with a warning that it needs host mode, instead of mounting without it. A permissive pack without `host` (`map`) works in both modes.

## Q3. The frame document and its policies

**D1. One static `frame.html` (recommended).** The package ships it with its script and the shared libraries; the host serves them where it likes (`base`). Its first policy, in a `<meta>`, allows scripts from `base` and from the host's pack files, `blob:` modules and the import map by its hash; nested frames of `frame.html` only; no objects, forms or `<base>`. It says nothing yet about the network directives. The frame's own script then waits for the host's page, which sends the frame's level; the script adds a second `<meta>` with the level's policy before it loads any component:

| Level | Second policy |
|---|---|
| strict | `default-src 'none'`, plus the first policy's script and style sources, `img-src data: blob:`, `font-src` from `base` |
| permissive | the same, with the pack's origins added to `connect-src`, `img-src`, `font-src` and `media-src` |

A browser enforces every policy a document has, so the second one can only narrow the first. Until it arrives, only the sandbox's own script runs.

**D2. A page per frame, written by the host's page (`srcdoc`).** The host's page writes each frame's document with its exact policy and an import map holding the modules' hashes. Rejected: a `srcdoc` frame inherits the host's page's own policy, so every host's policy would have to allow what the frame needs (`blob:`, the frame's scripts). And a nested frame in frame mode is made by the strict frame, so its `srcdoc` would be written by strict code.

**D3. A policy per request, from the host's server.** Rejected: the frame's URL would say its origins, and strict code chooses the URL of a frame it makes.

The host SHOULD also serve `frame.html` with the header `Content-Security-Policy: sandbox allow-scripts`, so it is sandboxed even when opened outside a frame; `@compmd/vite` will.

## Q4. Loading code

**Source (S1)**, from a document or a pack's component files:

| | C1. Each frame compiles | C2. A compile frame per document (recommended) | C3. The host's page compiles |
|---|---|---|---|
| Compiles each component | once per frame | once | once |
| Loads the compilers (sucrase 43 KB, Svelte's 221 KB, gzipped) | in every frame that needs one | once | once |
| Ten Svelte charts in host mode | ten loads of Svelte's compiler | one | one |
| Risk | none new | a compiler bug that runs code could change other components' compiled text, of this document only | the compilers parse untrusted text in the host's realm |

The compile frame is a hidden strict frame that runs no components: the host's page sends it each component's source and gets back the compiled text, which it then sends to the frames as their code. In frame mode the page's frame is strict, so it can't be the compile frame: the compile frame is separate, so that strict code never handles what permissive frames run.

**Built modules (S2).** The host gives the sandbox, for each module, its URL and its file's hash:

```ts
modules: (module) => ({ url: `/packs/${module.pack}@${module.version}/${module.path}`, integrity: hashes[…] })
```

| | M1. Script integrity (recommended) | M2. The import map's `integrity` | M3. The host's page fetches |
|---|---|---|---|
| How | the frame adds `<script type="module" src integrity>`, then imports the URL, which is the module the browser checked | the frame's import map lists each module's hash | `fetch(url, {integrity})` in the host's page, and the text sent to the frame as a `blob:` module |
| Needs | nothing more | an import map per frame, so D2 | the module as one file: its own imports can't resolve from `blob:` |
| Cached | by URL, compiled code too | by URL | the download only; parsed in every frame |

A module's own imports (its chunks) load from the same pack files, checked only by the host when it got the pack: a pack's module SHOULD be one file. Bare imports (`react`) resolve through the import map, as compiled source's do.

**The import map** holds the libraries every frame shares: the runtimes of the spec's bindings (React 19 and React DOM, Svelte 5). Which other libraries a frame offers is the host's: the package exports a function that builds the frame's files with the libraries the host names (the playground adds d3 for its charts), and ships a build with the runtimes alone. Libraries from the registry for packs are S3, still open.

## Q5. A part in a frame

A frame renders its part with `@compmd/render`, as the playground's frame renders the page today: the page, or one use and its slots. A use's slots are expanded nodes, rendered inside the frame, so a sparkline in a bar chart's slot runs in the bar chart's frame, at its level. A component of another level in a slot gets a nested frame there (decision 1); a trusted one shows its fallback, with a warning, since only the host's page runs trusted code.

This needs one thing of `@compmd/render` that it doesn't have: **a component that renders its slots itself.** Today the renderer renders a component's slots and gives it their elements. The sandbox's mount for a framed use gets the slots' nodes instead, and the frame renders them. In `@compmd/render`: a mount marked as taking its slots as nodes, used only by the sandbox.

A frame's box is the use's outer box (spec §9.2), laid out by the parent as today. The frame fills its width and reports its height, which the parent sets; an inline use's frame reports its width too.

## Q6. The API

```ts
import { createSandbox } from "@compmd/sandbox";

const sandbox = createSandbox({
  base: "/compmd/sandbox/",               // where the host serves the package's files
  page: "host",                           // or "frame", the default
  trusted: { stars: react(Stars) },       // trusted components by name; host mode only
  host: appApi,                           // what permissive components get, in host mode
  modules: (module) => ({ url, integrity }),
  stylesheets: ["/styles/docs.css"],      // the host's stylesheets, in every frame
  timeout: 5000,                          // ms a frame may not answer before it is stopped
});

const view = sandbox.open(container, { components: processed.hostComponents });
view.render(processed.expanded);          // or view.splice(splices, change, options), as @compmd/render's
view.components(next.hostComponents);     // when the document's components change
view.on("set", ({ input, value }) => rerender({ [input]: value }));
view.on("link", ({ url }) => open(url));  // only a reader's own click on a link
view.on("error", (error) => report(error));
view.destroy();
```

- **Each component's level** comes from the processor: `HostComponent.trust`, and `module` for a built one (T-297). The sandbox never reads the configuration itself, so a level is decided in one place.
- **`trusted`** gives the code of components the host implements; a trusted component not in it shows its fallback.
- **The playground** keeps what is its own (the host stylesheets it offers, links to the text, timings) on top of this API.

## Q7. The browser tests

Vitest's browser mode, with the installed Chrome through Playwright (decided in T-298's conversation), and WebKit through Playwright too: an application's web view is WebKit on macOS and Linux (WKWebView, WebKitGTK), and a browser's rules for frames and policies are what the sandbox rests on. The tests serve the package's built files, as a host would, and run documents with components that try each threat:

| Test | Threat |
|---|---|
| a strict component's `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `sendBeacon`, image, font, stylesheet and `<link rel=prefetch>` to another origin and to the host's origin all fail | T1 |
| a permissive component reaches its origins and no other; a strict component nested in its slot reaches none | T1, decision 1 |
| `document.cookie`, `localStorage`, `indexedDB` and `parent.document` throw or are empty | T2 |
| `top.location`, `window.open`, a form's submit and a link's click without the reader do nothing | T3 |
| a component in one document can't read another document's frame | T4 |
| a fake nested frame made by strict code gets no `host`, and a frame given code on its parent port doesn't run it | T5, T6 |
| a nested `srcdoc`, `blob:` or `data:` frame from strict code has no network | T7 |
| a module whose file doesn't match its hash doesn't run, and its uses show fallbacks | T8 |
| a component that loops forever is stopped within the timeout, and the host's page stays responsive | T9 |
| `RTCPeerConnection` in a strict frame: recorded, not asserted | WebRTC |

## Q8. Costs

Measured when built, not budgeted here:

- A frame: its document, the sandbox's script and the runtime it needs, from the HTTP cache after the first; then parsing and running them, once per frame.
- Host mode with ten charts: ten frames, each with React or Svelte. Frame mode: one.
- A built module: once per URL, then cached; Mermaid's 976 KB load only for a document that draws a diagram.

## What it means for the spec

Nothing changes in the rules: §15 already allows a sandbox per use, trusted code only where the processor's own code runs, and `host` only where the processor gives it, and §6.4 already checks a built module's files against the pack's integrity when the processor gets them. §15's informative note on browsers gains what this design found: the frame policy narrowed by the host's page, and that WebRTC isn't covered by a content security policy.

## Not in this design

- Serving the sandbox's files from a separate site, for process isolation and WebRTC: an option for later, which this design leaves room for: the frame would then learn the host's origin when it is built, instead of taking its own URL's.
- A sandbox outside a browser, which spec §15 also asks for: for a host that runs component code where there is no browser, such as a site's build that renders components into static HTML, or a processor that runs components to report W118. An application that renders documents in its own web view (a desktop app) needs only this one.
- Caching compiled code across visits.
- S3, libraries from the registry.
