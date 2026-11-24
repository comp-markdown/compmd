import { describe, expect, it } from "vitest";
import { scanHtml, type Scanned } from "./html-scanner.ts";

/** The regular expressions the scanner replaces, kept as what it must agree with. */
const TAG = /<!--[\s\S]*?-->|<(\/?)([a-zA-Z][a-zA-Z0-9-]*)((?:\s+[^\s"'>/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?)*)\s*\/?>/g;
const ATTRIBUTE = /([^\s"'>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;

/** `html` read by the regular expressions, as the renderer read it before. */
function byRegex(html: string): Scanned[] {
  const scanned: Scanned[] = [];
  let at = 0;
  for (const match of html.matchAll(TAG)) {
    if (match.index > at) scanned.push({ kind: "text", value: html.slice(at, match.index) });
    at = match.index + match[0].length;
    const tag = match[2];
    if (tag === undefined) {
      scanned.push({ kind: "comment" });
      continue;
    }
    const attributes = [...(match[3] ?? "").matchAll(ATTRIBUTE)].map((attribute) => ({
      name: (attribute[1] ?? "").toLowerCase(),
      value: attribute[2] ?? attribute[3] ?? attribute[4] ?? "",
    }));
    scanned.push({ kind: "tag", closing: match[1] === "/", tag: tag.toLowerCase(), attributes, source: match[0] });
  }
  if (at < html.length) scanned.push({ kind: "text", value: html.slice(at) });
  return scanned;
}

/** A seeded generator, so a failing case is the same on every run. */
function random(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1_103_515_245 + 12_345) % 2_147_483_648;
    return state / 2_147_483_648;
  };
}

/** The pieces HTML is made of, the troublesome ones among them. */
const PIECES = ["<", ">", "/", "=", '"', "'", "`", " ", "\n", "!", "-", "<!--", "-->", "a", "B", "kbd", "details", "href", "x1", "_", "<a", "</a>", '="v"'];

describe("scanHtml", () => {
  it("reads tags, attributes, comments and text as the regular expressions did, on many random inputs", () => {
    const next = random(7);
    for (let run = 0; run < 5_000; run += 1) {
      const length = Math.floor(next() * 24);
      const html = Array.from({ length }, () => PIECES[Math.floor(next() * PIECES.length)]).join("");
      expect(scanHtml(html), JSON.stringify(html)).toEqual(byRegex(html));
    }
  });

  it("reads the cases the renderer meets", () => {
    for (const html of ['<a href="https://x.dev" onclick=evil()>x</a>', "<details open>\n<summary>More</summary>", "<br/><img src='a.png' alt=A />", "<!-- a > b --><kbd>", "a < b > c", "<a\nhref = 'x'\n>", "</sub >", "<3"]) {
      expect(scanHtml(html), html).toEqual(byRegex(html));
    }
  });

  it("takes linear time on a run of tags that never closes, and on a comment that never ends", () => {
    for (const html of ["<details " + "<a ".repeat(34_000), "<!--<a ".repeat(15_000), '<a x="'.repeat(17_000)]) {
      expect(html.length).toBeGreaterThan(100_000);
      const started = performance.now();
      const scanned = scanHtml(html);
      expect(performance.now() - started, "a quadratic scan of 100 KB takes many seconds").toBeLessThan(1_000);
      expect(scanned.map((part) => part.kind)).toEqual(["text"]);
    }
  });
});
