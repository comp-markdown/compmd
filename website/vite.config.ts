import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { snapshotsOf } from "./build/plugin.ts";
import { guidePages, type GuidePage } from "./build/guide-pages.ts";
import { prerendered, relocated, type Rendered } from "./build/prerender.ts";
import { COMPANION, renderSpecPage, SPEC } from "./build/companion.ts";
import { serveFolder } from "./build/static.ts";

const REPO = join(import.meta.dirname, "..");
const SPEC_FILES = [SPEC, COMPANION].map((file) => join(REPO, file));

/**
 * In the dev server, what the spec's and the guides' pages link in their head: the stylesheets `src/spec.ts` imports,
 * which the build inlines, so a page paints styled before its script runs. The script removes them (`data-dev`).
 */
const devStylesheets = (rendered: Rendered): Rendered => ({
  ...rendered,
  head: `${rendered.head}${["/src/base.css", "/src/spec.css"].map((href) => `<link rel="stylesheet" href="${href}" data-dev>`).join("")}`,
});

/** In the dev server, what the build assembles into the site (`scripts/assemble.ts`): the playground's build and the logo. */
function assembled(): Plugin {
  return {
    name: "compmd-assembled",
    configureServer(server) {
      server.middlewares.use(serveFolder("/playground/", join(import.meta.dirname, "../editor/dist")));
      server.middlewares.use(serveFolder("/logo/", join(import.meta.dirname, "../logo")));
    },
  };
}

/**
 * In the dev server, the spec page, with its companion, rendered from the working tree, again after either changes; the
 * build prerenders it.
 */
function specInDev(): Plugin {
  let rendered: ReturnType<typeof renderSpecPage> | undefined;
  return {
    name: "compmd-spec-dev",
    apply: "serve",
    configureServer(server) {
      server.watcher.add(SPEC_FILES);
      server.watcher.on("change", (file) => {
        if (!SPEC_FILES.includes(file)) return;
        rendered = undefined;
        server.ws.send({ type: "full-reload" });
      });
    },
    transformIndexHtml: {
      order: "pre",
      async handler(html, { filename, server }) {
        if (!server || !filename.endsWith(join("spec", "index.html"))) return html;
        const { renderSpecPage: renderPage } = (await server.ssrLoadModule("/src/server.ts")) as { renderSpecPage: (props: object) => Rendered };
        rendered ??= renderSpecPage((path) => readFileSync(join(REPO, path), "utf8"), "https://github.com/comp-markdown/compmd/blob/main/");
        const { html: text, toc } = await rendered;
        return prerendered(html, devStylesheets(renderPage({ html: text, toc, root: "../", source: "", versions: [] })), () => "");
      },
    },
  };
}

/** A book's or a chapter's address in the guides, with its folder there: `developers/`, `developers/quick-start/`. */
const BOOK = /^\/guides\/([a-z-]+\/(?:[a-z0-9-]+\/)?)$/;

/**
 * In the dev server, the guides' pages rendered from the working tree, all of them when it starts and again when a
 * guide's file changes: the shelf, `/guides/`, a book's `/guides/<book>/` and a chapter's `/guides/<book>/<slug>/`, which the build
 * prerenders. A book's and a chapter's addresses are served from the shelf's page.
 */
function guideInDev(): Plugin {
  const guide = join(import.meta.dirname, "../guides");
  const render = () => {
    const rendering = guidePages((path) => readFileSync(join(guide, path), "utf8"), "https://github.com/comp-markdown/compmd/blob/main/");
    // A guide that fails to render fails the page asked for, not the server, before any page is asked for.
    rendering.catch(() => undefined);
    return rendering;
  };
  let pages: Promise<GuidePage[]> | undefined;
  return {
    name: "compmd-guide-dev",
    apply: "serve",
    configureServer(server) {
      // Rendered ahead, so the first page asked for comes as fast as the others.
      pages = render();
      server.watcher.add(guide);
      server.watcher.on("change", (file) => {
        if (!file.startsWith(guide)) return;
        pages = render();
        server.ws.send({ type: "full-reload" });
      });
      server.middlewares.use((request, _response, next) => {
        if (BOOK.test(request.url ?? "")) request.url = "/guides/index.html";
        next();
      });
    },
    transformIndexHtml: {
      order: "pre",
      async handler(html, { filename, server, originalUrl }) {
        if (!server || !filename.endsWith(join("guides", "index.html"))) return html;
        const folder = `guides/${BOOK.exec(originalUrl ?? "")?.[1] ?? ""}`;
        const { renderGuidePage } = (await server.ssrLoadModule("/src/server.ts")) as { renderGuidePage: (props: GuidePage["props"]) => Rendered };
        pages ??= render();
        const page = (await pages).find((candidate) => candidate.folder === folder);
        if (!page) return html;
        return prerendered(relocated(html, page.root), devStylesheets(renderGuidePage(page.props)), () => "");
      },
    },
  };
}

export default defineConfig({
  base: "./",
  // `compmd()` gives the site its own pack (`app/`, trusted) and the sandbox's frames, under `compmd/`.
  plugins: [snapshotsOf(join(import.meta.dirname, "content"), import.meta.dirname), assembled(), specInDev(), guideInDev(), svelte(), compmd()],
  build: {
    target: "es2024",
    rolldownOptions: { input: { index: "index.html", spec: "spec/index.html", guides: "guides/index.html" } },
  },
  server: { fs: { allow: [".."] } },
});
