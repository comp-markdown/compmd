// A wrapped pack's signature at each build: written between its comments from the package's types and the allowlist,
// what changed and what was left out reported, a param that went failing the build while it is still named, and an
// author's own fence left as it is.
import { readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { folder, write } from "../../test/folders.ts";
import { WRAPPED } from "../../test/wrapped.ts";
import type { Wrap } from "./config.ts";
import { generateSignature } from "./generate.ts";
import { generatedPart, opening } from "./generated.ts";

let dir: string;

const wrap: Wrap = { from: "qr", export: "default", kind: "react", params: ["value", "level", "bgColor", "size", "title"], label: "title", box: "width=12rem" };
const old = "```compmd component=qr-code signature\nparams:\n  value: string\n  margin: number\n```";
const documentWith = (generated: string) => `---\nmd:\n  export:\n    pack: "@kit/qr"\n    version: "0.1.0"\n    names: "*"\n---\n\n# QR codes\n\n${generated}\n\nA QR code holds a short text.\n`;

beforeEach(async () => {
  dir = await folder();
  await write(dir, { ...WRAPPED, "demo.comp.md": ":: qr-code[Our site] {value=x}\n" });
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

const documentText = () => readFileSync(join(dir, "index.comp.md"), "utf8");

describe("generateSignature", () => {
  it("writes the allowlist's params between the comments, keeps the prose, and says what changed and was left out", async () => {
    await write(dir, { "index.comp.md": documentWith(generatedPart(opening("qr", "2.1.0"), old)) });
    const { notes, errors } = generateSignature(dir, "index.comp.md", wrap);
    expect(errors).toEqual([]);
    expect(documentText()).toBe(
      documentWith(
        generatedPart(
          opening("qr", "2.2.0"),
          "```compmd component=qr-code signature {width=12rem}\nparams:\n  value:\n    type: string\n    doc: What the code holds\n  level:\n    type: L | M | Q | H\n    default: L\n    doc: How much of it can be lost.\n  bgColor: color?\n  title: string?\n```",
        ),
      ),
    );
    expect(notes).toEqual([
      "`size` isn't a param: a size in pixels: the component fits its box, which the use sizes",
      "`title` takes its string branch, of 2",
      "qr has props that aren't params: `when`, `tags`, `options`, `label`; add any you want to compmd.wrap.json's params",
      'the signature of `qr-code`: `level` is new: L | M | Q | H = "L"; `bgColor` is new: color?; `title` is new: string?; `margin` is gone',
    ]);
  });

  it("fails while the demo or the overrides still name a param the package no longer has, and only says so otherwise", async () => {
    await write(dir, { "index.comp.md": documentWith(generatedPart(opening("qr", "2.2.0"), old)), "demo.comp.md": ":: qr-code[Site] {value=x margin=2}\n" });
    expect(generateSignature(dir, "index.comp.md", { ...wrap, params: ["value", "margin"] }).errors).toEqual(["demo.comp.md still names `margin`, which the package no longer has"]);
    await write(dir, { "demo.comp.md": ":: qr-code[Site] {value=x}\n" });
    const { notes, errors } = generateSignature(dir, "index.comp.md", { ...wrap, params: ["value", "margin"] });
    expect(errors).toEqual([]);
    expect(notes[0]).toBe("`margin` is in compmd.wrap.json's params, but qr has no such prop now");
  });

  it("leaves an author's own fence, saying where it differs from the types, and fails on comments that aren't one pair", async () => {
    await write(dir, { "index.comp.md": documentWith(old) });
    expect(generateSignature(dir, "index.comp.md", wrap).notes).toEqual([
      "index.comp.md has no compmd-pack:generated comments, so its signature is yours: the build leaves it",
      'qr\'s types give another signature of `qr-code` than yours: `level` is new: L | M | Q | H = "L"; `bgColor` is new: color?; `title` is new: string?; `margin` is gone',
    ]);
    expect(documentText()).toBe(documentWith(old));
    expect(generateSignature(dir, "index.comp.md", { ...wrap, from: "nowhere" }).notes[1]).toMatch(/^it can't be compared with nowhere's types: /);
    await write(dir, { "index.comp.md": documentWith(`${opening("qr", "2.2.0")}\n${old}`) });
    expect(generateSignature(dir, "index.comp.md", wrap).errors).toEqual(["index.comp.md: it has 1 opening and 0 closing compmd-pack:generated comments, which should be one pair"]);
  });

  it("says when a required param names a URL, which a strict frame can't fetch", async () => {
    await write(dir, { "index.comp.md": documentWith(generatedPart(opening("viewer", "4.0.0"), old)) });
    const { notes } = generateSignature(dir, "index.comp.md", { from: "viewer", export: "x-viewer", kind: "web-component", params: ["src", "autoRotate"] });
    expect(notes).toContain("`src` names a URL, which a strict frame can't fetch: the pack needs the file in the pack, or a permissive level with its origin");
  });
});
