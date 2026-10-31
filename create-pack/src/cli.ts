#!/usr/bin/env node
// `npm create @compmd/pack <folder> [-- --name @scope/pack] [--component name] [--renders language]`: writes a pack,
// asking nothing, so it scripts the same way it runs (docs/create-pack.md, decision 1).

import { execFileSync } from "node:child_process";
import { basename } from "node:path";
import { parseArgs } from "node:util";
import { componentOf, createPack, CreateError } from "./create.ts";

const USAGE = `Writes a CompMD pack: a component as one built module, its signature, and a test page.

  npm create @compmd/pack <folder> [-- options]

  --name <@scope/pack>   the npm package; by default @<your npm user>/<folder>
  --component <name>     the component's name; by default the folder's
  --renders <language>   make it a code renderer of that language's code blocks
`;

/** The npm user logged in, if any. */
function npmUser(): string | undefined {
  try {
    return execFileSync("npm", ["whoami"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], timeout: 10_000 }).trim() || undefined;
  } catch {
    return undefined;
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { name: { type: "string" }, component: { type: "string" }, renders: { type: "string" }, help: { type: "boolean", short: "h" } },
});
const [folder] = positionals;
if (values.help || folder === undefined) {
  process.stdout.write(USAGE);
  process.exit(folder === undefined && !values.help ? 1 : 0);
}
const component = values.component ?? componentOf(folder);
const user = values.name === undefined ? npmUser() : undefined;
const name = values.name ?? `@${user ?? "me"}/${basename(folder).toLowerCase().replace(/[^a-z0-9._~-]+/g, "-")}`;
try {
  const written = await createPack({ folder, name, component, ...(values.renders === undefined ? {} : { renders: values.renders }) });
  process.stdout.write(`Wrote ${name} in ${folder}: ${String(written.length)} files, with the component ${component}${values.renders ? `, a renderer of \`\`\`${values.renders} blocks` : ""}.\n`);
  if (values.name === undefined && user === undefined) process.stdout.write(`npm didn't say who you are, so the package is ${name}: change its name in package.json and index.comp.md.\n`);
  process.stdout.write(`\n  cd ${folder}\n  npm install\n  npm run dev\n`);
} catch (error) {
  process.stderr.write(`${error instanceof CreateError ? error.message : String(error)}\n`);
  process.exit(1);
}
