# 7. Rendering and styling

A document's page should look like your app: its fonts, its colours, light or dark as your app is. This chapter builds a page styled with your app's stylesheet and theme tokens, with a switch between light and dark, that shows a link's address before it opens it, and shows beside it what a plain Markdown renderer makes of the same text. It ends with `@compmd/render` alone, for pages that need no sandbox.

## Where the page is drawn

`@compmd/render` draws an expanded tree as HTML and CSS, the reference rendering of the built-ins ([its README](../../render/README.md#the-reference-rendering-of-the-built-ins)): a `box` is a `<div class="compmd-box">` with its layout params as CSS, an input component a `<label>` with its control, an alert a `<div role="note">`. The sandbox uses it too: in frame mode the page is drawn by `@compmd/render` inside the strict frame, in host mode in your page.

So there are two places to style:

| | Frame mode (the default) | Host mode |
| --- | --- | --- |
| The page is in | a strict frame, a document of its own | your page |
| Your page's CSS | doesn't reach it | applies, as to any of your elements |
| Your stylesheets reach it by | `view.styles(urls)` | your page's own `<link>`s |
| Theme tokens | `view.theme(tokens)` | `view.theme(tokens)` |
| Its colour scheme | the container's `color-scheme`, sent to the frame | the container's `color-scheme` |

Components in frames of their own, in either mode, get the same: the sandbox gives every frame your stylesheets, the tokens and the scheme.

## The example

A document with boxes that use the theme's tokens, and a link:

```md snippet=styled-page/welcome.comp.md
---
team: the docs team
---

# Welcome aboard

::: box {padding=m border-radius=m background=accent}
Your first week, from :value[the docs team]{of=(md.doc.team)}.
::: /box

::: row {gap=m}
::: box {padding=m border=subtle border-radius=m}
**Monday:** read [the spec](https://compmd.hakanserce.com/spec/).
::: /box
::: box {padding=m border=subtle border-radius=m}
**Tuesday:** write a first document.
::: /box
::: /row
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/styled-page) in the playground.

The app's stylesheet for its documents, `public/page.css`, which Vite serves as `/page.css`:

```css snippet=styled-page/public/page.css
/* The app's look for its documents, which the page's frame loads: served from the app, as the frames' policy allows. */
body {
  font: 16px/1.6 Georgia, serif;
}

h1 {
  color: var(--compmd-accent);
}
```

The page: a switch, the document's page, a frame for the plain rendering, and a dialog for links:

```html snippet=styled-page/index.html
<!doctype html>
<html lang="en" style="color-scheme: light">
  <head>
    <meta charset="utf-8" />
    <title>A styled CompMD page</title>
  </head>
  <body style="background: Canvas; color: CanvasText">
    <button id="scheme">Dark</button>
    <main id="page" style="height: 20rem"></main>
    <h2>What a plain renderer shows</h2>
    <iframe id="plain" sandbox="" title="The plain rendering" style="width: 100%; height: 14rem; border: 0"></iframe>
    <dialog id="link">
      <p>This link goes to <output></output></p>
      <form method="dialog">
        <button value="cancel">Stay here</button>
        <button value="open">Open it</button>
      </form>
    </dialog>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

And `main.ts`, with the same `vite.config.ts` as the quick start:

```ts snippet=styled-page/main.ts
import { init, OpenDocument, plainHtml } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./welcome.comp.md?raw";

await init();

const opened = new OpenDocument({ ...request, path: "welcome.comp.md", files: { "welcome.comp.md": text }, now: new Date().toISOString() });
const view = createSandbox(sandbox).open(document.getElementById("page") as HTMLElement, { components: opened.components });

// The app's look: its stylesheet in the page's frame, and the theme tokens the built-ins' styles read.
view.styles([new URL("page.css", location.href).href]);
view.theme({ "--compmd-accent": "#0b7285", "--compmd-space-m": "1.25rem", "--compmd-radius-m": "12px" });
view.render(opened.expanded);

// Light or dark is the page's `color-scheme`; after the app switches it, the view reads it again.
const scheme = document.getElementById("scheme") as HTMLButtonElement;
scheme.addEventListener("click", () => {
  const dark = document.documentElement.style.colorScheme !== "dark";
  document.documentElement.style.colorScheme = dark ? "dark" : "light";
  scheme.textContent = dark ? "Light" : "Dark";
  view.scheme();
});

// A link the reader clicked in the page: show where it goes, and open it only when the reader says so.
const dialog = document.getElementById("link") as HTMLDialogElement;
view.on("link", ({ url }) => {
  const target = URL.parse(url, location.href);
  if (!target || !["https:", "http:", "mailto:"].includes(target.protocol)) return;
  (dialog.querySelector("output") as HTMLOutputElement).value = target.href;
  dialog.onclose = () => {
    if (dialog.returnValue === "open") window.open(target.href, "_blank", "noopener");
  };
  dialog.showModal();
});

// What a plain Markdown renderer shows of the same text, in a frame that runs nothing.
(document.getElementById("plain") as HTMLIFrameElement).srcdoc = plainHtml(text);
```

Run it: the heading is in Georgia and teal, the first box has the accent's background with rounder corners. *Dark* turns the page and its frame dark. A click on *the spec* opens the dialog with the link's address, and nothing opens until the reader says so. Under the page is the same text as GitHub would show it.

## Theme tokens

The built-ins' tokens are CSS custom properties ([spec §9.4](https://compmd.hakanserce.com/spec/#94-values-and-spacing)). `padding=m` is `var(--compmd-space-m, 1rem)`, `background=accent` a mix of `--compmd-accent`, so a theme changes them without a document changing:

| Token | Custom property | Default |
| --- | --- | --- |
| `s`, `m`, `l` spaces (`padding`, `gap`) | `--compmd-space-s`, `-m`, `-l` | `0.5rem`, `1rem`, `2rem` |
| `background=subtle`, `strong` | `--compmd-surface-subtle`, `-strong` | the text colour at 6%, 12% |
| `background=accent` | `--compmd-surface-accent` | `--compmd-accent` (`#0969da`) at 12% |
| `border=subtle`, `strong` | `--compmd-border-subtle`, `-strong` | a 1px line of the text colour at 20%, 45% |
| `border-radius=s`, `m`, `l`, `full` | `--compmd-radius-s`, `-m`, `-l`, `-full` | `4px`, `8px`, `16px`, `9999px` |

The defaults are made from the text colour, so they suit light and dark text with no theme at all. `view.theme(tokens)` sets the `--compmd-*` properties you give on the page's root and in every frame, in place of the last ones; other names are ignored. A frame's document doesn't inherit your page's custom properties, which is why the tokens go through the view and not your stylesheet.

## The host's stylesheets

`view.styles(urls)` gives the page's frame, and every component's frame, your stylesheets, in place of the last ones (`createSandbox`'s `stylesheets` option gives the first). The frames' policy loads stylesheets only from the origin that serves the frame documents, your app's with `@compmd/vite`, and a strict frame's fonts too, so serve them there; a stylesheet from a CDN doesn't load.

A stylesheet in a frame styles what the renderer writes: elements (`h1`, `table`), and the built-ins' classes, `compmd-box`, `compmd-input`, `compmd-alert`, `compmd-footnotes` and the others in the reference rendering. Each styled block also gets a numbered class, `compmd-b3`, which carries its own params' CSS: don't style those. A document can't style anything itself: its params are a closed vocabulary, and no param passes CSS through.

In host mode the page is in your page, and your page's CSS reaches it like any of your elements. `view.styles` then only reaches the frames of components that aren't trusted.

## Light and dark

The page takes its colour scheme from where you put it: the container's `color-scheme`, and the reader's preference when it allows both (`light dark`). The view reads it when it opens, at each render, when the reader's preference changes, and when printing starts and ends, and sends it to every frame, which sets it on its document. So a frame stays transparent on your page, its text in your page's colours.

An app that switches its own theme, with a class or a style as the example does, calls `view.scheme()` after it: the view reads the scheme again, and the frames and components follow. A component that draws with CSS colours needs nothing more; one that takes its colours when it draws, a diagram or a canvas, reads `context.scheme`, and gets an update when it changes ([chapter 10](10-your-apps-own-components.md)).

## Links and the `link` event

A component in a sandbox can't navigate or open a window ([spec §15](https://compmd.hakanserce.com/spec/#15-security), rule 4). When a reader clicks a link in a frame, the frame doesn't follow it: it sends the view `link`, `{url}`, with the `href` as written, and your app decides. Only a reader's real click sends it; a link to a place in the page, a footnote, stays in the frame.

- **In frame mode** every link of the page is in the frame, links written in Markdown too, so every click on one comes as `link`.
- **In host mode** the page's own links are your page's, and open as your other links do; `link` comes from the frames of components.

The spec's rule is to show the reader the full address before opening it: a document from anyone can write `[your bank](https://bank.example.evil/)`. The example resolves the address against the page, opens only `https:`, `http:` and `mailto:`, shows it in a dialog, and opens it in a new tab with `noopener` only when the reader asks. Never assign the URL to `location` unchecked: a component may send a `javascript:` URL.

## What a plain renderer shows

A CompMD document is a Markdown document, and GitHub, an editor's preview or any CommonMark renderer shows it without CompMD: the component lines as text, the content inside containers as Markdown, a leaf's fallback in its line ([spec §14](https://compmd.hakanserce.com/spec/#14-degradation-contract)). `plainHtml(text)` gives that rendering, CommonMark and GFM, as HTML, with raw HTML escaped. Writers use it to see how their document degrades; the example shows it under the page.

The example puts it in a frame with `sandbox=""`, which runs no script and opens no link in your page, rather than in your page: what it shows is the document's, which may be anyone's.

## Without the sandbox: pages and regions

`@compmd/render` draws a tree without `@compmd/sandbox`, in your page:

```ts
import { createPage } from "@compmd/render";

const page = createPage(container, {
  components: { stars: mountStars },                     // your own components, by name
  onSet: (input, value) => openAgain({ [input]: value }),
  tokens: { "--compmd-accent": "#0b7285" },
});
page.render(opened.expanded);                            // or page.splice(splices, change), as in chapter 5
```

It runs only the components you give it in `components`; every other component the document names shows its fallback, and `onError` hears of it as `missing`. Without a sandbox, though, the page is in your page: its links are your page's, and its images load from wherever they point. So use it alone for documents you trust as your own code, and the sandbox for any other.

Its options, beside `components`, `onSet`, `onError` and `tokens`: `code` renders the code blocks of a language (`{ mermaid: (source, target) => … }`), `math` renders TeX (`(tex, { display }, target) => …`), `wikiLink` says where a wiki link points, and `host` is what components get as `context.host`. The page has `theme(tokens)` and `scheme()` as a view has, and `destroy()`.

A **region** draws any list of nodes, for an editor that shows blocks in its own widgets: `createRegion(target, { components, stylesheet: page.stylesheet }).render(nodes, { inputs, headingLevel })`. Breakpoints are container queries, so a region outside a page needs a container to follow: the class `compmd-container` on its target, or on an element around it. Without one, `onError` reports `no-container` once. A region in a sandbox, one use per widget, is `HostRuntime`'s `mountUse` ([chapter 11](11-the-sandbox-closer.md)).

Next: [components and trust](08-components-and-trust.md), what the components in a document may reach, and who decides.
