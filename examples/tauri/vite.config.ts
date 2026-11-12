// The page's build, for Tauri: its dev server on the port `tauri.conf.json` names, a build into `dist/`, and the
// bundle (the frames, the installed Mermaid pack, the manifest) into `src-tauri/compmd/`, built for the scheme the
// plugin serves it on, which `src-tauri/build.rs` embeds in the app (docs/tauri-plugin.md, decision 3).
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "es2024", outDir: "dist", emptyOutDir: true },
  plugins: [compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })],
});
