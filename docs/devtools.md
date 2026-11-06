# The Developer Tools

**Status: accepted (2026-11-02).** The playground (`editor/`) already holds a document's developer tools: its text with the hint at the caret and the diagnostics, the rendered page, and views of what the processor makes of it (plain Markdown, the frontmatter's data, the components, the trees, the inputs). They live in one Svelte app that only the playground can use. This design makes them a part any page can embed, `@compmd/devtools`, with three new tabs on what the sandbox does: its frames, a console, and the messages between them. For those, `@compmd/sandbox` gains an observer, which is security-sensitive and has a threat model of its own.

## Decisions

1. **`@compmd/devtools` is one package, a function a page calls**, not a framework component: `mountDevtools(target, options)` renders into `target` and gives a handle (`open`, `destroy`, events). It is written in Svelte 5, compiled into the package, so a page in React, plain JavaScript or anything else embeds it the same way, and needs no Svelte of its own (Q1).
2. **It renders inside a shadow root**, its styles with it, so a page's CSS can't reach in, and its CSS can't leak out. A page's `iframe { … }` rule already showed what a host's styles do to parts that aren't the host's: it made the sandbox's hidden compile frame visible.
3. **Two ways to embed it** (Q2):
   - **The editor**: the text, the page and the views of documents the embedding page gives, editable, as the playground is today. The playground, and so the guides' **Try it** links, use it.
   - **The inspector**: attached to a sandbox view a host app already renders, with its document; no text area, the views and the sandbox's tabs. An app developer opens it in development to see what their documents do. The same function takes the view: `mountDevtools(target, { view, document })`, and `open(document)` gives it the document again when the host processes it again. Its views worker processes the document for its views and for the diagnostics the Console lists; it fills the element the host gives it, with no window of its own.
