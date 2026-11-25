// Assembles the site in `dist/`, after the prerender: the logo files at `logo/`. The playground is a page of the
// site's own build, at `playground/`; this only copies in what isn't.
import { cpSync, existsSync, rmSync } from "node:fs";
import { join } from "node:path";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");

const PARTS = [{ from: "branding", to: "logo", what: "the logo files" }];

for (const { from, to, what } of PARTS) {
  const source = join(REPO, from);
  if (!existsSync(source)) throw new Error(`${from} is missing: ${what}`);
  rmSync(join(DIST, to), { recursive: true, force: true });
  cpSync(source, join(DIST, to), { recursive: true, filter: (path) => !path.endsWith(".md") });
}
console.log(`Assembled ${PARTS.map(({ to }) => `${to}/`).join(" and ")} into dist/`);
