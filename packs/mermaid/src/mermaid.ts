// `mermaid`: draws a Mermaid diagram from a ```mermaid block, its text the `code` param, or from a leaf whose `code` is
// computed from data (docs/mermaid-pack.md). Mermaid is bundled in, and runs in the host's strict sandbox.
import mermaid from "mermaid";

/** What the host gives a component in the plain binding (CompMD spec §7.4). */
export interface Context<Params> {
  params: Params;
  /** The block's `alt`, or a leaf's fallback: the drawing's text alternative, and what shows when it can't be drawn. */
  fallback: { text: string; element: Element };
}

/** `auto` follows the page's colour scheme; the others are Mermaid's themes. */
export type Theme = "auto" | "default" | "dark" | "neutral" | "forest";

export interface Params {
  /** The diagram's source: the code block's text, or the leaf's param. */
  code: string;
  theme: Theme;
}

/** Mermaid's theme for `theme`: with `auto`, its dark theme when the page is dark. */
export function themeOf(theme: Theme, dark: boolean): Exclude<Theme, "auto"> {
  if (theme !== "auto") return theme;
  return dark ? "dark" : "default";
}

/**
 * Whether the frame shows in a dark scheme: its document's `color-scheme`, which the sandbox sets to its page's, and
 * the reader's preference when it allows both. The reader's preference alone isn't the page's.
 */
export function isDark(preference: { matches: boolean }): boolean {
  const words = (getComputedStyle(document.documentElement).colorScheme || "").split(/\s+/);
  if (!words.includes("dark")) return false;
  return !words.includes("light") || preference.matches;
}

/** The first line of what went wrong: Mermaid's parse errors go on with the source and a caret. */
const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error)).split("\n")[0] ?? "";

/** Every draw in this frame so far, which names each one's elements apart. */
let drawn = 0;

/** The draw before this one: Mermaid's configuration is global, so each draw sets it and renders before the next. */
let queue: Promise<unknown> = Promise.resolve();

/** The SVG of `code` drawn with `theme`; rejected with Mermaid's error when it can't read it. */
function render(code: string, theme: Exclude<Theme, "auto">): Promise<string> {
  drawn += 1;
  const id = `compmd-mermaid-${String(drawn)}`;
  const svg = queue.then(async () => {
    mermaid.initialize({ startOnLoad: false, securityLevel: "strict", suppressErrorRendering: true, theme });
    return (await mermaid.render(id, code)).svg;
  });
  queue = svg.catch(() => undefined);
  return svg;
}

export default function mount(target: HTMLElement, { params, fallback }: Context<Params>): { update(next: Params): void; destroy(): void } {
  const preference = matchMedia("(prefers-color-scheme: dark)");
  let current = params;
  let dark = isDark(preference);
  let latest = 0;
  let destroyed = false;

  const draw = () => {
    latest += 1;
    const draw = latest;
    // Mermaid answers later: a draw an update or a new colour scheme overtook is dropped.
    const shown = () => !destroyed && draw === latest;
    render(current.code, themeOf(current.theme, dark)).then(
      (svg) => {
        if (!shown()) return;
        // Mermaid's own markup, sanitized by it (`securityLevel: "strict"`).
        target.innerHTML = svg;
        const drawing = target.querySelector("svg");
        drawing?.setAttribute("role", "img");
        drawing?.setAttribute("aria-label", fallback.text);
      },
      (error: unknown) => {
        if (!shown()) return;
        const note = document.createElement("p");
        note.textContent = `Mermaid couldn't read this diagram: ${messageOf(error)}`;
        note.style.cssText = "margin: 0.5em 0 0; font-size: 0.875em; opacity: 0.8;";
        target.replaceChildren(fallback.element, note);
      },
    );
  };

  // The page's scheme reaches the frame as its document's `color-scheme`, or through the reader's preference.
  const schemeChanged = () => {
    const now = isDark(preference);
    if (now === dark) return;
    dark = now;
    if (current.theme === "auto") draw();
  };
  const watching = new MutationObserver(schemeChanged);
  watching.observe(document.documentElement, { attributes: true, attributeFilter: ["style", "class"] });
  preference.addEventListener("change", schemeChanged);
  draw();
  return {
    update: (next) => {
      current = next;
      draw();
    },
    destroy: () => {
      destroyed = true;
      watching.disconnect();
      preference.removeEventListener("change", schemeChanged);
      target.replaceChildren();
    },
  };
}
