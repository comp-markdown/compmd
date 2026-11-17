#!/usr/bin/env node
// `cblo` (also `compmd-blog`), in the blog's folder (docs/blog-tool.md, decision 3):
//
//   cblo new "Why CompMD" [--slug why]    a new post, a draft, its frontmatter filled in
//   cblo dev [--port 4321] [--no-drafts]  the blog served as its posts are saved, drafts included
//   cblo build [--out dist]               the blog as a static site, without drafts

import { parseArgs } from "node:util";
import { BlogError } from "./config.ts";
import { newPost } from "./new.ts";
import { serve, writeSite } from "./serve.ts";

const USAGE = `usage:
  cblo new <title> [--slug <slug>]
  cblo dev [--port <port>] [--no-drafts]
  cblo build [--out <folder>]`;

/** Runs `cblo` with `args`, in the blog's folder `dir`, on `today` (`YYYY-MM-DD`): its exit code. */
export async function run(args: string[], dir: string, today: string): Promise<number> {
  const { values, positionals } = parseArgs({
    args,
    allowPositionals: true,
    options: { slug: { type: "string" }, port: { type: "string", default: "4321" }, "no-drafts": { type: "boolean", default: false }, out: { type: "string", default: "dist" } },
  });
  const [command, ...rest] = positionals;
  try {
    if (command === "new" && rest.length === 1 && rest[0]) {
      const path = await newPost(dir, rest[0], { date: today, ...(values.slug === undefined ? {} : { slug: values.slug }) });
      console.log(`${path}: write the post, then take out \`draft: true\` to publish it`);
      return 0;
    }
    if (command === "build" && rest.length === 0) {
      const warnings = await writeSite(dir, values.out, { drafts: false });
      for (const warning of warnings) console.warn(warning);
      console.log(`built into ${values.out}/`);
      return 0;
    }
    if (command === "dev" && rest.length === 0) {
      const port = Number(values.port);
      await serve(dir, port, { drafts: !values["no-drafts"] });
      console.log(`the blog, at http://localhost:${String(port)}/ (Ctrl-C stops it)`);
      return 0;
    }
  } catch (error) {
    if (!(error instanceof BlogError)) throw error;
    console.error(error.message);
    return 1;
  }
  console.error(USAGE);
  return 2;
}

if (import.meta.main) {
  const code = await run(process.argv.slice(2), process.cwd(), new Date().toISOString().slice(0, 10));
  if (code !== 0) process.exit(code);
}
