// Math typeset as MathML, which the browser lays out: Temml turns the TeX into it. Temml loads the first time a page
// has math, so until then, and when the TeX doesn't typeset, the math shows as its source.

import type { MathRenderer } from "@compmd/render";

/** What the frame needs of Temml. */
export interface Typesetter {
  render(tex: string, target: HTMLElement, options: { displayMode: boolean; throwOnError: boolean }): void;
}

/** The TeX each element is to show last, so a typesetting that finishes after a newer one is dropped. */
const latest = new WeakMap<HTMLElement, string>();

/**
 * A math renderer for the page, typesetting with what `load` gives, loaded once.
 *
 * @example
 * serveFrame(port, { container, math: typesetMath(async () => (await import("temml")).default) });
 */
export function typesetMath(load: () => Promise<Typesetter>): MathRenderer {
  let typesetter: Promise<Typesetter> | undefined;
  return (tex, { display }, target) => {
    latest.set(target, tex);
    target.textContent = tex;
    typesetter ??= load();
    void typesetter.then(
      (temml) => {
        if (latest.get(target) !== tex) return;
        target.replaceChildren();
        try {
          temml.render(tex, target, { displayMode: display, throwOnError: true });
        } catch {
          target.textContent = tex;
        }
      },
      () => undefined,
    );
  };
}
