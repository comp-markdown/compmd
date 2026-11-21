// The generated signature fence in the pack's document (docs/wrapping-an-npm-package.md, decision 4): it stands between
// `<!-- compmd-pack:generated … -->` and `<!-- /compmd-pack:generated -->`, which plain renderers hide; the build writes
// what is between them and leaves the rest of the document as it is. Without the pair, the fence is the author's.

const OPENING = /<!-- compmd-pack:generated\b[^>]*-->/g;
const CLOSING = "<!-- /compmd-pack:generated -->";
const FENCE = /^(`{3,})compmd component=([a-z][a-z0-9]*(?:-[a-z0-9]+)*) signature\b.*\n[\s\S]*?^\1[ \t]*$/gm;

/** The opening comment: what the fence is generated from, and that edits to it are overwritten. */
export const opening = (from: string, version: string): string =>
  `<!-- compmd-pack:generated from ${from} ${version}'s types and compmd.wrap.json; edits between these comments are overwritten at each build -->`;

/** Where the generated fence is, in a document. */
export type Generated =
  /** Between the comments: the component's name, and the text between them. */
  | { kind: "generated"; component: string; start: number; end: number; fence: string }
  /** No comments: the author's own fence, if the document has one for `component`. */
  | { kind: "authors"; fence: string | undefined }
  /** Comments that aren't one pair around one signature fence. */
  | { kind: "broken"; why: string };

/** The signature fences in `text`, each with its component's name. */
function fences(text: string): { component: string; fence: string }[] {
  return [...text.matchAll(FENCE)].map((match) => ({ component: match[2] ?? "", fence: match[0] }));
}

/**
 * Where the generated fence is in `document`; `component` names the author's fence when there are no comments, `""`
 * for the document's first signature fence.
 *
 * @example
 * generatedIn(text, "qr-code"); // { kind: "generated", component: "qr-code", start, end, fence }
 */
export function generatedIn(document: string, component: string): Generated {
  const openings = [...document.matchAll(OPENING)];
  const closings = document.split(CLOSING).length - 1;
  if (openings.length === 0 && closings === 0) return { kind: "authors", fence: fences(document).find((found) => component === "" || found.component === component)?.fence };
  if (openings.length !== 1 || closings !== 1) return { kind: "broken", why: `it has ${String(openings.length)} opening and ${String(closings)} closing compmd-pack:generated comments, which should be one pair` };
  const [open] = openings as [RegExpExecArray];
  const start = open.index + open[0].length;
  const end = document.indexOf(CLOSING, start);
  if (end < 0) return { kind: "broken", why: "its closing compmd-pack:generated comment comes before the opening one" };
  const between = document.slice(start, end);
  const found = fences(between);
  const [only] = found;
  if (found.length !== 1 || only === undefined || between.replace(only.fence, "").trim() !== "") return { kind: "broken", why: "between its compmd-pack:generated comments there should be one signature fence, and nothing else" };
  return { kind: "generated", component: only.component, start: open.index, end: end + CLOSING.length, fence: only.fence };
}

/** `document` with the generated part, from `generated.start` to `generated.end`, written as `fence` under `heading`. */
export function withGenerated(document: string, generated: { start: number; end: number }, heading: string, fence: string): string {
  return `${document.slice(0, generated.start)}${heading}\n${fence}\n${CLOSING}${document.slice(generated.end)}`;
}

/** The generated part of a new pack's document: the comments around `fence`. */
export const generatedPart = (heading: string, fence: string): string => `${heading}\n${fence}\n${CLOSING}`;
