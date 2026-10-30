import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { snapshotsOf } from "./build/plugin.ts";
import { prerendered, type Rendered } from "./build/prerender.ts";
import { renderSpec } from "./build/spec.ts";
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
        const { html: text, toc } = await renderSpec(readFileSync(SPEC, "utf8"), { blob: "https://github.com/comp-markdown/compmd/blob/main/", path: "compmd-spec-v1.md" });
        const rendered = renderSpecPage({ html: text, toc, root: "../", source: "", versions: [] });
        return prerendered(html, rendered, () => "");
      },
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [snapshotsOf(join(import.meta.dirname, "content")), assembled(), specInDev(), svelte()],
  build: {
    target: "es2024",
    rolldownOptions: { input: { index: "index.html", spec: "spec/index.html" } },
  },
  server: { fs: { allow: [".."] } },
});
