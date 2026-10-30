// One use in a frame of its own: the frame document of its level, the protocol's parent end on the port the parent
// gives it, and the frame's size set from what it reports. The host's page frames strict and permissive uses this way,
// and a permissive frame its strict ones (docs/sandbox.md, decision 1).

import type { ExpandedComponent, ExpandedInput } from "@compmd/render";
import { networkPolicy } from "./frame/narrow.ts";
import { boundInputs } from "./protocol/inputs.ts";
import { HELLO, SANDBOX_VERSION, type ComponentCode, type Level } from "./protocol/messages.ts";
import { connectChild, type ChildEnd, type ChildEvents } from "./protocol/parent-end.ts";

export interface UseFrameOptions {
  /** The frame document of the use's level: `strict.html` or `permissive.html`. */
  url: string;
  level: Level;
  /** A permissive use's origins. */
  network?: readonly string[];
  /** A port for `host`, for a permissive use of a pack given `host`. */
  host?: () => MessagePort;
  code: readonly ComponentCode[];
  node: ExpandedComponent;
  inputs: Readonly<Record<string, ExpandedInput>>;
  styles?: readonly string[];
  ping?: { every: number; timeout: number };
  on: { [T in keyof ChildEvents]?: (event: ChildEvents[T]) => void };
}

export interface UseFrame {
  readonly iframe: HTMLIFrameElement;
  update(node: ExpandedComponent, inputs: Readonly<Record<string, ExpandedInput>>): void;
  code(code: readonly ComponentCode[]): void;
  destroy(): void;
}

/**
 * Frames `options.node` in `target`. The frame connects each time its document loads, as when an editor builds its
 * widget again and the frame loads again, and gets its level, code and part each time.
 *
 * @example
 * const frame = frameUse(box, { url: strictUrl, level: "strict", code, node, inputs, on: { set: ({ input, value }) => setInput(input, value) } });
 * frame.update(next, inputs);
 */
export function frameUse(target: Element, options: UseFrameOptions): UseFrame {
  const document = target.ownerDocument;
  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts");
  iframe.setAttribute("allow", "");
  iframe.title = options.node.name;
  iframe.className = "compmd-frame";
  iframe.style.cssText = `border: 0; width: 100%; height: 0; display: ${options.node.type === "componentInline" ? "inline-block" : "block"};`;
  let node = options.node;
  let inputs = options.inputs;
  let bound = boundInputs([node]);
  let code = options.code;
  let child: ChildEnd | undefined;
  const connect = () => {
    child?.close();
    const { port1, port2 } = new MessageChannel();
    iframe.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]);
    const end = connectChild(port1, {
      level: options.level,
      ...(options.network ? { policy: networkPolicy(options.network) } : {}),
      binds: (input) => bound.has(input),
      ...(options.host ? { host: options.host() } : {}),
      ...(options.ping ? { ping: options.ping } : {}),
    });
    end.on("size", ({ width, height }) => {
      iframe.style.height = `${String(height)}px`;
      if (node.type === "componentInline") iframe.style.width = `${String(width)}px`;
    });
    for (const [type, listener] of Object.entries(options.on)) end.on(type as keyof ChildEvents, listener as (event: unknown) => void);
    if (options.styles?.length) end.styles([...options.styles]);
    end.code([...code]);
    end.part({ node, inputs: { ...inputs } });
    child = end;
  };
  iframe.addEventListener("load", connect);
  iframe.src = options.url;
  target.append(iframe);
  return {
    iframe,
    update: (next, nextInputs) => {
      node = next;
      inputs = nextInputs;
      bound = boundInputs([next]);
      child?.part({ node, inputs: { ...inputs } });
    },
    code: (next) => {
      code = next;
      child?.code([...next]);
    },
    destroy: () => {
      iframe.removeEventListener("load", connect);
      child?.close();
      iframe.remove();
    },
  };
}