4. **The playground becomes a page that embeds the editor.** It keeps what is the site's: the site's header and footer, and a toolbar with its name and a line on what it is for, the menu of documents (`src/documents.ts`, the examples, the guides' documents) and the host's stylesheets. The guides' **Try it** links stay `/playground/?doc=guides/<book>/<example>`.
5. **The sandbox gets an observer, which only the host's page can turn on** (`view.observe(listener)`): never a document, a configuration of packs, or a frame. Off, nothing is copied or recorded, and no frame is asked to report; each window keeps only which of its frames and components are live, so that turning it on shows the tree as it is. It reports what the host's page itself does and receives, and what frames report (decision 6):
   - **frames**, as the host's page makes and removes them: an id, the parent's id, the level, the network a permissive frame was given, what it renders (the page, a use and its component, the compile frame), and why it went (destroyed, stopped when it hung, of another protocol version);
   - **messages** on each port the host's page holds, both ways, as copies, with their time; pings and pongs aren't copied, since one a second from every frame would push the rest out;
   - **fallbacks**: each use that shows its fallback, why (W118, W121, `not-here`), and its component.
6. **The observer sees the whole hierarchy** (Q3), [walked through below](#the-observer-walked-through). It is built into the sandbox's protocol layer, which records what passes each `Port` as a runtime sets up a frame, with **one view per window**: the host's page's, and each frame's. A permissive frame's strict frames, and the uses inside a page frame (frame mode), are made in a frame, with ports the host's page doesn't hold, so each frame's view records its own part: the frames it makes, the messages on its ports, the uses it mounts, with each render's and each component's time. A frame records only once its parent asks (`observe {on}` down the tree), and sends its view's events up in batches (`observed {events}`); its parent passes on its children's unchanged. The host's view puts them together into one tree, which the tools read live. What a frame's view reports comes from the window that frame's components share, so the tools mark it as that frame's report, never act on it, and show it only as text (O6, O7, and the [accepted risk](#the-accepted-risk)).
7. **Three new tabs** on the observer:
   - **Console**: a history of what went wrong, newest last: the diagnostics as each change left them, component fallbacks with why, refused `set`s, frames stopped, packs that couldn't be fetched. The diagnostics stay under the text as they are; the console is their history, with the rest.
   - **Sandbox**: the frame tree, down to the uses in each frame, each frame's level and network, each fallback where it happened, and how long each frame and component took to render and update; what a frame reported is marked so.
   - **Messages**: the protocol on a timeline, a lane per frame, each message expandable as the trees are.
8. **Tabs, in order** (Q4; revised 2026-11-03, since nine tabs in one row were crowded, and 2026-11-06, with the inputs moved into the drawer and the text added): the editor has two, **Page** and **CompMD Inspector**. The Inspector holds the views as tabs of its own, the first open when it opens: **Parse tree**, **Expanded tree**, **Data**, **Components**, **Sandbox**, **Messages**, **Plain Text** (the document's text as written, read-only) and **Plain Markdown**. Under them, whichever shows, a drawer that hides (Esc) and resizes, as a browser's developer tools have it, with two tabs: the **Console**, and **Inputs**, the document's inputs, each with a control by its type, which the editor sets; Inputs shows when the document declares any. The page has nothing over it. The inspector alone, which has no page, shows the Inspector, without Inputs: the inputs are the host's. A layout of panes, as a browser's developer tools have, may later take their place.

## The observer's threat model

The observer adds code that reads what frames send, in the host's page, which strict code can't otherwise reach. What it must keep:

| # | Threat | Stopped by |
|---|---|---|
| O1 | A frame's message is shown as markup in the tools, in the host's page (a component sends `<img onerror=…>` as an error message) | the tools show messages only as text and as the JSON tree, never as HTML; a URL is text, never a link that opens by itself |
| O2 | A frame floods the observer to exhaust the host's memory | the observer keeps the last 2,000 events of each source, the host's page and each of its frames, and counts what it dropped, so a flood pushes out only its own; a parent takes at most 2,000 events of one report; a message's large values (a splice's JSON text, compiled code) are kept by their size and start ([the protocol](sandbox-protocol.md#observing)) |
| O3 | Observing changes what the sandbox does | the observer only reads: besides `observe` and `observed`, it posts nothing and changes no message; a browser test runs each document with and without it and compares every other message |
| O4 | A document or a pack turns the observer on | only the host's page's API does; nothing in a request, a configuration of packs or a message does |
| O5 | What it keeps leaves the page | it keeps copies in the host's page only, sends nothing anywhere, and they go with the view |
| O6 | A frame reports a false tree: children, messages or times that aren't so, since component code of its level shares its runtime's window | what a frame reports is marked as its report, in the Sandbox and Messages tabs, and the tools never act on it: the sandbox's own decisions (levels, `host`, which frames to stop) never read a report |
| O7 | Reporting becomes a channel from strict code to the host's page | a frame reports only after its parent asked, and only to its parent, as it already sends sizes and errors; a report is data the tools show, bounded as O2 says, and reaches no network: strict code can only fill the host's page's memory with it, up to the bound |
| O8 | Code behaves differently while it is observed | a frame learns it is observed when its parent asks, so code of its level can tell: the tools show what a frame does, not what it would do unobserved. Said in the tools' README |

**Browser tests**, in Chrome and WebKit as the sandbox's are: the frame events for a page in both modes; the whole tree with a permissive frame's strict frames and a page frame's uses, marked as reported; a component's forged report, marked and acted on by nothing (O6); a report before it was asked for, dropped (O7); a message shown as text (O1); a flood bounded (O2); the same messages with and without the observer (O3); a document that can't turn it on (O4).

## The observer, walked through

The [example](running-components-in-the-sandbox.md#the-examples) in host mode: `stars` trusted, `bar-chart` strict with `sparkline` in its slot, `map` permissive with a strict `legend` in its slot, and `rating-summary` permissive and given `host`. Its windows, each with its view:

```
H   the host's page       view H
├─ C   the compile frame
├─ U1  strict             bar-chart and sparkline   view U1
├─ U2  permissive         map                       view U2
│  └─ U3  strict          legend                    view U3
└─ U4  permissive         rating-summary, host      view U4
```

1. **The developer opens the tools.** They call `view.observe(listener)` in H. View H starts recording, and H sends `observe {on}` to C, U1, U2 and U4; U2 passes it on to U3. Nothing in a frame starts it.
2. **Each view records**, every event with its time:
   - **view H**, at first hand: the frames it makes, C and U1 to U4, each with its level, network and use; every message on its ports, both ways (`code`, `part`, `ping` and `pong`, `set`, `error`, `rendered`, `size`); `stars` mounted inline, with its render time; U4's `host` calls;
   - **view U1**: `bar-chart` and `sparkline` mounted, each one's render and update times, a `set` it refused;
   - **view U2**: U3 made in `map`'s slot, strict, with no network; every message between U2 and U3 (`hello`, `level`, `code`, `part`, `ping`, `set`); `map`'s render time;
   - **view U3**: `legend` mounted, its render time, its `error` if it throws;
   - **view U4**: `rating-summary`'s times, and each `host` call as U4 sent it.
3. **The views reach the tools.** Each frame sends `observed {events}` to its parent every 100 ms or so; U2 sends U3's batches on with its own. View H puts them together, and the tools update as batches arrive.
4. **The tools show** the tree above with each use under its frame, each with its level, network, render times and fallbacks, and every port's messages on the Messages tab, a lane per frame, U2 to U3 included. What came from views U1 to U4 is marked as reported by that frame; what view H recorded isn't.
5. **`map`'s code lies.** It shares U2's window with U2's runtime, so it can change what view U2 sends: a batch saying `legend` rendered in 2 ms with no network, when it did something else. The tools show that, marked as U2's report, and nothing acts on it: who gets `host`, which `set` passes and which frame is stopped come from the protocol's checks, never from a view.

In **frame mode**, H makes C and P, and P holds every use: view H records `code`, `render`, `splices` and P's `set` and `error`; view P records each use mounted, its times, and each `not-here` fallback. Without view P, the tools would show one box.

**Not seen:** what component code does outside the protocol, its own DOM, and the requests a permissive frame makes. The second could later be recorded too (CSP violation events, resource timing), as that frame's report.

## The accepted risk

**A frame's view can be false, and the tools can't tell.** The sandbox code that makes a frame's children and mounts its uses runs in that frame's window, beside its components, which can change it; a browser lets no other window read that frame's ports or its document. So what views U1 to U4 report is only as true as the code of their level. Two ways around it were weighed and turned down:

- **The host's page brokers every port**, making each channel and passing each message on. The parent still hands its child the port, in `hello`, so a lying parent can hand it its own and send the host's page made-up traffic; it adds a hop to every message, changes the protocol even when nothing observes (O3), and still can't see uses mounted inside a window or their times.
- **A control frame per frame**, with the runtime alone in it and the components one frame down: true isolation, at twice the frames and the latency, for tooling.

It is accepted because a lie reaches no decision (O6), and costs little: a parent is never less trusted than its children ([sandbox, decision 1](running-components-in-the-sandbox.md#proposed-decisions)), so code that can falsify a view controls what it reports on anyway. The tools say so where they mark a report, and their README says it.

## Questions, settled

**Q1. A function, a custom element, or a Svelte component?** Settled: a function, `mountDevtools(target, options)`. A custom element (`<compmd-devtools>`) needs its options as properties anyway, since documents, sandbox options and callbacks aren't attributes; it can wrap the function later. A Svelte component would tie embedding pages to Svelte.

**Q2. Both ways to embed in the first version?** Settled: in two increments (below), the editor first, the inspector next, with its API in this design so the editor doesn't change for it.

**Q3. Should frames report their own children?** Settled: yes, so the tools show the whole hierarchy (decision 6), each report marked as its frame's (O6 to O8). Seeing only what the host's page holds would show one frame in frame mode, where every use is inside the page's frame, and nothing under a permissive frame: not much of a use.

**Q4. The tabs' order.** Settled as decision 8. Before the tools, the playground's order was Page, Plain Markdown, Data, Components, Parse tree, Expanded tree; the tools' first was Page, Console, Components, Sandbox, Data, Plain Markdown, Messages, Parse tree, Expanded tree.

## In two increments

1. **The editor, on the observer.** `@compmd/devtools` with `mountDevtools` and the editor; the observer in `@compmd/sandbox`, with the frames' reports, the protocol's `observe` and `observed`, and its browser tests; the Console, Sandbox and Messages tabs; the playground on it.
2. **The inspector.** `mountDevtools` attached to a host's own view (decision 3), with the views and the sandbox's tabs; `@compmd/vite`'s example opens it in development.

## What changes

- **`@compmd/devtools`**, new: the editor, the views, the new tabs, `mountDevtools`; from `editor/src/` (the session, the worker, the processing, the views, the JSON tree, the split), moved, not copied.
- **`@compmd/sandbox`**: the observer, its events and its browser tests; each frame's runtime observing its part and reporting it; `observe` and `observed` in [the frame protocol](sandbox-protocol.md) and its schema; the threat model above in [running components in the sandbox](running-components-in-the-sandbox.md).
- **The playground** (`editor/`): the site's page, embedding the editor.
- **The guides**: nothing; their links open the playground as today.

## Not in this change

- A browser extension that attaches the inspector to any page, as React's developer tools do.
