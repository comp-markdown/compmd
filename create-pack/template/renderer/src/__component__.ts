// `__component__`: draws a ```__language__ block, its text in `code`. Replace the drawing with yours, a library's
// rendering of the language, say: what it imports is bundled in.

/** What the host gives a component in the plain binding (CompMD spec §7.4). */
export interface Context<Params> {
  params: Params;
  /** The block's `alt`, or a leaf's fallback: use its text as the text alternative of what you draw. */
  fallback: { text: string; element: Element };
}

export interface Params {
  /** The code block's text. */
  code: string;
  theme: "light" | "dark";
}

export default function mount(target: HTMLElement, { params, fallback }: Context<Params>): { update(next: Params): void; destroy(): void } {
  const figure = document.createElement("figure");
  figure.setAttribute("role", "img");
  figure.setAttribute("aria-label", fallback.text);
  figure.style.cssText = "margin: 0; padding: 1em; border-radius: 8px; font: 14px/1.5 ui-monospace, monospace;";
  target.append(figure);
  const draw = ({ code, theme }: Params) => {
    figure.style.background = theme === "dark" ? "#1f2328" : "#f6f8fa";
    figure.style.color = theme === "dark" ? "#f6f8fa" : "#1f2328";
    figure.replaceChildren(
      ...code.split("\n").map((line, i) => {
        const row = document.createElement("div");
        row.textContent = `${String(i + 1).padStart(2)}  ${line}`;
        return row;
      }),
    );
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      figure.remove();
    },
  };
}
