import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["node_modules", "dist", "pkg"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
  },
  { files: ["*.js", "src/zones.js", "bench/worker.js"], ...tseslint.configs.disableTypeChecked },
  { files: ["bench/worker.js"], languageOptions: { globals: { self: "readonly", performance: "readonly", fetch: "readonly", postMessage: "readonly", navigator: "readonly" } } },
);
