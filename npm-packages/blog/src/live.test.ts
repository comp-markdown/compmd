import { rm } from "node:fs/promises";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { buildSite, type Site } from "./build.ts";
import { BLOG, post, writeBlog } from "./test-blog.ts";

/** A post with a host-language component of its own, which the page's script mounts. */
const LIVE = post(
  { title: "Live", date: "2026-11-15", summary: "A component of its own." },
  [
    "```jsx component=hello runtime=react@19",
    "export default function Hello({ name }) {",
    "  return <b>Hello, {name}</b>;",
    "}",
    "```",
    "",
    "```compmd component=hello signature",
    "params:",
    "  name: string",
    "```",
    "",
    ":: hello[Hello, reader] {name=reader}",
    "",
  ].join("\n"),
);

let dir: string;
let site: Site;

beforeAll(async () => {
  dir = await writeBlog({ ...BLOG, "posts/live.comp.md": LIVE });
  site = await buildSite(dir, { drafts: false, now: "2026-11-25T12:00:00Z" });
}, 120_000);

afterAll(async () => {
  await rm(dir, { recursive: true, force: true });
});

/** A blog whose posts use a trusted Svelte pack, as a site's own pack is: the page's script compiles it. */
const SVELTE_PACK: Record<string, string> = {
  "compmd.config.json": JSON.stringify({ packs: { "./kit": { trust: "trusted", ambient: true } } }),
  "kit/package.json": JSON.stringify({ name: "@test/kit", version: "1.0.0", private: true }),
  "kit/index.comp.md": [
    "---",
    "md:",
    "  import:",
    "    - from: ./Glow.svelte",
    "      names: [{ name: Glow, as: glow }]",
    "  export:",
    '    pack: "@test/kit"',
    '    version: "1.0.0"',
    "    names: [glow]",
    "---",
    "",
    "```compmd component=glow signature",
    "params:",
    "  word: string",
    "```",
    "",
  ].join("\n"),
  "kit/Glow.svelte": '<script lang="ts">\n  let { word }: { word: string } = $props();\n</script>\n\n<em class="glow">{word}</em>\n',
  "posts/glowing.comp.md": post({ title: "Glowing", date: "2026-11-16", summary: "A trusted Svelte component." }, ":: glow[Hello] {word=hello}\n"),
};

describe("a post with a trusted Svelte pack's component", () => {
  it("is built with Svelte's compiler into the page's script, and the posts that don't use the ambient pack get none", async () => {
    const svelteBlog = await writeBlog({ ...BLOG, ...SVELTE_PACK }, join(import.meta.dirname, "../test/.out"));
    try {
      const built = await buildSite(svelteBlog, { drafts: false, now: "2026-11-25T12:00:00Z" });
      const page = built.files.get("glowing/index.html") as string;
      expect(page).toContain("Hello");
      const script = /<script type="module" src="\/blog\/([^"]+)"><\/script>/.exec(page)?.[1];
      expect(built.files.has(script!)).toBe(true);
      // Every post can name `glow`, an ambient pack's, but only the one that uses it loads its code (G10).
      expect(built.files.get("first/index.html")).not.toContain('type="module"');
      expect(built.files.get("second/index.html")).not.toContain('type="module"');
    } finally {
      await rm(svelteBlog, { recursive: true, force: true });
    }
  }, 120_000);
});

describe("a post with host-language components", () => {
  it("shows the component's fallback, and gives the page its document, the origins it reaches and the script that mounts it", () => {
    const page = site.files.get("live/index.html") as string;
    expect(page).toContain("Hello, reader");
    const data = /<script type="application\/json" id="cblo-post">(.*?)<\/script>/.exec(page)?.[1];
    // The blog's own origin: its readers are there already, so its images and data need no question.
    expect(JSON.parse(data!)).toEqual({ path: "live.comp.md", files: { "live.comp.md": LIVE }, origins: ["https://example.com"] });
    const script = /<script type="module" src="\/blog\/([^"]+)"><\/script>/.exec(page)?.[1];
    expect(site.files.has(script!)).toBe(true);
  });

  it("brings the sandbox's frames, under compmd/, and gives the other posts no script", () => {
    expect([...site.files.keys()].some((path) => path.startsWith("compmd/frames/"))).toBe(true);
    expect(site.files.get("_headers")?.toString()).toContain("/blog/compmd/frames/*");
    expect(site.files.get("first/index.html")).not.toContain('type="module"');
    expect(site.files.get("index.html")).not.toContain('type="module"');
  });
});
