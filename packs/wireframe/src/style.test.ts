import { describe, expect, it } from "vitest";
import { DRAW } from "./draw.ts";
import { STYLE } from "./style.ts";
import { drawn, FILES, ORDER, SETTINGS } from "./testing.ts";

describe("the style", () => {
  it("styles every class the drawing makes, but those that only name a part", () => {
    const roots = [SETTINGS, FILES, ORDER, 'window:\n  chip "x"'].map(drawn);
    const classes = new Set(roots.flatMap((root) => [root, ...root.querySelectorAll("*")].flatMap((found) => [...found.classList])));
    const namesAPart = (name: string) => name.slice("wf-".length) in DRAW;
    const missing = [...classes].filter((name) => !namesAPart(name) && !new RegExp(`\\.${name}(?![\\w-])`).test(STYLE));
    expect(missing).toEqual([]);
  });

  it("takes its greys from the host's tokens, with fallbacks from the text colour", () => {
    expect(STYLE).toContain("var(--compmd-border-strong, 1px solid color-mix(in srgb, currentColor 45%, transparent))");
    expect(STYLE).toContain("var(--compmd-surface-subtle, color-mix(in srgb, currentColor 6%, transparent))");
  });

  it("follows the page's scheme, keeps a drawing whole on paper, and takes a page's hand-written font", () => {
    expect(STYLE).toContain('[data-scheme="dark"] { color-scheme: dark; }');
    expect(STYLE).toMatch(/\.wf-wireframe \{[^}]*break-inside: avoid/);
    expect(STYLE).toContain("var(--compmd-wireframe-font, ui-sans-serif, system-ui, sans-serif)");
  });
});
