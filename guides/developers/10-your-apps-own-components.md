# 10. Your app's own components

Your app can give its documents components of its own: a rating, a link to a record, a chart of its data. This chapter builds a page with three. `stars`, in React, and `spoiler`, in Svelte, are trusted: built with the app, they mount in your page. `reviews` is permissive and given `host`: it runs in a frame of its own, and asks your app for what it needs.

## A folder is a pack

A pack in your app is a folder with a `package.json` and a document, as a pack on npm is. The `package.json` names it:

```json snippet=own-components/components/package.json
{
  "name": "@you/app",
  "version": "1.0.0",
  "private": true
}
```

Its document, `components/index.comp.md`, imports each component's file, exports the components under the pack's name and version, and declares their signatures, which every use is checked against before anything runs:

````markdown snippet=own-components/components/index.comp.md
---
md:
  import:
    - from: ./Stars.tsx
      names: [{ name: Stars, as: stars }]
    - from: ./Spoiler.svelte
      names: [{ name: Spoiler, as: spoiler }]
  export:
    pack: "@you/app"
    version: "1.0.0"
    names: [stars, spoiler]
---

The app's own components, built with it and trusted: they mount in its page.

```compmd component=stars signature
params:
  value: input<number>
  of: number = 5
```

```compmd component=spoiler signature
params:
  label: string
slots: [default]
```
````

A component file exports one component, its default export, named after its file (`Stars.tsx` exports `Stars`); `as` gives it the name documents use. `value: input<number>` makes `value` a param a document binds to one of its inputs, which the component may set. `slots: [default]` gives `spoiler` a body: it is a container, and what a document writes in it is its default slot.

## The components

The file's extension says how it is mounted:

| Extension | Binding | Built with |
| --- | --- | --- |
| `.ts`, `.js` | plain: the default export is the host contract's `mount` | Vite alone |
| `.tsx`, `.jsx` | React 19: params are props, the default slot is `children`, the rest is the `compmd` prop | Vite alone, with `react` and `react-dom` installed |
| `.svelte` | Svelte 5: params are props, the default slot is the `children` snippet, the rest is the `compmd` prop | Svelte's plugin, `@sveltejs/vite-plugin-svelte`, with `svelte` installed |

