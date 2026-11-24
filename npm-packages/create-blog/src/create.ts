// `npm create @compmd/blog <folder>` (docs/blog-tool.md, decision 3): a new blog's folder, its configuration, a first
// post, and a `package.json` that pins `@compmd/blog` at this package's version, as the two are released together.

import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";

/** What a new blog is given. */
export interface BlogOptions {
  /** The folder it is written into: new, or empty. */
  folder: string;
  /** Its title; the folder's name by default. */
  title?: string;
  /** Who writes it. */
  author: string;
  /** Its address, ending in `/`; a placeholder by default, to change before it is published. */
  url?: string;
  /** The day its first post is dated, `YYYY-MM-DD`. */
  today: string;
}

/** What keeps a blog from being written: a folder that has files. Its message says what to do. */
export class CreateError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CreateError";
  }
}

const MANIFEST = new URL("../package.json", import.meta.url);

/** The version of `@compmd/blog` a new blog pins: this package's own, as they are released together. */
async function version(): Promise<string> {
  return (JSON.parse(await readFile(MANIFEST, "utf8")) as { version: string }).version;
}

/** A folder's own name, read from where it is: `.` is the current folder's name, not `.`. */
const folderName = (folder: string): string => basename(resolve(folder));

/** A folder's name as a blog's title: `my-notes` is "My notes". */
export const titleOf = (folder: string): string => {
  const words = folderName(folder).replace(/[-_]+/g, " ").trim();
  return words === "" ? "My blog" : words.charAt(0).toUpperCase() + words.slice(1);
};

/** A folder's name as an npm package's: lowercase, no characters npm refuses, not starting with `.` or `_`. */
export const packageNameOf = (folder: string): string =>
  folderName(folder)
    .toLowerCase()
    .replace(/[^a-z0-9._~-]+/g, "-")
    .replace(/^[._]+/, "") || "blog";

/** A new blog's files, by their paths in its folder. */
export async function blogFiles({ folder, title = titleOf(folder), author, url = "https://example.com/", today }: BlogOptions): Promise<Record<string, string>> {
  const name = packageNameOf(folder);
  return {
    "package.json": `${JSON.stringify({ name, private: true, type: "module", scripts: { dev: "cblo dev", build: "cblo build", new: "cblo new" }, dependencies: { "@compmd/blog": `^${await version()}` } }, null, 2)}\n`,
    "blog.config.json": `${JSON.stringify({ title, url, author, lang: "en", description: `${title}, written in CompMD.` }, null, 2)}\n`,
    "posts/hello.comp.md": `---\ntitle: Hello\ndate: ${today}\nsummary: The first post, and what this blog is for.\ntags: [intro]\n---\n\nThis blog is written in [CompMD](https://compmd.dev/): Markdown, with components.\n\n::: row {gap=m}\n::: box {padding=m border=subtle border-radius=m}\n**Write** a post with \`npm run new -- "Its title"\`.\n::: /box\n::: box {padding=m border=subtle border-radius=m}\n**See** it with \`npm run dev\`, and publish \`dist/\` after \`npm run build\`.\n::: /box\n::: /row\n`,
    ".gitignore": "node_modules/\ndist/\n",
    "README.md": `# ${title}\n\nA blog in CompMD, made with [@compmd/blog](https://www.npmjs.com/package/@compmd/blog).\n\n\`\`\`sh\nnpm install\nnpm run dev                    # the blog at http://localhost:4321/, drafts included\nnpm run new -- "A new post"   # posts/a-new-post.comp.md, a draft\nnpm run build                  # the site, in dist/\n\`\`\`\n\nSet the blog's address (\`url\`) in \`blog.config.json\` before it is published.\n`,
  };
}

/**
 * Writes a new blog into `options.folder`.
 *
 * @returns The files written, by their paths in the folder, sorted.
 * @throws CreateError when the folder has files in it.
 */
export async function createBlog(options: BlogOptions): Promise<string[]> {
  if (existsSync(options.folder) && (await readdir(options.folder)).length > 0) throw new CreateError(`${options.folder} has files in it: choose a new folder, or an empty one`);
  const files = await blogFiles(options);
  for (const [path, text] of Object.entries(files)) {
    const file = join(options.folder, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, text);
  }
  return Object.keys(files).sort();
}
