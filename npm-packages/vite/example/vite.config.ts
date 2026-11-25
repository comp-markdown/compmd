import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [compmd()],
  // Two pages: the app's own, and readers' posts, with a configuration of its own (`readers/compmd.config.json`).
  build: { target: "es2024", rolldownOptions: { input: { main: "index.html", readers: "readers/index.html" } } },
  // The inspector's workers are files of their own, which Vite builds only in a package it doesn't pre-bundle.
  optimizeDeps: { exclude: ["@compmd/devtools"] },
});
