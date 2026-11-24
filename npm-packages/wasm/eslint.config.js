import tseslint from "typescript-eslint";
import { lintConfig } from "../../eslint.shared.js";

export default lintConfig(
  import.meta.dirname,
  ["pkg"],
  { files: ["src/zones.js", "bench/worker.js"], ...tseslint.configs.disableTypeChecked },
  { files: ["bench/worker.js"], languageOptions: { globals: { self: "readonly", performance: "readonly", fetch: "readonly", postMessage: "readonly", navigator: "readonly" } } },
);
