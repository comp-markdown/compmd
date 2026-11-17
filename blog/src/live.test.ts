import { rm } from "node:fs/promises";
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

describe("a post with host-language components", () => {
  it("shows the component's fallback, and gives the page its document and the script that mounts it", () => {
    const page = site.files.get("live/index.html") as string;
    expect(page).toContain("Hello, reader");
    const data = /<script type="application\/json" id="cblo-post">(.*?)<\/script>/.exec(page)?.[1];
    expect(JSON.parse(data!)).toEqual({ path: "live.comp.md", files: { "live.comp.md": LIVE } });
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
