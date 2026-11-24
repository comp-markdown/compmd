// One use in a frame of its own: the frame document of its level, the protocol's parent end on the port the parent
// gives it, and the frame over a sizer of its component's natural size, which it reports. The host's page frames strict and permissive uses this way,
// and a permissive frame its strict ones (docs/running-components-in-the-sandbox.md, decision 1).

import { BUSY_MS, sameType, typeStyleOf, type ExpandedInput, type ExpandedUse, type TypeStyle } from "@compmd/render";
import { fontFilesOf, namesFonts } from "./box-fonts.ts";
import { fallbackUntilDrawn } from "./fallback-until-drawn.ts";
import { followLoads, sandboxedFrame } from "./frame-loads.ts";
import { framedNode } from "./framed-node.ts";
import { networkPolicy } from "./frame/narrow.ts";
import { forSent, sentBlocks, type FrameHighlight } from "./frames-highlight.ts";
import type { Observer } from "./observer.ts";
import { boundInputs } from "./protocol/inputs.ts";
import { HELLO, SANDBOX_VERSION, type ComponentCode, type Level, type Scheme } from "./protocol/messages.ts";
import { connectChild, type ChildEnd, type ChildEvents } from "./protocol/parent-end.ts";
import { sizerIn } from "./sizer.ts";

export interface UseFrameOptions {
  /** The frame document of the use's level: `strict.html` or `permissive.html`. */
  url: string;
  level: Level;
  /** A permissive use's origins. */
  network?: readonly string[];
  /** A port for `host`, for a permissive use of a pack given `host`. */
  host?: () => MessagePort;
  code: readonly ComponentCode[];
  node: ExpandedUse;
  inputs: Readonly<Record<string, ExpandedInput>>;
  /** The use's fallback, which its box shows until the frame says its component has mounted (spec §7.4). */
  fallback?: Element;
  /** The number of the example the use is in (spec §7.8); absent for the document's. */
  example?: number;
  styles?: readonly string[];
  /** The host's own URL schemes, checked, which the links and images the frame renders may use (spec §15). */
  schemes?: readonly string[];
  /** Where the host serves the packs' files, ending in `/`, which the frame's links and images to them are under. */
  packs?: string;
  /** The tokens of a code block in the use's slots, which the frame asks for; asked only for a block of its node. */
  highlight?: FrameHighlight;
  ping?: { every: number; timeout: number };
  on: { [T in keyof ChildEvents]?: (event: ChildEvents[T]) => void };
  /** The observer of the window the frame is made in, which records the frame and the messages on its port. */
  observer?: Observer;
}

export interface UseFrame {
  readonly iframe: HTMLIFrameElement;
  update(node: ExpandedUse, inputs: Readonly<Record<string, ExpandedInput>>): void;
  code(code: readonly ComponentCode[]): void;
  /** Sends the page's theme tokens, as `@compmd/render`'s `theme` message. */
  theme(tokens: Readonly<Record<string, string>>): void;
  /** The host's stylesheets, in place of those the frame had. */
  styles(urls: readonly string[]): void;
  /** The page's colour scheme, which the frame's document takes. */
  scheme(scheme: Scheme): void;
  /** Sends the box's type to the frame's root again when it changed: after the host switched its theme. */
  restyle(): void;
  /** Resolves once the frame has settled after everything sent it, its size applied; or it is gone. */
  settled(): Promise<void>;
  destroy(): void;
}

/** The next use frame's key in this window, which the use ids its frame reports are prefixed with. */
let nextFrame = 0;

/**
 * Loads of a use frame the parent didn't start, within `UNASKED_WITHIN` ms, after which the frame is failed: its code
 * keeps navigating it, and each frame loaded again would only run it again (docs/running-components-in-the-sandbox.md,
 * decision 10). A widget an editor moves doesn't move that often.
 */
const UNASKED_LOADS = 3;
const UNASKED_WITHIN = 10_000;

