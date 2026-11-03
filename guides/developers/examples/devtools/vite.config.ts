import { join } from "node:path";
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [compmd()],
  // Two pages: the app's, with the inspector in development, and the editor's.
  build: { target: "es2024", rolldownOptions: { input: [join(import.meta.dirname, "index.html"), join(import.meta.dirname, "editor.html")] } },
  // The tools' workers are files of their own, which Vite builds only in a package it doesn't pre-bundle.
  optimizeDeps: { exclude: ["@compmd/devtools"] },
});
