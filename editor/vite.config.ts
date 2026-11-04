import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// The page's frames have an opaque origin, so the files they load from the site, `frame.css` among them, load
// cross-origin; `@compmd/vite` gives its own files, under `compmd/`, the same header.
const cors = { origin: "*" };

// `compmd()` reads `compmd.config.json`: the playground's packs, the sandbox's frame documents with d3, and the built
// modules, served under `compmd/`; `virtual:compmd` gives them to the documents and the page.
export default defineConfig({
  base: "./",
  plugins: [svelte(), compmd()],
  build: { target: "es2024" },
  worker: { format: "es" },
  // The tools' workers are files of their own, which Vite builds only in a package it doesn't pre-bundle.
  optimizeDeps: { entries: ["index.html"], exclude: ["@compmd/devtools"] },
  server: { cors, fs: { allow: [".."] } },
  preview: { cors },
});
