// Writes the server-rendered pages into the client build, `dist/`: the landing page, and each version of the spec
// (`build/versions.ts`), the draft from the working tree and each release from its tag. Runs after `vite build` and
// the server build.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { prerendered, relocated, type Rendered } from "../build/prerender.ts";
import { renderSpec } from "../build/spec.ts";
import { rootOf, specPages } from "../build/versions.ts";
import type { ComponentProps } from "svelte";
import type Spec from "../src/Spec.svelte";
import { REPOSITORY } from "../src/site.ts";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");
const SPEC = "compmd-spec-v1.md";

const server = (await import(join(import.meta.dirname, "../dist-server/server.js"))) as {
  renderPage: () => Rendered;
  renderSpecPage: (props: ComponentProps<typeof Spec>) => Rendered;
};
const git = (...args: string[]) => execFileSync("git", args, { cwd: REPO, encoding: "utf8" });
const stylesheet = (folder: string) => (href: string) => readFileSync(join(DIST, folder, href), "utf8");

const index = join(DIST, "index.html");
writeFileSync(index, prerendered(readFileSync(index, "utf8"), server.renderPage(), stylesheet("")));

const template = readFileSync(join(DIST, "spec/index.html"), "utf8");
const pages = specPages(git("tag", "--list", "spec-v*").split("\n").filter(Boolean));
// Each version once, at the folder that is its own: `spec/` shows the latest release, which has its own folder too.
const versions = pages.filter((page) => page.folder !== "spec/" || page.version === undefined);
for (const page of pages) {
  const root = rootOf(page.folder);
  const markdown = page.ref === undefined ? readFileSync(join(REPO, SPEC), "utf8") : git("show", `${page.ref}:${SPEC}`);
  const blob = `${REPOSITORY}/blob/${page.ref ?? "main"}/`;
  const { html, toc } = await renderSpec(markdown, { blob, path: SPEC });
  const rendered = server.renderSpecPage({
    html,
    toc,
    root,
    version: page.version,
    source: `${blob}${SPEC}`,
    versions: versions.map((other) => ({ label: other.version ?? "Draft", href: `${root}${other.folder}`, current: other.version === page.version })),
  });
  mkdirSync(join(DIST, page.folder), { recursive: true });
  writeFileSync(join(DIST, page.folder, "index.html"), prerendered(relocated(template, root), rendered, stylesheet(page.folder)));
}
console.log(`Prerendered the landing page and ${String(pages.length)} page(s) of the spec: ${pages.map((page) => page.folder).join(", ")}`);
