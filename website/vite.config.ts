import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { snapshotsOf } from "./build/plugin.ts";
import { guidePages, type GuidePage } from "./build/guide-pages.ts";
import { prerendered, relocated, type Rendered } from "./build/prerender.ts";
import { renderMarkdown } from "./build/spec.ts";
import { serveFolder } from "./build/static.ts";

const SPEC = join(import.meta.dirname, "../compmd-spec-v1.md");

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

/** In the dev server, the spec page rendered from the working tree on each request; the build prerenders it. */
function specInDev(): Plugin {
  return {
    name: "compmd-spec-dev",
    apply: "serve",
    configureServer(server) {
      server.watcher.add(SPEC);
      server.watcher.on("change", (file) => {
        if (file === SPEC) server.ws.send({ type: "full-reload" });
      });
    },
    transformIndexHtml: {
      order: "pre",
      async handler(html, { filename, server }) {
        if (!server || !filename.endsWith(join("spec", "index.html"))) return html;
        const { renderSpecPage } = (await server.ssrLoadModule("/src/server.ts")) as { renderSpecPage: (props: object) => Rendered };
        const { html: text, toc } = await renderMarkdown(readFileSync(SPEC, "utf8"), { blob: "https://github.com/comp-markdown/compmd/blob/main/", path: "compmd-spec-v1.md" }, { rules: true });
        const rendered = renderSpecPage({ html: text, toc, root: "../", source: "", versions: [] });
        return prerendered(html, rendered, () => "");
      },
    },
  };
}

/** A chapter's address in the guide, with its slug. */
const CHAPTER = /^\/guide\/([a-z0-9-]+)\/$/;

/**
 * In the dev server, the guide's pages rendered from the working tree on each request: `/guide/` and a chapter's
 * `/guide/<slug>/`, which the build prerenders. A chapter's address is served from the guide's own page.
 */
function guideInDev(): Plugin {
  const guide = join(import.meta.dirname, "../guide");
  return {
    name: "compmd-guide-dev",
    apply: "serve",
    configureServer(server) {
      server.watcher.add(guide);
      server.watcher.on("change", (file) => {
        if (file.startsWith(guide)) server.ws.send({ type: "full-reload" });
      });
      server.middlewares.use((request, _response, next) => {
        if (CHAPTER.test(request.url ?? "")) request.url = "/guide/index.html";
        next();
      });
    },
    transformIndexHtml: {
      order: "pre",
      async handler(html, { filename, server, originalUrl }) {
        if (!server || !filename.endsWith(join("guide", "index.html"))) return html;
        const chapter = CHAPTER.exec(originalUrl ?? "")?.[1] ?? null;
        const { renderGuidePage } = (await server.ssrLoadModule("/src/server.ts")) as { renderGuidePage: (props: GuidePage["props"]) => Rendered };
        const pages = await guidePages((file) => readFileSync(join(guide, file), "utf8"), "https://github.com/comp-markdown/compmd/blob/main/");
        const page = pages.find((candidate) => candidate.folder === (chapter === null ? "guide/" : `guide/${chapter}/`));
        if (!page) return html;
        return prerendered(relocated(html, page.root), renderGuidePage(page.props), () => "");
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
    rolldownOptions: { input: { index: "index.html", spec: "spec/index.html", guide: "guide/index.html" } },
  },
  server: { fs: { allow: [".."] } },
});
