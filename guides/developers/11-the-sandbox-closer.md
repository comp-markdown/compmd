# 11. The sandbox, closer

The chapters so far used the sandbox as a box: trees in, a page out. This one opens it. It renders one document in both page modes side by side, then goes through what each mode makes, what the frames load and how to serve them, how a frame that hangs is stopped, and how an editor renders a document block by block.

## Both modes, side by side

A document with a component it defines itself, used twice, and an input:

````md snippet=two-modes/status.comp.md
---
md:
  inputs:
    docs: number = 72
---

# Release status

Two meters, drawn by a component the document defines itself, so they run strict wherever they are shown.

```compmd component=meter signature
params:
  value: number
  label: string
```

```jsx component=meter runtime=react@19
export default function Meter({ value, label }) {
  const shown = Math.min(Math.max(Number(value) || 0, 0), 100);
  return (
    <label style={{ display: "grid", gap: 4 }}>
      <span>{label}: <strong>{shown}%</strong></span>
      <progress max="100" value={shown} />
    </label>
  );
}
```

:: meter[Docs: 72%] {label="Docs" value=(md.input.docs)}

:: meter[Tests: 90%] {label="Tests" value=90}

:: number-input[Docs done, in percent: 72] {bind=docs}
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/two-modes) in the playground.

A page with a place for each mode:

```html snippet=two-modes/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Frame mode and host mode</title>
  </head>
  <body style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem">
    <section>
      <h2>Frame mode</h2>
      <div id="frame" style="height: 24rem"></div>
    </section>
    <section>
      <h2>Host mode</h2>
      <div id="host"></div>
    </section>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

And `main.ts`, which opens the document once and gives the same tree to two views:

```ts snippet=two-modes/main.ts
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox, type View } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./status.comp.md?raw";

await init();