/**
 * Frames `options.node` in `target`. The frame connects after each load of its document the parent started, and gets
 * its level, code and part each time. After any other load, its code navigating it or its element moved, as when an
 * editor moves its widget, what loaded gets nothing, and the parent loads the frame's document again.
 *
 * @example
 * const frame = frameUse(box, { url: strictUrl, level: "strict", code, node, inputs, on: { set: ({ input, value }) => setInput(input, value) } });
 * frame.update(next, inputs);
 */
export function frameUse(target: Element, options: UseFrameOptions): UseFrame {
  const document = target.ownerDocument;
  const iframe = sandboxedFrame(document, options.node.type === "code" ? (options.node.component?.name ?? "") : options.node.name);
  iframe.className = "compmd-frame";
  // The frame is over the sizer, the box's inside, and adds nothing to its size (docs/sizing-components.md, *The
  // mechanism*).
  iframe.style.cssText = "position: absolute; inset: 0; width: 100%; height: 100%; border: 0; display: block;";
  const placed = sizerIn(target, options.node.type === "componentInline");
  const { sizer, natural } = placed;
  const drawn = options.fallback ? fallbackUntilDrawn(target, sizer, options.fallback) : () => undefined;
  let node = options.node;
  let inputs = options.inputs;
  /** The example the use is in (spec §7.8), whose components the frame finds its node's in. */
  const inExample = options.example === undefined ? {} : { example: options.example };
  let bound = boundInputs([node]);
  /** The code blocks of the node sent, which alone the frame gets tokens for. */
  const blocks = sentBlocks();
  blocks.rendered([node]);
  let code = options.code;
  let styles = options.styles ?? [];
  let tokens: Readonly<Record<string, string>> | undefined;
  let scheme: Scheme | undefined;
  let child: ChildEnd | undefined;
  /** When each load the parent didn't start came, within the last `UNASKED_WITHIN` ms. */
  let unasked: number[] = [];
  /** Resolves once the frame has connected the first time, or is gone before it did. */
  let connected: () => void = () => undefined;
  const first = new Promise<void>((resolve) => {
    connected = resolve;
  });
  const key = `f${String(nextFrame++)}`;
  /** The box's type as last sent, which the frame's root has. */
  let sentType: TypeStyle | undefined;
  /**
   * Sends `style` again with the files of the document's own fonts it names (spec §9.4), once they are had, while it
   * is still the type sent: the frame declares them, which the page's `@font-face` rules don't reach.
   */
  const sendFonts = (style: TypeStyle) => {
    if (!namesFonts(style.family)) return;
    void fontFilesOf(style.family).then((fonts) => {
      if (sentType === style && fonts.length > 0) child?.typeStyle(style, fonts);
    });
  };
  /** Sends the box's type when it changed since it was last sent: its params, a theme, the page's styles. */
  const restyle = () => {
    const now = typeStyleOf(target);
    if (sentType && sameType(now, sentType)) return;
    sentType = now;
    child?.typeStyle(now);
    sendFonts(now);
  };
  const { observer } = options;
  const id = observer?.frame(options.level, { kind: "use", component: iframe.title }, options.network);
  const connect = () => {
    child?.close();
    const { port1, port2 } = new MessageChannel();
    iframe.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]);
    const end = connectChild(observer && id !== undefined ? observer.port(id, port1) : port1, {
      level: options.level,
      ...(options.network ? { policy: networkPolicy(options.network) } : {}),
      ...(options.schemes?.length ? { schemes: options.schemes } : {}),
      ...(options.packs === undefined ? {} : { packs: options.packs }),
      binds: (input) => bound.has(input),
      ...(options.host ? { host: options.host() } : {}),
      ...(options.highlight ? { highlight: forSent(blocks, options.highlight) } : {}),
      ...(options.ping ? { ping: options.ping } : {}),
    });
    if (observer && id !== undefined) {
      observer.connected(id, end);
      // Before the use's own listener, which removes the frame: it went because it failed.
      end.on("failed", ({ reason }) => {
        observer.gone(id, reason);
      });
      end.on("refused", ({ input }) => {
        observer.refused(id, input);
      });
    }
    // The component's natural size, which the box takes as if it held the component; never without area: WebKit may
    // stop rendering a frame with none, and its observers with it.
    end.on("size", ({ width, height }) => {
      natural.style.width = `${String(Math.max(1, width))}px`;
      natural.style.height = `${String(Math.max(1, height))}px`;
    });
    // The component mounted in the frame: its drawing takes the fallback's place.
    end.on("rendered", drawn);
    // A use id is its window's: this frame's key goes in front, so ids from different frames never meet.
    const { error, recovered, ...rest } = options.on;
    if (error) {
      end.on("error", (event) => {
        error(event.use === undefined ? event : { ...event, use: `${key}/${event.use}` });
      });
    }
    if (recovered) {
      end.on("recovered", (event) => {
        recovered({ ...event, use: `${key}/${event.use}` });
      });
    }
    for (const [type, listener] of Object.entries(rest)) end.on(type as keyof ChildEvents, listener as (event: unknown) => void);
    if (styles.length > 0) end.styles([...styles]);
    if (scheme) end.scheme(scheme);
    sentType = typeStyleOf(target);
    end.typeStyle(sentType);
    sendFonts(sentType);
    end.code([...code]);
    if (tokens) end.send({ type: "theme", tokens: { ...tokens } });
    end.part({ node: framedNode(node, target), inputs: { ...inputs }, ...inExample });
    end.box(sizer.getBoundingClientRect().width);
    child = end;
    connected();
  };
  // The page's observer of the sizer is never paused, unlike the frame's own off screen, and a message always arrives,
  // with the width, which a frame off screen may not have been given yet: so a box below the fold has its size before
  // it is scrolled to.
  const boxes = new ResizeObserver(([entry]) => {
    if (entry) child?.box(entry.contentRect.width);
  });
  boxes.observe(sizer);
  const loads = followLoads(iframe, options.url, {
    started: connect,
    other: () => {
      child?.close();
      child = undefined;
      const at = performance.now();
      unasked = [...unasked.filter((then) => at - then < UNASKED_WITHIN), at];
      if (unasked.length < UNASKED_LOADS) {
        loads.load();
        return;
      }
      const failure = { reason: "navigated" } as const;
      if (id !== undefined) observer?.gone(id, failure.reason);
      options.on.failed?.(failure);
    },
  });
  loads.load();
  sizer.append(iframe);
  return {
    iframe,
    update: (next, nextInputs) => {
      node = next;
      inputs = nextInputs;
      bound = boundInputs([next]);
      blocks.rendered([next]);
      // After the render that updates this, once the page has flushed the box's new params.
      queueMicrotask(restyle);
      child?.part({ node: framedNode(node, target), inputs: { ...inputs }, ...inExample });
    },
    code: (next) => {
      code = next;
      child?.code([...next]);
    },
    theme: (next) => {
      tokens = next;
      child?.send({ type: "theme", tokens: { ...next } });
    },
    styles: (next) => {
      styles = next;
      child?.styles([...next]);
    },
    scheme: (next) => {
      scheme = next;
      child?.scheme(next);
      restyle();
    },
    restyle,
    settled: async () => {
      // A frame that never loads holds nothing up for longer than a drawing could.
      let timer: ReturnType<typeof setTimeout> | undefined;
      await Promise.race([first, new Promise((resolve) => (timer = setTimeout(resolve, BUSY_MS)))]);
      clearTimeout(timer);
      await child?.settled();
    },
    destroy: () => {
      loads.stop();
      drawn();
      boxes.disconnect();
      connected();
      child?.close();
      placed.remove();
      if (id !== undefined) observer?.gone(id, "removed");
    },
  };
}
