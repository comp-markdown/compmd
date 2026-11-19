// A blog written into a folder of its own, for the tests: its configuration and its posts.

import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

/** A post's text with `fields` as its frontmatter and `body` after it. */
export const post = (fields: Record<string, unknown>, body = "Text.\n"): string =>
  `---\n${Object.entries(fields)
    .map(([name, value]) => `${name}: ${JSON.stringify(value)}`)
    .join("\n")}\n---\n\n${body}`;

/** The blog the tests build: three posts, one a draft, with tags, code, an image and a component. */
export const BLOG: Record<string, string> = {
  "blog.config.json": JSON.stringify({ title: "Notes & Code", url: "https://example.com/blog/", author: "Ada", description: "Things written down." }),
  "posts/first.comp.md": post({ title: "First", date: "2026-11-01", summary: "The first post.", tags: ["intro"] }, "# Hello\n\nSee ![a photo](./first/photo.svg) and [the next](https://example.com/).\n"),
  "posts/first/photo.svg": '<svg xmlns="http://www.w3.org/2000/svg"/>',
  "posts/second.comp.md": post(
    { title: "Second <post>", date: "2026-11-10", updated: "2026-11-12", summary: "Code and a box.", tags: ["intro", "Code Notes"], author: "Grace" },
    "::: box {padding=m}\nIn a box.\n::: /box\n\n```js\nconst a = 1;\n```\n",
  ),
  "posts/draft.comp.md": post({ title: "Draft", date: "2026-11-20", draft: true }),
};

/**
 * Writes `files` into a new folder in `parent`; its path. A blog whose packs import packages (`svelte`) is written
 * inside this package, where Node finds them.
 */
export async function writeBlog(files: Record<string, string> = BLOG, parent = tmpdir()): Promise<string> {
  await mkdir(parent, { recursive: true });
  const dir = await mkdtemp(join(parent, "compmd-blog-"));
  for (const [path, text] of Object.entries(files)) {
    const file = join(dir, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, text);
  }
  return dir;
}