A param is a prop of the same name, in camelCase. The `compmd` prop holds the rest of the host contract ([spec §7.4](https://compmd.hakanserce.com/spec/#74-the-host-contract)): `set`, `fallback`, `scheme`, `fail` and the others.

`Stars.tsx` is a React component:

```tsx snippet=own-components/components/Stars.tsx
import type { ReactElement } from "react";

interface Props {
  value: number;
  of: number;
  compmd: { set(param: string, value: unknown): void; fallback: { text: string } };
}

/** The most stars a document may ask for. */
const MOST = 10;

/** `value` as a whole number from 0 to `max`, whatever a document gave. */
function wholeUpTo(value: unknown, max: number): number {
  const number = Math.round(Number(value));
  return Number.isFinite(number) ? Math.min(Math.max(number, 0), max) : 0;
}

// A rating of `of` stars, `value` of them filled; a click sets the input `value` is bound to.
export default function Stars({ value, of, compmd }: Props): ReactElement {
  const count = Math.max(1, wholeUpTo(of, MOST));
  const filled = wholeUpTo(value, count);
  return (
    <span role="radiogroup" aria-label={compmd.fallback.text}>
      {Array.from({ length: count }, (_, i) => (
        <button
          key={i}
          type="button"
          role="radio"
          aria-checked={i + 1 === filled}
          aria-label={`${String(i + 1)} of ${String(count)}`}
          onClick={() => {
            compmd.set("value", i + 1);
          }}
        >
          {i < filled ? "★" : "☆"}
        </button>
      ))}
    </span>
  );
}
```

The binding renders it into a React root in its box, and on an update renders it again into the same root, so React keeps its state. `compmd.fallback.text` is the text a reader without it sees, *3 of 5 stars*, which labels the group; `compmd.set("value", …)` changes the input `value` is bound to. It is [`@compmd/vite`'s example's `Stars`](../../vite/example/components/Stars.tsx), with its numbers kept in bounds (below).

`Spoiler.svelte` is a Svelte component, which hides its body until the reader asks:

```svelte snippet=own-components/components/Spoiler.svelte
<!-- Its content, hidden until the reader asks: a button shows it, and hides it again. -->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    children?: Snippet;
  }

  let { label, children }: Props = $props();
  let shown = $state(false);
</script>

<button type="button" aria-expanded={shown} onclick={() => (shown = !shown)}>
  {shown ? "Hide" : "Show"} {label}
</button>
<div hidden={!shown}>
  {@render children?.()}
</div>
```

`children` is its body: the binding places in it what the renderer made of the Markdown a document writes in `spoiler`. `shown` is the component's own state: a click shows the body, the next hides it again, and `aria-expanded` tells a screen reader which it is. `spoiler` never sets an input, so it doesn't use the `compmd` prop. [The website's own `stars`](../../website/app/Stars.svelte) is a Svelte one too.

Vite compiles JSX itself, so React needs no plugin: `"jsx": "react-jsx"` in your `tsconfig.json` gives it React's automatic runtime. A `.svelte` file needs Svelte's compiler, which its plugin brings, so this app's `vite.config.ts` is the quick start's with `svelte()` added:

```ts snippet=own-components/vite.config.ts
import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [svelte(), compmd()],
});
```

## Params are untrusted input

Your component is trusted, but the documents that use it may be anyone's, and so are its params. The signature checks each param's type, and the values an enum allows; it can't check that a number is in a range, or that two params agree. So a trusted component:

- **never makes HTML from a param**: in React and Svelte, text and attributes in the template, as `Stars` and `Spoiler` do, never `dangerouslySetInnerHTML` or `{@html}`; in a plain component, `textContent` and attributes, never `innerHTML`;
- **keeps numbers in bounds**: here a document that asks for a million stars gets ten, and a value past `of` fills them all;
- **checks URLs** before it shows a link from a param, as the processor's interpolation does: `https:` and the like, never `javascript:`;
- **acts outside the document only when the reader asks**, after showing what it will do ("Send *notes.csv* to …?"): a document that isn't yours can aim it at what it shouldn't touch ([spec §15](https://compmd.hakanserce.com/spec/#15-security)).

The same holds for anything the component reads through `set`'s bound input: it came from the reader or the document.

## Trusting it

`compmd.config.json` lists the folder, trusted, and ambient, so every document gets `stars` and `spoiler` without importing them. It lists the second pack of this chapter too, `reviews`, which comes below:

```json snippet=own-components/compmd.config.json
{
  "packs": {
    "./components": { "trust": "trusted", "ambient": true },
    "./reviews": { "trust": "permissive", "host": true }
  }
}
```

A folder pack is listed by its path, from your app's root; documents can also import it by its name, `@you/app`. `@compmd/vite` reads it when your app builds: it checks the pack's document through the processor, imports `Stars.tsx` and `Spoiler.svelte` into your app's bundle, each wrapped in its binding (a plain file stays as it is), and gives them to the sandbox in `virtual:compmd`'s `sandbox.trusted`. Their code is never sent to a frame.

## The document

`notes.comp.md` uses `stars`, bound to its input `rating`, and `spoiler` around a note on the ending, and imports `reviews` by its pack's name:

```markdown snippet=own-components/notes.comp.md
---
md:
  import:
    - from: "@you/reviews@1"
      names: [reviews]
  inputs:
    rating: number = 3
---

# Reading notes: *The Left Hand of Darkness*

My rating: :value[3]{of=(string(md.input.rating))} of 5.

:: stars[3 of 5 stars] {value=rating}

:: reviews[What other readers thought] {book="The Left Hand of Darkness"}

::: spoiler {label="the ending"}

Estraven is shot at the border, and Genly finishes the mission without him.

::: /spoiler
```

It needs this app's components, so the playground shows their fallbacks: *3 of 5 stars*, *What other readers thought*, and the note on the ending, since a container's fallback is its body. Any other app shows the same fallbacks, as a plain Markdown renderer does.

## Permissive components with `host`

`reviews` shows what other readers thought of a book. The reviews are your app's data, so the component asks your app for them, through `host`. It is a pack folder too:

```json snippet=own-components/reviews/package.json
{
  "name": "@you/reviews",
  "version": "1.0.0",
  "private": true
}
```

````markdown snippet=own-components/reviews/index.comp.md
---
md:
  import:
    - from: ./reviews.ts
      names: [reviews]
  export:
    pack: "@you/reviews"
    version: "1.0.0"
    names: [reviews]
---

What other readers thought of a book, from the app's data: permissive, and given `host`.

```compmd component=reviews signature
params:
  book: string
```
````

Listed as `"trust": "permissive", "host": true`, with no `network`, it gets a frame of its own with no network, and `host` in its context. Its source is compiled in the sandbox, as a document's own components are, so it imports nothing but what the frames provide:

```ts snippet=own-components/reviews/reviews.ts
/** What a permissive component given `host` gets: the app's functions, called by name, answered by its page. */
interface Host {
  call(name: string, ...args: unknown[]): Promise<unknown>;
}

/** What the renderer gives a plain component (spec §7.4), as far as `reviews` uses it. */
interface Context {
  params: Record<string, unknown>;
  host?: unknown;
  busy(work: Promise<unknown>): void;
}

/** What the app answers for a book it has reviews of. */
interface Summary {
  count: number;
  average: number;
}

const isSummary = (value: unknown): value is Summary =>
  typeof value === "object" && value !== null && typeof (value as Summary).count === "number" && typeof (value as Summary).average === "number";

// A summary of a book's reviews, which it asks the app for: the reviews are the app's data, never the document's.
export default function reviews(target: Element, { params, host, busy }: Context) {
  const line = target.ownerDocument.createElement("p");
  target.append(line);
  /** How many times it asked: an answer to an older question is dropped. */
  let asked = 0;
  const show = async (next: Record<string, unknown>) => {
    if (host === undefined) {
      line.textContent = "Reviews aren't available here.";
      return;
    }
    const question = ++asked;
    line.textContent = "Asking for the reviews…";
    let summary: unknown;
    try {
      summary = await (host as Host).call("reviewsOf", String(next.book));
    } catch {
      summary = undefined;
    }
    if (question !== asked) return;
    line.textContent = isSummary(summary) ? `${String(summary.count)} readers gave it ${summary.average.toFixed(1)} of 5 on average.` : "No reviews yet.";
  };
  // It draws once the app answers: `busy` tells the page, which waits for it to settle.
  busy(show(params));
  return {
    update: (next: Record<string, unknown>) => {
      busy(show(next));
    },
  };
}
```

`host.call(name, ...args)` sends the call to your page over the frame's port, and resolves with what your function returned. Arguments and results cross by copy, so they must be copyable (no functions, no DOM). A name your app doesn't offer, or a function that throws, rejects the call with a message. A component without `host`, in an app that doesn't give it, gets `undefined`, and should say so rather than fail.

`busy(work)` says the component is still drawing until `work` settles, here until the answer is shown ([spec §7.4](https://compmd.hakanserce.com/spec/#74-the-host-contract)). Nothing shows differently meanwhile, but a page that waits to settle, to print, to scroll to a link's target, or in a test, waits for it: the view's `settled()` ([chapter 11](11-the-sandbox-closer.md)).

What `host` holds is your app's: `createSandbox`'s `host` option, a `HostApi`, an object of functions by name.

**Your trusted components get the same `host`**, in their `compmd` prop (`compmd.host`, or `context.host` for a plain one), answered in your page directly: the same `call`, failing the same way, so a component written for `host` works trusted or permissive. Its type is `HostClient`, from `@compmd/sandbox`.

**Your app can also tell them something changed.** `view.emit(event, data)` sends an event to every component given `host`, trusted or permissive, and each hears it with `host.on(event, listener)`, which returns the function that stops listening: `view.emit("reviews-changed", { book })` after a reader adds a review, so `reviews` asks again. The data crosses by copy too. Strict components never hear it.

## The page

The page as in the quick start, and `main.ts`, which renders the document in host mode with the app's `host`:

```html snippet=own-components/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>The app's own components</title>
  </head>
  <body>
    <main id="page"></main>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

```ts snippet=own-components/main.ts
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox, type HostApi } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

await init();

// The app's data, which only the app's page reads: a document can't, and a component asks for it through `host`.
const summaries = new Map([["The Left Hand of Darkness", { count: 12, average: 4.2 }]]);

// What trusted components, and permissive ones of packs given `host`, may call. Arguments come from documents: check them.
const host: HostApi = {
  reviewsOf: (book) => (typeof book === "string" ? (summaries.get(book) ?? null) : null),
};

let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// Host mode: the page in this page, `stars` and `spoiler` inline, `reviews` in a permissive frame of its own.
const page = document.getElementById("page") as HTMLElement;
const view = createSandbox({ ...sandbox, page: "host", host }).open(page, { components: opened.components });
view.render(opened.expanded);

view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded);
});
```

`page: "host"` matters: in frame mode the whole page is one strict frame, where a trusted component can't run, so `stars` and `spoiler` would show their fallbacks, and the view's `error` event would say `not-here`.

The functions in `host` are where your app's data meets documents that may be anyone's. `reviewsOf` takes its argument from a param, so it checks its type, and looks it up in a `Map`, which has no inherited keys for a document to name. A function that reads only what the reader may see anyway, as this one does, needs nothing more; one that acts (sends, writes, shares) asks the reader first, in your page's own interface.

## Run it

In the quick start's app, add these files, set `"jsx": "react-jsx"` in its `tsconfig.json`, and install the components' frameworks:

```sh
npm install react react-dom svelte @sveltejs/vite-plugin-svelte
npm install -D @types/react
npm run dev
```

The page shows the heading, *My rating: 3 of 5*, three of five stars filled, in a frame below them *12 readers gave it 4.2 of 5 on average.*, and a button, *Show the ending*. Click the fifth star, and the line says *5 of 5*. Click the button, and the note on the ending shows, and the button says *Hide the ending*.

## What just happened

- `@compmd/vite` found both folders the configuration lists, read their documents, and gave the processor both packs, with their levels, in `request`. It built `Stars.tsx` and `Spoiler.svelte` into your app, each in its binding, and gave them to the sandbox as `sandbox.trusted.stars` and `sandbox.trusted.spoiler`.
- The processor resolved `stars` and `spoiler` from the ambient pack and `reviews` from the import, checked each use against its signature, and marked each component with its pack's level.
- In host mode, the sandbox rendered the page in `#page` and mounted `stars` and `spoiler` inline, as any of your code. A click on a star called `set("value", 5)`; the view sent `set` for the input `rating`, and your page opened the document again with it. A click on the button changed only `spoiler`'s own state.
- For `reviews`, it compiled the source in its hidden compile frame, made a `permissive.html` frame where the use is, and gave it a port for `host`. The component's `host.call("reviewsOf", …)` came to your page's `reviewsOf` over that port, and its answer went back the same way.
- Only the host's page gives `host`, and only to its trusted components and to frames of packs the configuration gives it. A strict component, or a document, can't get it.

Next: [the sandbox, closer](11-the-sandbox-closer.md), both page modes, the frames and how to serve them.