let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "status.comp.md", files: { "status.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// The same document twice: in one strict frame, and in this page with a frame per component use.
const views: View[] = (["frame", "host"] as const).map((page) => {
  const container = document.getElementById(page) as HTMLElement;
  const view = createSandbox({ ...sandbox, page }).open(container, { components: opened.components });
  view.on("error", ({ component, code, message }) => {
    console.warn(`${component}: ${code ?? "no diagnostic"}, ${message}`);
  });
  view.render(opened.expanded);
  return view;
});

// An input set in either view opens the document again, and both show it.
for (const view of views) {
  view.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    opened.free();
    opened = open();
    for (const each of views) each.render(opened.expanded);
  });
}
```

With the quick start's `vite.config.ts`, `npm run dev`. Both sides look the same: the heading, two meters, the number input. Set the input to 40 on either side, and both say *Docs: 40%*. Open your browser's element inspector to see the difference: on the left, one `iframe` holds the whole page; on the right, the heading, the text and the input are in your page, and each meter is an `iframe` of its own.

## Frame mode

`page: "frame"`, the default. The view makes one `strict.html` frame in the container, which fills it (give the container a height), and renders the page in it with `@compmd/render`'s frame protocol: trees and splices go down the frame's port, `set`, links and errors come up.

- **Strict components share the frame**, so ten charts cost one frame, and one copy of React.
- **Permissive and trusted components show their fallbacks**, each with an `error` event whose `reason` is `not-here`.
- **The page's links come to your page** as `link` events, since nothing in the frame may navigate.

It is right for documents from anyone with strict packs: a site of others' documents, a preview, a playground.

## Host mode

`page: "host"`. The view renders the page in the container itself, with `@compmd/render`, and decides for each use:

- **A trusted component mounts inline**, from `trusted`, as your own code does.
- **A strict or permissive component gets a frame where it is used**, `strict.html` or `permissive.html`, holding that use and its slots. The frame fills the use's box and reports its height, which the view sets; an inline use's frame reports its width too.
- **Built-in components, layout and components written in CompMD** run no code, so they render in your page, and a framed component in their slots gets its frame there.

Frames only go down in trust. Your page holds permissive and strict frames; a permissive frame holds strict frames, for strict components in its slots; a strict frame holds none. So a permissive or trusted component in the slot of a strict container shows its fallback, `not-here`, with a message that names why. Write such a container in CompMD, or, when it is your own, make it trusted.

The cost is a frame, and a copy of its runtime, per use. It is right for an app with components of its own, or permissive packs.

## What the frames load

`@compmd/vite` writes the frames' files into your app's `compmd/` folder (`buildFrames` does it for other hosts, [chapter 18](18-other-hosts.md)):

| File | What it is |
| --- | --- |
| `strict.html` | the strict frame: a fixed content security policy with no network and no nested frames, then the import map, then the frame runtime |
| `permissive.html` | the same, without the network directives; its parent sends the pack's origins, which the frame adds as a second policy before any component loads, and it may nest `strict.html` only |
| `runtime.js` and its files | the frame runtime: the sandbox's end of the protocol, the renderer, the bindings |
| `highlight.js`, `math.js` | Shiki and Temml, which highlight code blocks and typeset math: the frames load them through the runtime, and in host mode your page loads them from here, so your app ships each once |
| `libs/` | the runtimes every frame shares through the import map, React 19, React DOM and Svelte 5, and the libraries your configuration offers |
| `library-definition.json` | those runtimes and libraries with their versions, which the processor checks components against |
| `packs/<name>@<version>/…` | each installed pack's built modules |

A browser enforces every policy a document has, so the second policy can only narrow the first: no message can widen a frame's network.

Besides the frames that render, each view makes one hidden `strict.html` frame, the **compile frame**, which compiles the document's source components (JSX, TSX, Svelte, TypeScript) once, and runs none of them. Each frame then gets the compiled code it needs, and only that: a frame never holds code of a level above its own. A pack's **built module** loads in its frame by its URL with its file's hash, `<script type="module" integrity>`, so a file that doesn't match never runs (W121); or, for a pack fetched from a registry, as the checked file itself ([chapter 9](09-packs-from-npm.md)).

## Serving `compmd/`

The frames have an opaque origin: `sandbox="allow-scripts"` without `allow-same-origin`. So what they load from your app is a cross-origin load, and the folder needs two headers:

| Path | Header | Why |
| --- | --- | --- |
| everything under `compmd/` | `Access-Control-Allow-Origin: *` | the frames' scripts are modules, which load cross-origin only with it |
| `compmd/strict.html`, `compmd/permissive.html` | `Content-Security-Policy: sandbox allow-scripts` | the frame documents stay sandboxed even when someone opens one on its own, outside a frame |

The dev server and `vite preview` send both. A deployed app sends them from its server, or its static host's header rules. Nothing under `compmd/` is secret: it is the same for every reader.

The frames may be served from another origin than your page, with the same headers: `createSandbox`'s `base` takes a full URL. That is the way to keep the requests a frame can make to them (its scripts, styles and prefetches) apart from your app's own server and logs.

## Pings and stopped frames

Every frame that runs components is pinged by its parent every second. One that hasn't answered within `timeout` (5000 ms unless you give `createSandbox` another) is removed, and its use shows its fallback: a component that loops forever stops there. The view's `error` event tells you:

```ts
view.on("error", ({ component, reason, code, message }) => {
  // A stopped frame: reason "threw", code "W118", message "its frame stopped answering".
});
```

| `reason` | `code` | When |
| --- | --- | --- |
| `threw` | `W118` | the component threw, called `fail`, or its frame stopped answering |
| `failed` | `W121` | its code didn't compile or load: an import it may not make, a module that doesn't match its hash |
| `not-here` | none | its level can't run where it is used |
| `missing` | none | no code for it here |

`diagnosticOf(reason)`, from `@compmd/sandbox`, gives the code for a reason.

In frame mode the pinged frame is the page's: when it stops answering, the error's `component` is empty, and its message says *the page's frame stopped answering*. In WebKit, a frame that spins freezes your page with it, since WebKit runs these frames in the page's process; Chrome keeps the page going and stops the frame ([chapter 8](08-components-and-trust.md#the-threat-model-in-plain-words)).

## When the page has settled

A page keeps moving after `render()` returns: each frame takes its height once it has drawn and reported it, its own frames later still, and a component that draws late, as `reviews` does once your app answers ([chapter 10](10-your-apps-own-components.md)), later again. To print, to scroll to a link's target, or to click something in a test, wait for it to stop:

```ts
view.render(opened.expanded);
await view.settled();
```

`settled()` resolves once, after the latest render or splice, every use in your page and in every frame has drawn (a component that draws late says so with `busy`, spec §7.4), every frame has its last height, the page's images and fonts have loaded, and nothing moved for an animation frame; at once when that holds already. The view's `settled` event comes each time it settles again, and the container has the attribute `data-compmd-settled` while it is settled, so a browser test waits for it without reaching into your app: `await page.locator("#page[data-compmd-settled]").waitFor()`. A frame that hangs is stopped and counts as settled; a drawing or a frame that never says it is done holds it up for 10 s at most.

## Editors that render block by block

An editor that shows a document's blocks in its own widgets, as CodeMirror's are, renders block by block rather than as a page. It uses the layer under `createSandbox`, the **host runtime**, one per document, and mounts each use in its widget:

```ts
import { HostRuntime } from "@compmd/sandbox";
import { sandbox } from "virtual:compmd";

