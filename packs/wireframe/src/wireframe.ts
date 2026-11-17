// `wireframe`: draws a ```wireframe block, its text in `code`, as HTML and CSS (LANGUAGE.md), in the page's colour
// scheme. A source that isn't valid draws nothing and fails with its first error, which the host reports (W118), and
// the reader sees the code.
import { draw } from "@compmd/pack/component";
import { draw as drawWireframe } from "./draw.ts";
import { parse } from "./parse.ts";
import { STYLE } from "./style.ts";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  /** The block's text, or the leaf's param. */
  code: string;
}

/** What a source's errors say: the first, with how many more there are. */
export function failureOf(errors: readonly Error[]): string {
  const [first] = errors;
  const more = errors.length - 1;
  return `${first?.message ?? "nothing to draw"}${more > 0 ? ` (and ${String(more)} more)` : ""}`;
}

// The stylesheet goes with each drawing: a host may show each use in a frame of its own.
export default draw<Params>(
  ({ code }, { scheme }) => {
    const { wireframe, errors } = parse(code);
    if (!wireframe) throw new Error(failureOf(errors));
    const drawing = drawWireframe(wireframe, document);
    drawing.dataset.scheme = scheme;
    const style = document.createElement("style");
    style.textContent = STYLE;
    const holder = document.createElement("div");
    holder.append(style, drawing);
    return holder;
  },
  { role: "img" },
);
