import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { buildSite } from "@compmd/blog";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import manifest from "../package.json" with { type: "json" };
import { blogFiles, createBlog, CreateError, packageNameOf, titleOf } from "./create.ts";
import { localDay } from "./local-day.ts";

let root: string;

beforeEach(async () => {
  root = await mkdtemp(join(tmpdir(), "compmd-create-blog-"));
});

afterEach(async () => {
  await rm(root, { recursive: true, force: true });
});

describe("createBlog", () => {
  it("writes a blog @compmd/blog builds, pinned to this package's version", async () => {
    const folder = join(root, "my-notes");
    expect(await createBlog({ folder, author: "Ada", url: "https://example.com/notes/", today: "2026-11-25" })).toEqual([".gitignore", "README.md", "blog.config.json", "package.json", "posts/hello.comp.md"]);
    const pkg = JSON.parse(await readFile(join(folder, "package.json"), "utf8")) as { name: string; dependencies: Record<string, string>; scripts: Record<string, string> };
    expect(pkg).toMatchObject({ name: "my-notes", dependencies: { "@compmd/blog": `^${manifest.version}` }, scripts: { dev: "cblo dev", build: "cblo build", new: "cblo new" } });
    expect(JSON.parse(await readFile(join(folder, "blog.config.json"), "utf8"))).toMatchObject({ title: "My notes", author: "Ada", url: "https://example.com/notes/" });
    const site = await buildSite(folder, { drafts: false, now: "2026-11-25T00:00:00Z" });
    expect(site.warnings).toEqual([]);
    const post = site.files.get("hello/index.html") as string;
    // The page shows the title as its heading, and the post starts with its first paragraph: one heading, not two.
    expect(post.match(/<h1[ >]/g)).toEqual(["<h1>"]);
    expect(post).toContain("<h1>Hello</h1>");
  });

  it("refuses a folder with files in it", async () => {
    await writeFile(join(root, "kept.txt"), "");
    await expect(createBlog({ folder: root, author: "Ada", today: "2026-11-25" })).rejects.toThrow(new CreateError(`${root} has files in it: choose a new folder, or an empty one`));
  });
});

describe("titleOf and blogFiles", () => {
  it("make a title of the folder's name, and a placeholder address", async () => {
    expect(titleOf("/x/my_tech-notes")).toBe("My tech notes");
    expect(titleOf("/")).toBe("My blog");
    expect(JSON.parse((await blogFiles({ folder: "/x/b", author: "A", today: "2026-11-25" }))["blog.config.json"]!)).toMatchObject({ title: "B", url: "https://example.com/" });
  });

  it("name `.` after the folder it is, and give a package name npm takes", () => {
    const here = basename(process.cwd());
    expect(titleOf(".")).toBe(titleOf(here));
    expect(packageNameOf(".")).toBe(packageNameOf(here));
    expect(packageNameOf("/x/My Notes")).toBe("my-notes");
    expect(packageNameOf("/x/.notes")).toBe("notes");
    expect(packageNameOf("/x/...")).toBe("blog");
  });
});

describe("localDay", () => {
  it("gives the day of this machine's calendar, late in the evening and early in the morning", () => {
    expect(localDay(new Date(2026, 10, 23, 23, 30))).toBe("2026-11-23");
    expect(localDay(new Date(2026, 0, 1, 0, 15))).toBe("2026-01-01");
  });
});
