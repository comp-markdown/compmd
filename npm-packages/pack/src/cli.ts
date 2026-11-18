#!/usr/bin/env node
// `compmd-pack build | dev | test`, run in a pack's folder by its scripts (docs/writing-a-pack.md, decision 2).

import { parseArgs } from "node:util";
import { buildModules } from "./build.ts";
import { checkPack } from "./check.ts";
import { servePack } from "./dev.ts";
import { PackError, readPack } from "./pack.ts";
import { testPack } from "./test.ts";

const USAGE = `Builds, tests and serves the CompMD pack in this folder.

  compmd-pack build              each module the pack publishes, then the publish check
  compmd-pack dev [--port <n>]   the test page, demo.comp.md in the sandbox, built again on save
  compmd-pack test [filter…]     the pack's tests, src/**/*.test.ts, in happy-dom
`;

const { values, positionals } = parseArgs({ allowPositionals: true, options: { port: { type: "string" }, help: { type: "boolean", short: "h" } } });
const [command, ...rest] = positionals;

try {
  if (values.help || command === undefined) {
    process.stdout.write(USAGE);
    process.exitCode = values.help ? 0 : 1;
  } else if (command === "build") {
    const pack = await readPack(process.cwd());
    await buildModules(pack);
    const problems = await checkPack(await readPack(process.cwd()));
    for (const problem of problems) process.stderr.write(`${problem}\n`);
    process.exitCode = problems.length > 0 ? 1 : 0;
  } else if (command === "dev") {
    const port = values.port === undefined ? undefined : Number(values.port);
    const server = await servePack(await readPack(process.cwd()), port === undefined ? {} : { port });
    server.printUrls();
  } else if (command === "test") {
    process.exitCode = (await testPack(await readPack(process.cwd()), { filters: rest })) ? 0 : 1;
  } else {
    process.stderr.write(`compmd-pack has no command ${command}\n\n${USAGE}`);
    process.exitCode = 1;
  }
} catch (error) {
  process.stderr.write(`${error instanceof PackError ? error.message : error instanceof Error ? (error.stack ?? error.message) : String(error)}\n`);
  process.exitCode = 1;
}
