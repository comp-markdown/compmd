// Code highlighting and math for a host-mode page, from the frames' own files (`highlight.js` and `math.js` where the
// host serves them): the app's bundle then has no Shiki and no Temml of its own, and the page and its frames load one
// copy, which the browser caches once.

import type { CodeHighlighter, MathRenderer } from "@compmd/render";
import { frameUrl } from "./host/runtime.ts";

/** A module of the frames' files, imported by its URL: the app's bundler leaves it alone. */
const imported = <T>(url: string): Promise<T> => import(/* @vite-ignore */ url) as Promise<T>;

/**
 * The frames' highlighter (`shikiHighlight`), loaded from `base` the first time a page has a code block. Until then
 * a block shows its text, as it does when its language isn't highlighted.
 */
export function framesHighlight(base: string): CodeHighlighter {
  let loaded: Promise<CodeHighlighter> | undefined;
  return (code, language, target) => {
    loaded ??= imported<{ shikiHighlight: () => CodeHighlighter }>(frameUrl(base, "highlight.js")).then((module) => module.shikiHighlight());
    void loaded.then(
      (highlight) => {
        highlight(code, language, target);
      },
      () => undefined,
    );
  };
}

/** The frames' math (`temmlMath`), loaded from `base` the first time a page has math; until then, its TeX. */
export function framesMath(base: string): MathRenderer {
  let loaded: Promise<MathRenderer> | undefined;
  return (tex, options, target) => {
    target.textContent = tex;
    loaded ??= imported<{ temmlMath: () => MathRenderer }>(frameUrl(base, "math.js")).then((module) => module.temmlMath());
    void loaded.then(
      (math) => {
        math(tex, options, target);
      },
      () => undefined,
    );
  };
}
