// What a strict frame refuses, found in a built module (docs/wrapping-an-npm-package.md, decision 9; spec §15): a URL of
// another origin it may fetch, a worker, code made from text, and WebGL. Each is a warning naming the module and the
// text it found; the pack then needs its assets in the pack, a permissive level with the origins it reaches, or
// another package.

/** What a strict frame refuses, and the text that finds it. */
const RISKS: readonly { what: string; found: RegExp }[] = [
  // Namespaces and specifications are names, never fetched.
  { what: "a URL of another origin, which a strict frame can't fetch", found: /\bhttps?:\/\/(?!www\.w3\.org\/|www\.w3c\.org\/|purl\.org\/|ns\.adobe\.com\/|schemas\.)[\w.-]+\.[a-z]{2,}[^\s"'`)]*/gi },
  { what: "a worker, which a strict frame can't start", found: /\bnew\s+(?:Shared)?Worker\s*\(/g },
  { what: "code made from text, which a strict frame refuses", found: /\b(?:eval\s*\(|new\s+Function\s*\()/g },
  { what: "WebGL, which a strict frame has no context for", found: /getContext\(\s*["'`](?:webgl2?|experimental-webgl)["'`]/g },
];

/** How much of what it found a warning quotes. */
const QUOTED = 80;

/**
 * The warnings for the built module `code`, written at `file`: each kind once, with the first text that found it.
 *
 * @example
 * risksOf("dist/viewer.js", code); // ["dist/viewer.js has a worker, which a strict frame can't start: `new Worker(`"]
 */
export function risksOf(file: string, code: string): string[] {
  return RISKS.flatMap(({ what, found }) => {
    const first = [...code.matchAll(found)][0]?.[0];
    return first === undefined ? [] : [`${file} has ${what}: \`${first.slice(0, QUOTED)}\``];
  });
}
