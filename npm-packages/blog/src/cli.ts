#!/usr/bin/env node
// `cblo` (also `compmd-blog`), run in the blog's folder (docs/blog-tool.md, decision 3). Each command is a function in
// COMMANDS below, with its usage; the work is done by the module it calls:
//
//   cblo new <title> [--slug <slug>]        newPost, new.ts: a new post, a draft, its frontmatter filled in
//   cblo dev [--port <port>] [--no-drafts]  serve, serve.ts: the blog served as its posts are saved, drafts included
//   cblo build [--out <folder>]             writeSite, serve.ts (buildSite, build.ts): the blog as a static site

import { parseArgs } from "node:util";
import { BlogError } from "./config.ts";
import { localDay } from "./local-day.ts";
import { newPost } from "./new.ts";
import { serve, writeSite } from "./serve.ts";

/** What a command is given: its positional arguments after its name, its options, the blog's folder, and today. */
interface Context {
  args: string[];
  options: { slug?: string | undefined; port: string; "no-drafts": boolean; out: string };
  dir: string;
  today: string;
}

/** A command: its usage, and what it does; `undefined` when its arguments are wrong, an exit code otherwise. */
interface Command {
  usage: string;
  run(context: Context): Promise<number | undefined>;
}

/** The commands, by name. */
const COMMANDS: Record<string, Command> = {
  new: {
    usage: "cblo new <title> [--slug <slug>]",
    async run({ args, options, dir, today }) {
      const [title] = args;
      if (args.length !== 1 || !title) return undefined;
      const path = await newPost(dir, title, { date: today, ...(options.slug === undefined ? {} : { slug: options.slug }) });
      console.log(`${path}: write the post, then take out \`draft: true\` to publish it`);
      return 0;
    },
  },
  dev: {
    usage: "cblo dev [--port <port>] [--no-drafts]",
    async run({ args, options, dir }) {
      if (args.length !== 0) return undefined;
      const port = Number(options.port);
      if (!/^\d+$/.test(options.port) || port < 1 || port > 65535) throw new BlogError(`--port is a port number, 1 to 65535: not ${options.port}`);
      await serve(dir, port, { drafts: !options["no-drafts"] });
      console.log(`the blog, at http://localhost:${String(port)}/ (Ctrl-C stops it)`);
      return 0;
    },
  },
  build: {
    usage: "cblo build [--out <folder>]",
    async run({ args, options, dir }) {
      if (args.length !== 0) return undefined;
      const warnings = await writeSite(dir, options.out, { drafts: false });
      for (const warning of warnings) console.warn(warning);
      console.log(`built into ${options.out}/`);
      return 0;
    },
  },
};

const USAGE = `usage:\n${Object.values(COMMANDS)
  .map(({ usage }) => `  ${usage}`)
  .join("\n")}`;

/** Runs `cblo` with `args`, in the blog's folder `dir`, on `today` (`YYYY-MM-DD`): its exit code. */
export async function run(args: string[], dir: string, today: string): Promise<number> {
  const { values, positionals } = parseArgs({
    args,
    allowPositionals: true,
    options: { slug: { type: "string" }, port: { type: "string", default: "4321" }, "no-drafts": { type: "boolean", default: false }, out: { type: "string", default: "dist" } },
  });
  const [name = "", ...rest] = positionals;
  const command = Object.hasOwn(COMMANDS, name) ? COMMANDS[name] : undefined;
  try {
    const code = await command?.run({ args: rest, options: values, dir, today });
    if (code !== undefined) return code;
  } catch (error) {
    if (!(error instanceof BlogError)) throw error;
    console.error(error.message);
    return 1;
  }
  console.error(USAGE);
  return 2;
}

if (import.meta.main) {
  const code = await run(process.argv.slice(2), process.cwd(), localDay());
  if (code !== 0) process.exit(code);
}
