// A frame its parent makes, and which of its loads the parent started (docs/running-components-in-the-sandbox.md,
// decision 10): only those are greeted. Any other load is the frame navigating itself, or its element moved in the
// page, which loads it again, and the parent can't tell which, since the frame's location is opaque; what loaded then
// gets nothing.

/** An `iframe` of `document` for a frame document: sandboxed, with scripts and nothing else, no features delegated. */
export function sandboxedFrame(document: Document, title: string): HTMLIFrameElement {
  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts");
  iframe.setAttribute("allow", "");
  iframe.title = title;
  return iframe;
}

export interface FrameLoads {
  /** Loads the frame's document: a load the parent started, which `started` is called for. */
  load(): void;
  /** Stops listening to the frame's loads. */
  stop(): void;
}

/**
 * Follows `iframe`'s loads: `on.started` after each `load()` the parent asked for, `on.other` after any other.
 *
 * @example
 * const loads = followLoads(iframe, url, { started: greet, other: () => loads.load() });
 * loads.load();
 */
export function followLoads(iframe: HTMLIFrameElement, url: string, on: { started(): void; other(): void }): FrameLoads {
  let asked = false;
  const loaded = () => {
    if (asked) {
      asked = false;
      on.started();
    } else {
      on.other();
    }
  };
  iframe.addEventListener("load", loaded);
  return {
    load: () => {
      asked = true;
      iframe.src = url;
    },
    stop: () => {
      iframe.removeEventListener("load", loaded);
    },
  };
}
