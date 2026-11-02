# The Developer Tools

**Status: proposed.** The playground (`editor/`) already holds a document's developer tools: its text with the hint at the caret and the diagnostics, the rendered page, and views of what the processor makes of it (plain Markdown, the frontmatter's data, the components, the trees, the inputs). They live in one Svelte app that only the playground can use. This design makes them a part any page can embed, `@compmd/devtools`, with three new tabs on what the sandbox does: its frames, a console, and the messages between them. For those, `@compmd/sandbox` gains an observer, which is security-sensitive and has a threat model of its own.

## Decisions

1. **`@compmd/devtools` is one package, a function a page calls**, not a framework component: `mountDevtools(target, options)` renders into `target` and gives a handle (`open`, `destroy`, events). It is written in Svelte 5, compiled into the package, so a page in React, plain JavaScript or anything else embeds it the same way, and needs no Svelte of its own (Q1).
2. **It renders inside a shadow root**, its styles with it, so a page's CSS can't reach in, and its CSS can't leak out. A page's `iframe { … }` rule already showed what a host's styles do to parts that aren't the host's: it made the sandbox's hidden compile frame visible.
3. **Two ways to embed it** (Q2):
   - **The editor**: the text, the page and the views of documents the embedding page gives, editable, as the playground is today. The playground, and so the guides' **Try it** links, use it.
   - **The inspector**: attached to a sandbox view a host app already renders, with its document; no text area, the views and the sandbox's tabs. An app developer opens it in development to see what their documents do.
4. **The playground becomes a page that embeds the editor.** It keeps what is the site's: the header, the menu of documents (`src/documents.ts`, the examples, the guides' documents), the About, the links. The guides' **Try it** links stay `/playground/?doc=guides/<book>/<example>`.
5. **The sandbox gets an observer, which only the host's page can turn on** (`createSandbox({ observe })`, or `view.observe(listener)`): never a document, a configuration of packs, or a frame. Off, nothing is wrapped and nothing kept. It reports what the host's page itself does and receives:
   - **frames**, as the host's page makes and removes them: an id, the parent's id, the level, the network a permissive frame was given, what it renders (the page, a use and its component, the compile frame), and why it went (destroyed, stopped when it hung, of another protocol version);
   - **messages** on each port the host's page holds, both ways, as copies, with their time;
   - **fallbacks**: each use that shows its fallback, why (W118, W121, `not-here`), and its component.
6. **The observer sees only what the host's page holds** (Q3). A permissive frame's strict frames, and the uses inside a page frame (frame mode), are made by a frame, with ports the host's page doesn't hold. The observer reports them as what they are, inside a frame it can't see into, and adds no message for frames to describe their own children: what a lower level says about itself is never shown as fact. What the host's page knows anyway, from the processor (each use's component and trust level) and from what frames already report (fallbacks), the Sandbox tab shows under that frame.
7. **Three new tabs** on the observer:
   - **Console**: a history of what went wrong, newest last: the diagnostics as each change left them, component fallbacks with why, refused `set`s, frames stopped, packs that couldn't be fetched. The diagnostics stay under the text as they are; the console is their history, with the rest.
   - **Sandbox**: the frame tree, each frame's level and network, which uses are in which, and each fallback where it happened.
   - **Messages**: the protocol on a timeline, a lane per frame, each message expandable as the trees are.
8. **Tabs, in order** (Q4): **Page**, **Console**, **Components**, **Sandbox**, **Data**, **Plain Markdown**, **Messages**, **Parse tree**, **Expanded tree**. The page first, then why something is wrong, then what the document is made of, then the raw material.

## The observer's threat model

The observer adds code that reads what frames send, in the host's page, which strict code can't otherwise reach. What it must keep:

| # | Threat | Stopped by |
|---|---|---|
| O1 | A frame's message is shown as markup in the tools, in the host's page (a component sends `<img onerror=…>` as an error message) | the tools show messages only as text and as the JSON tree, never as HTML; a URL is text, never a link that opens by itself |
| O2 | A frame floods the observer to exhaust the host's memory | the observer keeps the last 2,000 messages per view and counts what it dropped; a message's large values (a splice's JSON text, compiled code) are kept by size, and in full only while the tools show them |
| O3 | Observing changes what the sandbox does | the observer only reads: it posts nothing, and a frame can't tell it is there; a browser test runs each document with and without it and compares every message |
| O4 | A document or a pack turns the observer on | only the host's page's API does; nothing in a request, a configuration of packs or a message does |
| O5 | What it keeps leaves the page | it keeps copies in the host's page only, sends nothing anywhere, and they go with the view |

**Browser tests**, in Chrome and WebKit as the sandbox's are: the frame events for a page in both modes, nested frames reported as unobservable, a message shown as text (O1), a flood bounded (O2), the same messages with and without the observer (O3), a document that can't turn it on (O4).

## Questions, with proposed answers

**Q1. A function, a custom element, or a Svelte component?** Proposed: a function, `mountDevtools(target, options)`. A custom element (`<compmd-devtools>`) needs its options as properties anyway, since documents, sandbox options and callbacks aren't attributes; it can wrap the function later. A Svelte component would tie embedding pages to Svelte.

**Q2. Both ways to embed in the first version?** Proposed: the editor first, with the three tabs, since the playground and the guides need it, and the inspector's API in this design so the editor doesn't have to change for it; the inspector itself as the next task. *Alternative:* both together, since the observer's value is mostly for app developers.

**Q3. Should frames report their own children?** Proposed: no (decision 6). *Alternative:* a permissive frame relays its strict frames' events, which the tools show marked "as the frame says": more to see in host mode, but the first information the host's page would take from a lower level about the sandbox itself, a new message in the protocol, and a frame that lies about its children would show a false tree.

**Q4. The tabs' order.** Proposed above (decision 8). Today's order is Page, Plain Markdown, Data, Components, Parse tree, Expanded tree.

## What changes

- **`@compmd/devtools`**, new: the editor, the views, the new tabs, `mountDevtools`; from `editor/src/` (the session, the worker, the processing, the views, the JSON tree, the split), moved, not copied.
- **`@compmd/sandbox`**: the observer, its events and its browser tests; the threat model above in [running components in the sandbox](running-components-in-the-sandbox.md).
- **The playground** (`editor/`): the site's page, embedding the editor.
- **The guides**: nothing; their links open the playground as today.

## Not in this change

- The inspector, if Q2 keeps it for the next task.
- A browser extension that attaches the inspector to any page, as React's developer tools do.
- Seeing inside frames (Q3), and timing a frame's own work.
