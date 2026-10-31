// The frame runtime (docs/sandbox.md, Q7): what runs in `strict.html` and `permissive.html` before any component. It
// takes its port from its parent only, adds a permissive frame's second policy before it loads code, and renders its
// part: the page (frame mode, through `@compmd/render`'s frame protocol on the same port) or one use with its slots.
// A component of a lower level gets a nested strict frame; one of a higher level shows its fallback.

import { createRegion, type Region } from "@compmd/render";
import { serveFrame } from "@compmd/render/frame";
import { hostClient, type HostClient } from "../host-api.ts";
import { codeFor, Mounts } from "../mounts.ts";
import { serveParent, type FrameEnd } from "../protocol/frame-end.ts";
import { hasCode, HELLO, SANDBOX_VERSION, type ComponentCode, type ErrorReason, type Level, type Part } from "../protocol/messages.ts";
import { frameUse } from "../use.ts";
import { compile } from "./compile.ts";
import { Components } from "./components.ts";
import { load } from "./load.ts";
import { typesetMath } from "./math.ts";
import { narrowed, networkPolicy } from "./narrow.ts";

/** The level this document is for, from its root: a strict document stays strict whatever its parent says. */
const own: Level = document.documentElement.dataset.compmdFrame === "permissive" ? "permissive" : "strict";

/** WebRTC isn't covered by a content security policy (docs/sandbox.md, threat model): its constructors go first. */
for (const name of ["RTCPeerConnection", "webkitRTCPeerConnection", "RTCDataChannel", "RTCIceCandidate", "RTCSessionDescription", "RTCRtpSender", "RTCRtpReceiver", "RTCRtpTransceiver"]) {
  Object.defineProperty(globalThis, name, { value: undefined, writable: false, configurable: false });
}

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

/** Adds the second policy to the document's head, after the first, with `strict.html` next to this document. */
function narrow(network: string): void {
  const first = document.querySelector<HTMLMetaElement>('meta[http-equiv="Content-Security-Policy"]')?.content ?? "";
  const meta = document.createElement("meta");
  meta.httpEquiv = "Content-Security-Policy";
  meta.content = narrowed(first, network, new URL("strict.html", location.href).href);
  document.head.append(meta);
}

/** The host's stylesheets, as links the frame's policy allows from its own origin. */
function styles(urls: readonly string[]): void {
  for (const old of document.querySelectorAll("link[data-compmd-style]")) old.remove();
  for (const url of urls) {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = url;
    link.dataset.compmdStyle = "";
    document.head.append(link);
  }
}

/**
 * A link a reader clicked goes to the parent, which decides what opens; a component may not navigate (spec §15). A
 * link to a place in the page (a footnote) goes there; one to a passage of the text (`#:~:text=`) is the parent's, and
 * a press on it leaves the focus where it is, as an editor's selection needs.
 */
function forwardLinks(parent: FrameEnd): void {
  const linkOf = (target: EventTarget | null) => (target instanceof Element ? target.closest("a[href]") : null);
  const inPage = (href: string) => href.startsWith("#") && !href.includes(":~:text=");
  document.addEventListener("mousedown", (event) => {
    if (linkOf(event.target)?.getAttribute("href")?.includes(":~:text=")) event.preventDefault();
  });
  document.addEventListener("click", (event) => {
    const href = linkOf(event.target)?.getAttribute("href");
    if (href === undefined || href === null || inPage(href)) return;
    event.preventDefault();
    if (event.isTrusted) parent.link(href);
  });
}

/** Reports the content's size, for the parent to size the frame. */
function reportSize(parent: FrameEnd): void {
  const body = document.body;
  new ResizeObserver(() => {
    parent.size(Math.ceil(body.scrollWidth), Math.ceil(body.scrollHeight));
  }).observe(body);
}

