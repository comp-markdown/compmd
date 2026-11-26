// The preview page's build: the page into `crates/lsp/preview/page/`, and its bundle (the sandbox's frames, the
// manifest) into `crates/lsp/preview/compmd/`, which the language server's `build.rs` embeds and serves, the frames on a
// port of their own; the page takes their origin from the server (`globalThis.__COMPMD__`).
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  build: { target: "es2024", outDir: "../../../crates/lsp/preview/page", emptyOutDir: true },
  worker: { format: "es" },
  // The tools' workers are files beside their code, which Vite builds only from the package as it is.
  optimizeDeps: { exclude: ["@compmd/devtools"] },
  plugins: [compmd({ out: "../../../crates/lsp/preview/compmd" })],
});
