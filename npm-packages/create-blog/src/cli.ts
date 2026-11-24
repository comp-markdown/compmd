#!/usr/bin/env node
// `npm create @compmd/blog <folder> [-- --title "My notes"] [--author "Ada"] [--url https://example.com/blog/]`: writes a
// new blog, asking nothing, so it scripts the same way it runs.

import { execFileSync } from "node:child_process";
import { parseArgs } from "node:util";
import { createBlog, CreateError } from "./create.ts";
import { localDay } from "./local-day.ts";

const USAGE = `Writes a blog in CompMD: its configuration and a first post.

  npm create @compmd/blog <folder> [-- options]

  --title <title>   the blog's title; by default the folder's name
  --author <name>   who writes it; by default git's user.name
  --url <address>   where it is published, ending in /; set it later in blog.config.json otherwise
`;

/** git's `user.name`, if it has one. */
function gitUser(): string | undefined {
  try {
    return execFileSync("git", ["config", "user.name"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], timeout: 10_000 }).trim() || undefined;
  } catch {
    return undefined;
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { title: { type: "string" }, author: { type: "string" }, url: { type: "string" }, help: { type: "boolean", short: "h" } },
});
const [folder] = positionals;
if (values.help || folder === undefined) {
  process.stdout.write(USAGE);
  process.exit(folder === undefined && !values.help ? 1 : 0);
}
try {
  const { title, url } = values;
  const author = values.author ?? gitUser() ?? "Me";
  const written = await createBlog({ folder, author, today: localDay(), ...(title === undefined ? {} : { title }), ...(url === undefined ? {} : { url }) });
  process.stdout.write(`Wrote a blog in ${folder}: ${String(written.length)} files.\n\n  cd ${folder}\n  npm install\n  npm run dev\n`);
} catch (error) {
  process.stderr.write(`${error instanceof CreateError ? error.message : String(error)}\n`);
  process.exit(1);
}
