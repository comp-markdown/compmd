// `cblo new` (docs/blog-tool.md, decision 3): a new post, its frontmatter filled in, so writing starts at once.

import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { BlogError, readConfig } from "./config.ts";
import { EXTENSION, POSTS } from "./posts.ts";
import { slugOf } from "./slug.ts";

/** A YAML string, quoted as JSON is, which YAML reads the same. */
const quoted = (text: string): string => JSON.stringify(text);

/**
 * A new post's text: its frontmatter (a draft, dated `date`), then an empty line where the post starts. No heading:
 * the post's page shows its title as its heading.
 */
export function postText(title: string, date: string, author: string): string {
  return `---\ntitle: ${quoted(title)}\ndate: ${date}\nauthor: ${quoted(author)}\nsummary: ""\ntags: []\ndraft: true\n---\n\n`;
}

/**
 * Writes a new post titled `title` into the blog in `dir`, dated `date`, as `posts/<slug>.comp.md`: the slug from the
 * title unless `slug` is given. Its path in the blog.
 *
 * @throws BlogError when the title gives no slug, or a post has it already.
 */
export async function newPost(dir: string, title: string, { slug = slugOf(title), date }: { slug?: string; date: string }): Promise<string> {
  const config = await readConfig(dir);
  if (slug === "") throw new BlogError(`"${title}" gives no slug: name one with --slug`);
  if (slugOf(slug) !== slug) throw new BlogError(`the slug "${slug}" is lowercase letters and digits, joined by -: "${slugOf(slug)}"`);
  const path = join(POSTS, `${slug}${EXTENSION}`);
  if (existsSync(join(dir, path))) throw new BlogError(`${path} is there already`);
  await mkdir(join(dir, POSTS), { recursive: true });
  await writeFile(join(dir, path), postText(title, date, config.author));
  return path;
}
