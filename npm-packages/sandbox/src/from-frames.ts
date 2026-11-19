// Code highlighting and math for a host-mode page, from the frames' own files (`highlight.js` and `math.js` where the
// host serves them): the app's bundle then has no Shiki and no Temml of its own, and the page and its frames load one
// copy, which the browser caches once.

import type { CodeHighlighter, MathRenderer } from "@compmd/render";
import { frameUrl } from "./host/runtime.ts";

/** A module of the frames' files, imported by its URL: the app's bundler leaves it alone. */
const imported = <T>(url: string): Promise<T> => import(/* @vite-ignore */ url) as Promise<T>;

/** Makes the highlighter of the frames' `highlight.js` at `url`. */
export type LoadHighlighter = (url: string) => Promise<CodeHighlighter>;

const loadFramesHighlighter: LoadHighlighter = (url) => imported<{ shikiHighlight: () => CodeHighlighter }>(url).then((module) => module.shikiHighlight());

/**
 * The page's highlighters, by the URL of the `highlight.js` they come from: one for every view of the page, so Shiki
 * is made once, and a grammar loads once for all of them, where a Shiki a view made each paid for it again.
 */
const shared = new Map<string, Promise<CodeHighlighter>>();

/**
 * The frames' highlighter (`shikiHighlight`), loaded from `base` the first time a page has a code block, and shared by
 * every view of the page that loads it from there. Until then a block shows its text, as it does when its language
 * isn't highlighted, or when the file can't be had; a view after a failed load tries again.
 */
export function framesHighlight(base: string, load: LoadHighlighter = loadFramesHighlighter): CodeHighlighter {
  const url = frameUrl(base, "highlight.js");
  return async (code, language) => {
    let highlighter = shared.get(url);
    if (!highlighter) {
      highlighter = load(url);
      shared.set(url, highlighter);
      highlighter.catch(() => shared.delete(url));
    }
    return (await highlighter)(code, language);
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
