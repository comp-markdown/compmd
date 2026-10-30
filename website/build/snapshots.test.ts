// @vitest-environment node
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { snapshot, snapshots } from "./snapshots.ts";

const NOW = "2026-10-30T09:00:00Z";
const box = "::: box {padding=m}\nHello\n::: /box\n";

describe("snapshot", () => {
  it("renders the page, its stylesheet and the plain view, with the block classes prefixed", () => {
    const shown = snapshot("a.comp.md", { "a.comp.md": box }, NOW, "hello");
    expect(shown.text).toBe(box);
    expect(shown.html).toBe('<div class="compmd-box compmd-hello-b0"><p>Hello</p></div>');
    expect(shown.css).toContain(".compmd-hello-b0 { padding: var(--compmd-space-m, 1rem); }");
    expect(shown.css).not.toMatch(/compmd-b\d/);
    expect(shown.plain).toContain("::: box {padding=m}");
  });

  it("reads the other files it is given", () => {
    const text = "---\nmd:\n  data:\n    - {from: ./n.json, as: n}\n---\n\n{{md.data.n.count}}\n";
    expect(snapshot("a.comp.md", { "a.comp.md": text, "n.json": '{"count": 3}' }, NOW, "a").html).toBe("<p>3</p>");
  });

  it("gives inputs their values", () => {
    const text = "---\nmd:\n  inputs:\n    n: number = 3\n    on: boolean = true\n---\n\n:: number-input[N] {bind=n}\n\n:: toggle-input[On] {bind=on}\n";
    const { html } = snapshot("a.comp.md", { "a.comp.md": text }, NOW, "a");
    expect(html).toContain('<input type="number" value="3">');
    expect(html).toContain('<input type="checkbox" checked="">');
  });

  it("refuses a document with a diagnostic, saying where", () => {
    expect(() => snapshot("a.comp.md", { "a.comp.md": "::: box\nopen\n" }, NOW, "a")).toThrow(/^a\.comp\.md: F\d+ at 1:1: /);
  });
});

describe("snapshots", () => {
  it("renders every document of the site's content, each by its name", () => {
    const all = snapshots(join(import.meta.dirname, "../content"), NOW);
    expect(Object.keys(all)).toContain("hero");
    for (const [name, shown] of Object.entries(all)) expect(shown.css, name).not.toMatch(/compmd-b\d/);
  });
});
