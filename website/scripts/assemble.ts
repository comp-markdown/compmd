// Assembles the site in `dist/`, after the prerender: the playground's build at `playground/`, and the logo files at
// `logo/`. Each part keeps its own folder and build; this only copies them in.
import { cpSync, existsSync, rmSync } from "node:fs";
import { join } from "node:path";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");

const PARTS = [
  { from: "editor/dist", to: "playground", what: "the playground's build: `pnpm --dir editor build`" },
  { from: "logo", to: "logo", what: "the logo files" },
];

for (const { from, to, what } of PARTS) {
  const source = join(REPO, from);
  if (!existsSync(source)) throw new Error(`${from} is missing: ${what}`);
  rmSync(join(DIST, to), { recursive: true, force: true });
  cpSync(source, join(DIST, to), { recursive: true, filter: (path) => !path.endsWith(".md") });
}
console.log(`Assembled ${PARTS.map(({ to }) => `${to}/`).join(" and ")} into dist/`);
