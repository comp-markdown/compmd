// The frame: a sandboxed iframe that renders what the editor sends, through `@compmd/render`'s frame protocol, and
// compiles the document's host-language components the editor sends on the same port, in the host's stylesheet the
// editor picks, with its math typeset.

import { serveFrame } from "@compmd/render/frame";
import { FRAME_HELLO, type ComponentFailed, type ComponentsMessage, type HostStyleMessage } from "../protocol.ts";
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
    if (data.type === "components") void components.set((data as ComponentsMessage).components);
    // The host's stylesheet: `frame.css` holds each, by this attribute, so a switch reloads nothing.
    if (data.type === "host-style") document.documentElement.dataset.host = (data as HostStyleMessage).style;
  });
  const container = document.getElementById("page");
  if (container) serveFrame(port, { container, components: components.mounts, math });
});
