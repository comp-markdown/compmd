// The frame: a sandboxed iframe that renders what the editor sends, through `@compmd/render`'s frame protocol, and
// compiles the document's host-language components the editor sends on the same port, in the host's stylesheet the
// editor picks, with its math typeset.

import { serveFrame } from "@compmd/render/frame";
import { FRAME_HELLO, type ComponentFailed, type ComponentsLoaded, type ComponentsMessage, type HostStyleMessage, type OpenLink, type SourceLink } from "../protocol.ts";
import { Components } from "./components.ts";
import { load } from "./load.ts";
import { typesetMath } from "./math.ts";

/** Math typeset by Temml, which loads the first time a page has math. */
const math = typesetMath(async () => (await import("temml")).default);

/** Each component's stylesheet, adopted by the frame's document. */
const sheets = new Map<string, CSSStyleSheet>();

function adopt(component: string, css: string | undefined): void {
  const old = sheets.get(component);
  sheets.delete(component);
  let next = document.adoptedStyleSheets.filter((sheet) => sheet !== old);
  if (css !== undefined) {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(css);
    sheets.set(component, sheet);
    next = [...next, sheet];
  }
  document.adoptedStyleSheets = next;
}

addEventListener("message", (event: MessageEvent) => {
  const [port] = event.ports;
  if (event.data !== FRAME_HELLO || !port) return;
  const components = new Components({
    load,
    adopt,
    report: (component, message) => {
      port.postMessage({ type: "component-failed", component, message } satisfies ComponentFailed);
    },
  });
  // Before `serveFrame` starts the port, so the components come before the first splice.
  port.addEventListener("message", ({ data }: MessageEvent<{ type?: string }>) => {
    if (data.type === "components") {
      void components.set((data as ComponentsMessage).components).then(() => {
        port.postMessage({ type: "components-loaded" } satisfies ComponentsLoaded);
      });
    }
    // The host's stylesheet: `frame.css` holds each, by this attribute, so a switch reloads nothing.
    if (data.type === "host-style") document.documentElement.dataset.host = (data as HostStyleMessage).style;
  });
  // A link to a passage of the text selects it in the editor, rather than looking for it on the page; a link to
  // another page opens in a new tab, which the editor opens: only for the reader's own clicks, since a component may
  // not navigate (spec §15).
  const linkOf = (target: EventTarget | null) => (target instanceof Element ? target.closest("a[href]") : null);
  // A press on a link to the text doesn't take the focus into the frame, which would take it from the selection.
  document.addEventListener("mousedown", (event) => {
    if (linkOf(event.target)?.getAttribute("href")?.includes(":~:text=")) event.preventDefault();
  });
  document.addEventListener("click", (event) => {
    const link = linkOf(event.target);
    const href = link?.getAttribute("href") ?? "";
    if (href.includes(":~:text=")) {
      event.preventDefault();
      port.postMessage({ type: "source-link", fragment: href } satisfies SourceLink);
    } else if (/^(https?|mailto):/i.test(href)) {
      event.preventDefault();
      if (event.isTrusted) port.postMessage({ type: "open-link", url: href } satisfies OpenLink);
    }
  });
  const container = document.getElementById("page");
  if (container) serveFrame(port, { container, components: components.mounts, math });
});
