// The page's build, for Tauri: its dev server on the port `tauri.conf.json` names, and a build into `dist/`.
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  // The Mermaid pack's module is in the page's bundle, as text the frames load: about 5 MB.
  build: { target: "es2024", outDir: "dist", emptyOutDir: true, chunkSizeWarningLimit: 8000 },
});
