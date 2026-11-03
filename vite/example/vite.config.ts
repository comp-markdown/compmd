import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [compmd()],
  build: { target: "es2024" },
  // The inspector's workers are files of their own, which Vite builds only in a package it doesn't pre-bundle.
  optimizeDeps: { exclude: ["@compmd/devtools"] },
});
