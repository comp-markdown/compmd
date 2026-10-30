# The sandbox's frame protocol

**Status: proposed (2026-10-30).** The messages between the host's page and the frames of [the sandbox](sandbox.md): who may send what to whom, on which port, and what each end checks. It builds on `@compmd/render`'s frame protocol ([render README, "In a frame"](../render/README.md#in-a-frame)) and replaces the playground's own messages (`editor/src/protocol.ts`).

## Proposed decisions

1. **The protocol is a star.** Every frame talks to the host's page, and only to it: its level, its policy, its code, its part of the tree, `host`, pings, and everything it sends back (`set`, links, errors). A nested frame tells its parent one thing, its size.
2. **The host's page is believed, and nothing else is.** It is the only window whose origin isn't opaque, so a frame knows its messages by their origin. Whatever a frame sends, the host's page treats as coming from the least trusted code in that frame.
3. **A frame is named by its use.** The host's page gives every use it frames an id, written into the tree it sends; a frame for a use carries that id in its URL (`frame.html#use=u3`), so the host's page knows which use a frame asks for. Ids follow edits, as the renderer's keys do, so an edit updates a frame instead of making it again.
4. **A frame made by strict code gets no more than its use already gives.** A frame that claims a use gets that use's level, code and part, which its document gives it anyway; the policy of that level, which a `srcdoc` frame can't widen; and never `host`, which goes only to windows the host's page made itself.
5. **`@compmd/render`'s protocol stays as it is,** for the page's tree: `render`, `splices`, `theme`, `ready`, `set`, `error`, `rendered`. The sandbox's messages travel on the same port, with their own version, as the playground's do today.

## The windows

| Window | Made by | Runs | Its origin |
|---|---|---|---|
| **H**, the host's page | the application | the sandbox's host side, the renderer in host mode, trusted components | the application's |
| **P**, the page frame (frame mode) | H, one per document | the renderer, the document's strict components | opaque |
| **U**, a use frame | H (host mode, or a use the page renders in host mode); or the frame around it, for a use nested in a frame | the renderer for one use and its slots, that use's component and those of its level in its slots | opaque |
| **C**, the compile frame | H, one per document | the compilers, no components | opaque |

In our [example](sandbox.md#the-examples), in frame mode: H makes P and C; P makes U frames for `map` and `rating-summary`. In host mode: H renders the page and makes U frames for `bar-chart` (with `sparkline` inside it), `mermaid`, `map` and `rating-summary`, and C.

## The ports

Each frame has one **host port**, a `MessageChannel` port shared with H.

- **A frame H made:** when it loads, H posts `hello` to its window with the port. The frame takes the first window message whose origin is H's (its own URL's origin; `null` is never H's) and listens on that port only.
- **A nested frame:** it makes a channel itself and posts `hello {use, version}` to `top`, which is H, with one end. H answers on that port, or not at all.

A frame accepts no other window messages, except a nested frame's `size` from its own child.

## Messages

**H to a frame**, on its host port:

| Message | Says | To |
|---|---|---|
| `hello {version}` (a window message, with the port) | this is your host port | frames H made |
| `level {level, policy}` | your level, `strict` or `permissive`, and its policy, which the frame adds as a second `<meta>` before anything else | every frame, first |
| `code {components}` | each component's code: compiled text with its binding (`react`, `svelte`, `plain`) and CSS, or a module's `{url, integrity}` | P, U |
| `render`, `splices` | `@compmd/render`'s, with use ids in the tree | P |
| `part {node, inputs}` | the use to render, with its slots as expanded nodes, and the document's inputs | U |
| `theme {tokens}` | `@compmd/render`'s | P, U |
| `styles {urls}` | the host's stylesheets | P, U |
| `host {port}` | a port for `host`, as calls and events | U frames H made, of permissive packs given `host` |
| `ping {n}` | answer, or be stopped | every frame |
| `compile {name, language, runtime, code}` | compile this | C |

**A frame to H**, on its host port:

| Message | Says | From |
|---|---|---|
| `ready {version}` | `@compmd/render`'s, with the sandbox's version beside it | P, U, C |
| `loaded` | the code last sent is loaded, or shows fallbacks | P, U |
| `set {use, param, value}` | a component set an input; H checks that the param is bound to an input in that use's node | P, U |
| `error {reason, component, message}` | `@compmd/render`'s, and `failed` for code that didn't compile or load | P, U |
| `link {url}` | a link was clicked; H decides what opens, and shows the reader the URL first (spec §15 rule 4) | P, U |
| `rendered {ms}` | `@compmd/render`'s | P, U |
| `size {width, height}` | the content's size, for H to set on the frame | U frames H made |
| `pong {n}` | the answer to `ping` | every frame |
| `compiled {name, code, css?}`, `failed {name, message}` | a compile's result | C |

**A nested frame to its parent**, as a window message: `size {width, height}`. The parent sets the frame's size if the message came from that frame's window.

## Sequences

**Opening a document, frame mode.**

1. H makes C and P, each with `src` `frame.html`. Each sends `ready`; H sends `level {strict, …}`.
2. H sends C each component's source; C answers `compiled` or `failed`.
3. H sends P `code` and `render`, with ids on the uses P can't run (`map` is `u5`).
4. P renders the page, mounts `bar-chart`, `sparkline` and `mermaid`, and puts `<iframe src="frame.html#use=u5">` in `map`'s box.
5. The `map` frame posts `hello {use: "u5"}` to `top`. H finds the document from the frame's `parent` chain (P's window), and `u5` in it, and sends `level {permissive, …tiles.example.com}`, `code` and `part`; no `host`, since H didn't make this frame.
6. The `map` frame reports its `size` to P; it sends `set` and `link` to H.

**Opening a document, host mode.** H renders the page. For `bar-chart` it makes a frame in the chart's box, and sends `hello`, `level {strict}`, `code` (the bar chart's and the sparkline's) and `part` (the chart's node, the sparkline in its slot). For `rating-summary` it does the same at `permissive`, plus `host`. `stars` mounts in H's page.

**An edit.** H sends P `splices` (frame mode) or renders them (host mode), and sends `part` to every frame whose use changed, by its id. A use that is gone loses its frame; a new one gets one. When the document's components change, H compiles them in C and sends `code` to the frames that have them.

**A frame that hangs.** H pings each frame every second; one that hasn't answered within `timeout` is stopped: a frame H made is removed and its use shows its fallback (W118). For a nested frame, H tells the frame around it to do so, which strict code there could ignore: it can only keep its own document's frame frozen.

**Strict code makes a frame.**

| It makes | H gives it | So |
|---|---|---|
| `frame.html#use=u5`, a second `map` | `map`'s level, code and part, as the real one's | the same params reach the same origin: nothing the document doesn't already send |
| `frame.html#use=u5` and sends it params of its own, as a window message | nothing: the frame takes params only from H | strict code can't make `map` send what it chose (T5) |
| a `srcdoc` frame claiming `u5` | the same as the real one | it inherits P's policy, which `level` can't widen: no network |
| a frame claiming a use its document doesn't have | nothing | — |
| any frame asking for `host` | never | `host` goes only to windows H made (T6) |

## Checks, by end

- **A frame:** its host port comes from a window message of H's origin, or is its own channel to `top`; it adds the `level` policy before it loads any code; it runs code and renders parts only from its host port; it answers `ping`.
- **H:** it gives a nested frame only a use of the document the frame is nested in; `host` only on a port to a window it made; it checks `set` against the use's node; it never acts on `link` without the reader.
- **A parent frame:** it sets a nested frame's size only from that frame's window.

## Versions

The sandbox's version travels in `hello` and `ready`, beside `@compmd/render`'s. H and its frames come from one package, but a host may serve `frame.html` from an older deploy: a frame with another version gets nothing, and H reports it.

## From the playground's messages

| Playground (`editor/src/protocol.ts`) | Sandbox |
|---|---|
| `FRAME_HELLO` with the port | `hello {version}` |
| `components {components}`, compiled in the frame | `code {components}`, compiled in C |
| `components-loaded` | `loaded` |
| `component-failed` | `error {reason: "failed"}` |
| `host-style {style}` | `styles {urls}`; the playground's stylesheet picker becomes a list of URLs |
| `source-link {fragment}`, `open-link {url}` | `link {url}`; the playground handles `#:~:text=` links itself |

## Open

- **`@compmd/render`'s protocol** could take the sandbox's messages into its own, as T-298's description first said, or stay as it is with the sandbox's beside it on the same port (recommended, decision 5): a host that builds its own sandbox can still use `connectFrame` and `serveFrame` alone.
