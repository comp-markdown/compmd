import { join } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  worker: { format: "es" },
  build: {
    target: "es2024",
    rolldownOptions: {
      // `frame.js`, at a name of its own, is what each frame of `?frames=N` imports (CMD-260).
      input: { index: join(import.meta.dirname, "index.html"), frame: join(import.meta.dirname, "src/frame.ts") },
      // Its export is what the frames call, which nothing in the page imports.
      preserveEntrySignatures: "exports-only",
      output: { entryFileNames: (chunk) => (chunk.name === "frame" ? "frame.js" : "assets/[name]-[hash].js") },
    },
  },
});
