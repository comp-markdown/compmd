// `__component__`: draws a ```__language__ block, its text in `code`, in the page's colour scheme. Replace the drawing
// with yours, a library's rendering of the language, say: what it imports is bundled in. A library that draws
// asynchronously is simpler with `draw` from `@compmd/pack/component`.
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  /** The code block's text. */
  code: string;
}

export default component<Params>((target, context) => {
  const figure = document.createElement("figure");
  figure.setAttribute("role", "img");
  // The block's `alt`, or a leaf's fallback: the text alternative of what is drawn.
  figure.setAttribute("aria-label", context.fallback.text);
  figure.style.cssText = "margin: 0; padding: 1em; border-radius: 8px; font: 14px/1.5 ui-monospace, monospace;";
  target.append(figure);
  const draw = ({ code }: Params) => {
    // The scheme is the page's, which may not be the reader's preference; a change of it comes as an update.
    const dark = context.scheme === "dark";
    figure.style.background = dark ? "#1f2328" : "#f6f8fa";
    figure.style.color = dark ? "#f6f8fa" : "#1f2328";
    figure.replaceChildren(
      ...code.split("\n").map((line, i) => {
        const row = document.createElement("div");
        row.textContent = `${String(i + 1).padStart(2)}  ${line}`;
        return row;
      }),
    );
  };
  draw(context.params);
  return {
    update: draw,
    destroy: () => {
      figure.remove();
    },
  };
});
