import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { chromium, type Browser } from "playwright-core";
import { defineConfig, type Plugin } from "vite";
import { bookPages, type BookPage, type BookProps } from "./build/book.ts";
import { snapshotsOf } from "./build/plugin.ts";
import { guidePages, type GuidePage } from "./build/guide-pages.ts";
import { PRINT, printJobs, printPdf } from "./build/print.ts";
import { specPages } from "./build/versions.ts";
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
        return prerendered(html, devStylesheets(renderPage({ html: text, toc, root: "../", versions: [] })), () => "");
      },
    },
  };
}

/** A book's or a chapter's address in the guides, with its folder there: `developers/`, `developers/quick-start/`. */
const BOOK = /^\/guides\/([a-z-]+\/(?:[a-z0-9-]+\/)?)$/;

/**
 * In the dev server, the guides' pages rendered from the working tree, all of them when it starts, and again at the
 * next request after a chapter or a contents changes: the shelf, `/guides/`, a book's `/guides/<book>/` and a chapter's `/guides/<book>/<slug>/`, which the build
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
  /** Each book's page to print, made only when one is asked for: the PDFs are. */
  let books: Promise<BookPage[]> | undefined;
  return {
    name: "compmd-guide-dev",
    apply: "serve",
    configureServer(server) {
      // Rendered ahead, so the first page asked for comes as fast as the others.
      pages = render();
      server.watcher.add(guide);
      server.watcher.on("change", (file) => {
        // A book's own files: the chapters and the contents, not what the guides' tests write into `test/.out/`.
        if (!/^(?:[^/\\]+[/\\])?[^/\\]+\.md$/.test(relative(guide, file))) return;
        // Rendered again when a page is next asked for, so a burst of changes renders once.
        pages = undefined;
        books = undefined;
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
        const { renderGuidePage, renderBookPage } = (await server.ssrLoadModule("/src/server.ts")) as {
          renderGuidePage: (props: GuidePage["props"]) => Rendered;
          renderBookPage: (props: BookProps) => Rendered;
        };
        pages ??= render();
        const page = (await pages).find((candidate) => candidate.folder === folder);
        if (page) return prerendered(relocated(html, page.root), devStylesheets(renderGuidePage(page.props)), () => "");
        if (!folder.endsWith(`/${PRINT}`)) return html;
        books ??= bookPages((path) => readFileSync(join(guide, path), "utf8"), "https://github.com/comp-markdown/compmd/blob/main/");
        const book = (await books).find((candidate) => candidate.folder === folder);
        return book ? prerendered(relocated(html, book.root), devStylesheets(renderBookPage(book.props)), () => "") : html;
      },
    },
  };
}

/**
 * In the dev server, a PDF asked for (the books' and the draft spec's), printed from the working tree's page when it is
 * asked for, as the build prints it (`build/print.ts`): a few seconds, in the installed Chrome.
 */
function pdfInDev(): Plugin {
  let browser: Promise<Browser> | undefined;
  return {
    name: "compmd-pdf-dev",
    apply: "serve",
    configureServer(server) {
      const jobs = printJobs(specPages([]));
      server.middlewares.use((request, response, next) => {
        const job = jobs.find(({ pdf }) => new URL(request.url ?? "/", "http://localhost").pathname === `/${pdf}`);
        if (!job) {
          next();
          return;
        }
        const print = async () => {
          browser ??= chromium.launch({ channel: "chrome" });
          const page = await (await browser).newPage({ colorScheme: "light" });
          try {
            const origin = server.resolvedUrls?.local[0] ?? `http://localhost:${String(server.config.server.port)}/`;
            const pdf = await printPdf(page, new URL(job.page, origin).href);
            response.setHeader("Content-Type", "application/pdf");
            response.end(pdf);
          } finally {
            await page.close();
          }
        };
        print().catch(next);
      });
      server.httpServer?.on("close", () => {
        void browser?.then((launched) => launched.close());
      });
    },
  };
}

export default defineConfig({
  base: "./",
  // `compmd()` gives the site its own pack (`app/`, trusted) and the sandbox's frames, under `compmd/`.
  plugins: [snapshotsOf(join(import.meta.dirname, "content"), import.meta.dirname), assembled(), specInDev(), guideInDev(), pdfInDev(), svelte(), compmd()],
  build: {
    target: "es2024",
    rolldownOptions: { input: { index: "index.html", spec: "spec/index.html", guides: "guides/index.html" } },
  },
  server: {
    fs: { allow: [".."] },
    // `guideInDev` watches `guides/` for the books' Markdown. Not what the guides' tests write as they run, nor their
    // examples: a page or a `tsconfig.json` written there would reload every page, and re-optimize the dependencies
    // while a page renders.
    watch: { ignored: ["**/guides/test/**", "**/guides/*/examples/**", "**/guides/node_modules/**"] },
  },
});
