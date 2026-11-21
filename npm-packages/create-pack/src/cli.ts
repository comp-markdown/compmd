#!/usr/bin/env node
// `npm create @compmd/pack <folder> [-- --name @scope/pack] [--component name] [--renders language | --library package |
// --from package[@version] [--export name] [--kind kind]]`: writes a pack, asking nothing, so it scripts the same way it
// runs (docs/create-pack.md, decisions 1 and 7; docs/wrapping-an-npm-package.md, decision 1).

import { execFileSync } from "node:child_process";
import { basename } from "node:path";
import { parseArgs } from "node:util";
import { KINDS, type Kind } from "@compmd/pack/wrap";
import { componentOf, createPack, CreateError } from "./create.ts";
import { createWrappedPack } from "./wrap.ts";

const USAGE = `Writes a CompMD pack: a component as one built module, its signature, and a test page.

  npm create @compmd/pack <folder> [-- options]

  --name <@scope/pack>   the npm package; by default @<your npm user>/<folder>
  --component <name>     the component's name; by default the folder's
  --renders <language>   make it a code renderer of that language's code blocks
  --library <package>    make it a library pack of that npm package, which packs share
  --from <package>       make it a pack of that npm package's component (react-qr-code@2.2.0), installed
  --export <name>        with --from: the export that is the component (BarChart), or a web component's tag
  --kind <kind>          with --from: ${KINDS.join(", ")}, over what is read from the package
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
  options: {
    name: { type: "string" },
    component: { type: "string" },
    renders: { type: "string" },
    library: { type: "string" },
    from: { type: "string" },
    export: { type: "string" },
    kind: { type: "string" },
    help: { type: "boolean", short: "h" },
  },
});
const [folder] = positionals;
if (values.help || folder === undefined) {
  process.stdout.write(USAGE);
  process.exit(folder === undefined && !values.help ? 1 : 0);
}
/** Ends with `message`, as a mistake in the options does. */
const refuse = (message: string): never => {
  process.stderr.write(`${message}\n`);
  process.exit(1);
};
if (values.library !== undefined && values.component !== undefined) refuse("a library pack has no components: leave out --component");
if (values.from !== undefined && (values.library !== undefined || values.renders !== undefined)) refuse("--from makes a pack of a package's component: leave out --library and --renders");
if (values.from === undefined && (values.export !== undefined || values.kind !== undefined)) refuse("--export and --kind say what --from wraps: give --from too");
if (values.kind !== undefined && !KINDS.includes(values.kind as Kind)) refuse(`--kind is one of ${KINDS.join(", ")}`);
const component = values.component ?? componentOf(folder);
const user = values.name === undefined ? npmUser() : undefined;
const name = values.name ?? `@${user ?? "me"}/${basename(folder).toLowerCase().replace(/[^a-z0-9._~-]+/g, "-")}`;
try {
  const { library, renders, from } = values;
  if (from !== undefined) {
    const { written, kind, notes } = await createWrappedPack({
      folder,
      name,
      component,
      from,
      ...(values.export === undefined ? {} : { export: values.export }),
      ...(values.kind === undefined ? {} : { kind: values.kind as Kind }),
    });
    process.stdout.write(`Wrote ${name} in ${folder}: ${String(written.length)} files, with the component ${component}, ${from} as a ${kind} component.\n`);
    for (const note of notes) process.stdout.write(`  ${note}\n`);
    process.stdout.write(`\n  cd ${folder}\n  npm run dev\n`);
  } else {
    const written = await createPack({ folder, name, component, ...(renders === undefined ? {} : { renders }), ...(library === undefined ? {} : { library }) });
    const what = library === undefined ? `with the component ${component}${renders ? `, a renderer of \`\`\`${renders} blocks` : ""}` : `a library pack of ${library}`;
    process.stdout.write(`Wrote ${name} in ${folder}: ${String(written.length)} files, ${what}.\n`);
    process.stdout.write(`\n  cd ${folder}\n  npm install\n  npm run ${library === undefined ? "dev" : "build"}\n`);
  }
  if (values.name === undefined && user === undefined) process.stdout.write(`npm didn't say who you are, so the package is ${name}: change its name in package.json${values.library === undefined ? " and index.comp.md" : ""}.\n`);
} catch (error) {
  process.stderr.write(`${error instanceof CreateError ? error.message : String(error)}\n`);
  process.exit(1);
}
