// Writes the server-rendered pages into the client build, `dist/`: the landing page, each version of the spec with its
// companion (`build/versions.ts`, `build/companion.ts`), the draft from the working tree and each release from its tag,
// the guides' pages and the page a wrong address gets, `404.html`; the search index of them, `search.json`
// (`build/search.ts`); and `sitemap.xml` and `robots.txt` (`build/sitemap.ts`). Runs after `vite build` and the server
// build.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { renderSpecPage } from "../build/companion.ts";
import { bookPages, type BookProps } from "../build/book.ts";
import { guidePages } from "../build/guide-pages.ts";
import { prerendered, relocated, type Rendered } from "../build/prerender.ts";
import { sectionsOf, sitePages } from "../build/search.ts";
import { ROBOTS, sitemapOf } from "../build/sitemap.ts";
import { rootOf, specPages } from "../build/versions.ts";
import type { ComponentProps } from "svelte";
import type Guide from "../src/Guide.svelte";
import type Spec from "../src/Spec.svelte";
import { REPOSITORY } from "../src/site.ts";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");

const server = (await import(join(import.meta.dirname, "../dist-server/server.js"))) as {
  renderPage: () => Rendered;
  renderSpecPage: (props: ComponentProps<typeof Spec>) => Rendered;
  renderGuidePage: (props: ComponentProps<typeof Guide>) => Rendered;
  renderBookPage: (props: BookProps) => Rendered;
  renderNotFoundPage: () => Rendered;
};
const git = (...args: string[]) => execFileSync("git", args, { cwd: REPO, encoding: "utf8" });
const stylesheet = (folder: string) => (href: string) => readFileSync(join(DIST, folder, href), "utf8");

const index = join(DIST, "index.html");
const landing = server.renderPage();
writeFileSync(index, prerendered(readFileSync(index, "utf8"), landing, stylesheet("")));

const template = readFileSync(join(DIST, "spec/index.html"), "utf8");
const pages = specPages(git("tag", "--list", "spec-v*").split("\n").filter(Boolean));
// Each version once, at the folder that is its own: `spec/` shows the latest release, which has its own folder too.
const versions = pages.filter((page) => page.folder !== "spec/" || page.version === undefined);
/** A file of the repository at `ref`, or the working tree's; `undefined` when it wasn't there then. */
const fileAt = (ref: string | undefined, path: string): string | undefined => {
  if (ref === undefined) return readFileSync(join(REPO, path), "utf8");
  try {
    return execFileSync("git", ["show", `${ref}:${path}`], { cwd: REPO, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
  } catch {
    return undefined;
  }
};

/** The spec page's content at `spec/`, for the search index. */
let specAtSpec = "";
for (const page of pages) {
  const root = rootOf(page.folder);
  const blob = `${REPOSITORY}/blob/${page.ref ?? "main"}/`;
  // The spec and its companion, at the page's version; one released before the companion has the spec alone.
  const { html, toc } = await renderSpecPage((path) => fileAt(page.ref, path), blob);
  if (page.folder === "spec/") specAtSpec = html;
  const rendered = server.renderSpecPage({
    html,
    toc,
    root,
    version: page.version,
    versions: versions.map((other) => ({ label: other.version ?? "Draft", href: `${root}${other.folder}`, current: other.version === page.version })),
  });
  mkdirSync(join(DIST, page.folder), { recursive: true });
  writeFileSync(join(DIST, page.folder, "index.html"), prerendered(relocated(template, root), rendered, stylesheet(page.folder)));
}

// The guides, from the working tree: they follow `main`.
const guideTemplate = readFileSync(join(DIST, "guides/index.html"), "utf8");
const guide = await guidePages((path) => readFileSync(join(REPO, "guides", path), "utf8"), `${REPOSITORY}/blob/main/`);
for (const page of guide) {
  mkdirSync(join(DIST, page.folder), { recursive: true });
  writeFileSync(join(DIST, page.folder, "index.html"), prerendered(relocated(guideTemplate, page.root), server.renderGuidePage(page.props), stylesheet(page.folder)));
}
// Each book as one page, which `scripts/pdf.ts` prints into the book's PDF, then removes.
// The search index: every section of the landing page, the spec at `spec/` and the guides.
writeFileSync(join(DIST, "search.json"), JSON.stringify(sectionsOf(sitePages(landing.body, specAtSpec, guide))));
const books = await bookPages((path) => readFileSync(join(REPO, "guides", path), "utf8"), `${REPOSITORY}/blob/main/`);
for (const page of books) {
  mkdirSync(join(DIST, page.folder), { recursive: true });
  writeFileSync(join(DIST, page.folder, "index.html"), prerendered(relocated(guideTemplate, page.root), server.renderBookPage(page.props), stylesheet(page.folder)));
}
// The page a wrong address gets, at any depth: the guides' page, with its links and stylesheets from the site's root.
writeFileSync(join(DIST, "404.html"), prerendered(relocated(guideTemplate, "/"), server.renderNotFoundPage(), stylesheet("")));
// What the site tells search engines: every page made above, and the playground.
writeFileSync(join(DIST, "sitemap.xml"), sitemapOf(["", ...versions.map((page) => page.folder), ...guide.map((page) => page.folder), "playground/"]));
writeFileSync(join(DIST, "robots.txt"), ROBOTS);
console.log(`Prerendered the landing page, ${String(pages.length)} page(s) of the spec (${pages.map((page) => page.folder).join(", ")}), ${String(guide.length)} of the guides and ${String(books.length)} books to print`);