function start(port: MessagePort): void {
  let level: Level = "strict";
  let components: readonly ComponentCode[] = [];
  /** The host's stylesheets, which this frame's nested frames get too. */
  let stylesheets: readonly string[] = [];
  // Every callback below runs on a message, after `parent` is set at the end.
  const report = (name: string, reason: ErrorReason, message: string) => {
    parent.error(reason, message, name);
  };
  const loaded = new Components({
    load,
    adopt,
    report: (name, message) => {
      report(name, "failed", message);
    },
  });
  const strictUrl = new URL("strict.html", location.href).href;
  const mounts = new Mounts({
    here: own,
    inline: loaded.mounts,
    frame: (target, node, inputs, component) => {
      let failed: (message: string) => void = () => undefined;
      const frame = frameUse(target, {
        url: strictUrl,
        level: "strict",
        code: codeFor(components, "strict"),
        node,
        inputs,
        styles: stylesheets,
        ping: { every: 1000, timeout: 5000 },
        on: {
          set: ({ input, value }) => {
            parent.set(input, value);
          },
          link: ({ url }) => {
            parent.link(url);
          },
          error: ({ reason, component: name, message }) => {
            report(name ?? component.name, reason, message);
          },
          failed: (failure) => {
            failed(failure.reason === "hung" ? "its frame stopped answering" : `its frame is of protocol version ${String(failure.version)}`);
          },
        },
      });
      return {
        update: (next, nextInputs) => {
          frame.update(next, nextInputs);
        },
        destroy: () => {
          frame.destroy();
        },
        code: (next) => {
          frame.code(codeFor(next, "strict"));
        },
        theme: (tokens) => {
          frame.theme(tokens);
        },
        styles: (urls) => {
          frame.styles(urls);
        },
        failed: (listener) => {
          failed = listener;
        },
      };
    },
    report,
  });
  /** What a permissive frame's components get as `host`, when the parent gives it, before the first part. */
  let host: HostClient | undefined;
  let region: Region | undefined;
  const regionFor = (target: HTMLElement): Region =>
    createRegion(target, {
      components: mounts.mounts,
      math,
      onSet: (input, value) => {
        parent.set(input, value);
      },
      onError: (error) => {
        if ("component" in error) report(error.component, error.reason, error.error instanceof Error ? error.error.message : "");
      },
      ...(host ? { host } : {}),
    });
  const parent: FrameEnd = serveParent(port, {
    level: (asked, network) => {
      level = own === "permissive" && asked === "permissive" ? "permissive" : "strict";
      if (own === "permissive") narrow(network ?? networkPolicy([]));
      // A page frame (frame mode) renders the page through `@compmd/render`'s protocol, on the same port.
      const page = document.getElementById("page");
      if (page) serveFrame(port, { container: page, components: mounts.mounts, math });
    },
    code: async (next) => {
      components = next;
      const loading = loaded.set(next.filter(hasCode).filter((component) => component.level === level));
      mounts.set(next);
      await loading;
    },
    part: ({ node, inputs }: Part) => {
      const target = document.getElementById("part");
      if (!target) return;
      region ??= regionFor(target);
      region.render([node], { inputs });
    },
    styles: (urls) => {
      stylesheets = urls;
      styles(urls);
      mounts.styles(urls);
    },
    host: (hostPort) => {
      host = hostClient(hostPort);
    },
    compile,
  });
  // A use frame's theme (a page frame's is `serveFrame`'s): the tokens on its part, and on its own nested frames.
  port.addEventListener("message", ({ data }: MessageEvent<{ type?: unknown; tokens?: Record<string, string> }>) => {
    if (data.type !== "theme" || !data.tokens) return;
    const target = document.getElementById("part");
    for (const [name, value] of Object.entries(data.tokens)) if (name.startsWith("--compmd-")) target?.style.setProperty(name, value);
    mounts.theme(data.tokens);
  });
  forwardLinks(parent);
  reportSize(parent);
}

/** The parent's hello, once: the frame's port, from the window that made it. */
addEventListener("message", function hello(event: MessageEvent<{ type?: unknown; version?: unknown }>) {
  const [port] = event.ports;
  if (event.source !== window.parent || event.data.type !== HELLO || !port) return;
  removeEventListener("message", hello);
  if (event.data.version !== SANDBOX_VERSION) return;
  start(port);
});
