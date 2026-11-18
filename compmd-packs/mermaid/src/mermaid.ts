// `mermaid`: draws a Mermaid diagram from a ```mermaid block, its text the `code` param, or from a leaf whose `code` is
// computed from data (docs/mermaid-pack.md). Mermaid is bundled in, and runs in the host's strict sandbox.
import mermaid from "mermaid";
import { draw, type Scheme } from "@compmd/pack/component";

/** `auto` follows the page's colour scheme; the others are Mermaid's themes. */
export type Theme = "auto" | "default" | "dark" | "neutral" | "forest";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  /** The diagram's source: the code block's text, or the leaf's param. */
  code: string;
  theme: Theme;
}

/** Mermaid's theme for `theme`: with `auto`, its dark theme when the page is dark. */
export function themeOf(theme: Theme, scheme: Scheme): Exclude<Theme, "auto"> {
  if (theme !== "auto") return theme;
  return scheme === "dark" ? "dark" : "default";
}

/** The first line of what went wrong: Mermaid's parse errors go on with the source and a caret. */
const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error)).split("\n")[0] ?? "";

/** Every draw in this frame so far, which names each one's elements apart. */
let drawn = 0;

// Mermaid's configuration is global, so draws go one at a time; a draw that can't read its code fails with Mermaid's
// message, which the host reports, and the reader sees the code.
export default draw<Params>(
  async ({ code, theme }, { scheme }) => {
    drawn += 1;
    mermaid.initialize({ startOnLoad: false, securityLevel: "strict", suppressErrorRendering: true, theme: themeOf(theme, scheme) });
    try {
      // Mermaid's own markup, sanitized by it (`securityLevel: "strict"`).
      return (await mermaid.render(`compmd-mermaid-${String(drawn)}`, code)).svg;
    } catch (error) {
      throw new Error(`Mermaid couldn't read this diagram: ${messageOf(error)}`);
    }
  },
  { role: "img", oneAtATime: true },
);