const runtime = new HostRuntime(sandbox, {
  set: (input, value) => setInput(input, value),
  link: (url) => confirmAndOpen(url),
  error: (component, reason, message) => report(component, reason, message),
});
await runtime.set(opened.components);

// In a widget, for one use of the expanded tree, with the document's inputs.
const use = runtime.mountUse(widget, node, opened.expanded.inputs ?? {});
use.update(nextNode, nextInputs);
use.destroy();
```

`mountUse` does what host mode does for one use: a trusted component inline, any other in a frame of its level. `runtime.mounts` is the same as a `components` map for `@compmd/render`'s regions, for the blocks around the uses.

An editor removes widgets that scroll out of view and builds them again, and a frame whose element is removed or moved loads again: its component boots again, and its state is lost. So a component's state belongs in its params and inputs, and an editor that minds keeps a widget's DOM while it is near the viewport.

## A host page that is itself sandboxed

The window where the sandbox runs, the **host's page**, needn't be your top-level page. An app may run its editor in a sandboxed frame of its own; that frame is then the host's page, with an opaque origin, its trusted components are the app's code in it, and its `host` is what the app gives that frame. It works because a frame believes its parent and nothing else: it takes its port, its level, its code and its part only from the window that made it, by that window, never by an origin. A message from any other window, such as a strict frame posing as a parent through `top.frames`, is ignored.

## The protocol, in brief

The windows talk over `MessageChannel` ports, one per frame, to its parent:

| Window | Made by | Runs |
| --- | --- | --- |
| the host's page | your app | the host runtime; in host mode the page and trusted components |
| the page frame (frame mode) | the host's page | the page and its strict components |
| a use frame, strict or permissive | the host's page; a permissive frame, for a strict use in its slots | one use and its slots |
| the compile frame | the host's page | the compilers, no components |

A parent sends `hello` with the port, then `level` (and a permissive frame's second policy), `code`, and the part to render; it pings, and sends `host` to permissive frames of packs given it. A frame sends `ready`, `loaded`, `set`, `link`, `error`, `rendered`, its `size`, `settled` once it has, and `pong`. A parent checks each `set` against the use it rendered, so a frame can only set the inputs its own use binds, and passes what concerns the windows above it up. [The protocol's design](../../docs/sandbox-protocol.md) has every message and sequence; `@compmd/sandbox/protocol` is the protocol alone, with no DOM, and `protocol.schema.json` describes its messages, for an implementation elsewhere ([the sandbox's README](../../sandbox/README.md#the-protocol)).

Next: [the Developer Tools](12-the-developer-tools.md), which show all of this as it happens.
