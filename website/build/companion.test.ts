import { describe, expect, it } from "vitest";
import { betweenThem, renderSpecPage } from "./companion.ts";

const BLOB = "https://github.com/comp-markdown/compmd/blob/main/";

/** A spec and a companion that link to each other, as the real ones do. */
const FILES: Record<string, string> = {
  "compmd-spec-v1.md": [
    "# CompMD specification",
    "",
    "## 1. Introduction",
    "",
    "What a pack on a registry is: [*CompMD packs on a registry*](compmd-registry-v1.md), and [who fetches](compmd-registry-v1.md#4-who-fetches).",
    "",
    "## 6. Imports and exports",
    "",
    "### 6.4 Packages and the registry",
    "",
    "A processor MAY cap a pack's size. [The changelog](CHANGELOG.md).",
  ].join("\n"),
  "compmd-registry-v1.md": [
    "# CompMD packs on a registry, version 1 (draft)",
    "",
    "A companion to the [CompMD specification](compmd-spec-v1.md), for [§6.4](compmd-spec-v1.md#64-packages-and-the-registry).",
    "",
    "## 1. Conventions",
    "",
    "The host MUST check the tarball; [who fetches](#4-who-fetches) says more.",
    "",
    "## 4. Who fetches",
    "",
    "### By default",
  ].join("\n"),
};

describe("betweenThem", () => {
  it("points a link between the spec and its companion within the page, and leaves the rest", () => {
    expect(betweenThem("compmd-registry-v1.md")).toBe("#registry");
    expect(betweenThem("./compmd-registry-v1.md#4-who-fetches")).toBe("#registry-4-who-fetches");
    expect(betweenThem("compmd-spec-v1.md")).toBe("#");
    expect(betweenThem("compmd-spec-v1.md#64-packages-and-the-registry")).toBe("#64-packages-and-the-registry");
    expect(betweenThem("CHANGELOG.md")).toBeUndefined();
    expect(betweenThem("https://www.npmjs.com/")).toBeUndefined();
  });
});

describe("renderSpecPage", () => {
  it("puts the companion after the spec, a section of its own, its headings a level lower and its ids under the section's", async () => {
    const { html, toc } = await renderSpecPage((path) => FILES[path], BLOB);
    expect(toc.map(({ level, id }) => [level, id])).toEqual([
      [2, "1-introduction"],
      [2, "6-imports-and-exports"],
      [3, "64-packages-and-the-registry"],
      [2, "registry"],
      [3, "registry-1-conventions"],
      [3, "registry-4-who-fetches"],
    ]);
    expect(html).toContain('<section class="companion" aria-labelledby="registry"><h2 id="registry">CompMD packs on a registry, version 1 (draft)');
    expect(html).toContain('<h4 id="registry-by-default">');
    // Each of the companion's headings links to its own file.
    expect(html).toContain(`href="${BLOB}compmd-registry-v1.md#L9"`);
    expect(html).toContain('<em class="rule">MUST</em>');
  });

  it("links the spec and its companion to each other within the page, and keeps other links on GitHub", async () => {
    const { html } = await renderSpecPage((path) => FILES[path], BLOB);
    expect(html).toContain('href="#registry"><em>CompMD packs on a registry</em></a>');
    expect(html).toContain('href="#registry-4-who-fetches">who fetches</a>');
    expect(html).toContain('href="#">CompMD specification</a>');
    expect(html).toContain('href="#64-packages-and-the-registry">§6.4</a>');
    expect(html).toContain(`href="${BLOB}CHANGELOG.md"`);
  });

  it("is the spec alone for a version that has no companion", async () => {
    const { html, toc } = await renderSpecPage((path) => (path === "compmd-spec-v1.md" ? FILES[path] : undefined), BLOB);
    expect(html).not.toContain("companion");
    expect(toc.map(({ id }) => id)).toEqual(["1-introduction", "6-imports-and-exports", "64-packages-and-the-registry"]);
  });
});
