import { svelte } from "@sveltejs/vite-plugin-svelte";
import { execFileSync } from "node:child_process";
import { defineConfig } from "vite";

/** The commit the playground is built from, which its About names; empty outside a git checkout. */
function commit(): string {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { cwd: import.meta.dirname, encoding: "utf8" }).trim();
  } catch {
    return "";
  }
}

// The page's frames have an opaque origin, so the frame documents' files load cross-origin (`public/sandbox/`).
const cors = { origin: "*" };

export default defineConfig({
  base: "./",
  plugins: [svelte()],
  define: { __COMMIT__: JSON.stringify(commit()) },
  build: { target: "es2024" },
  worker: { format: "es" },
  // The editor's own page only: the frame documents in `public/sandbox/` import the runtimes through their import map.
  optimizeDeps: { entries: ["index.html"] },
  server: { cors, fs: { allow: [".."] } },
  preview: { cors },
});
