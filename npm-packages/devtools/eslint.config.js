import { lintConfig } from "../../eslint.shared.js";

// `preview/` is a package of its own, the language server's page, which lints itself.
export default lintConfig(import.meta.dirname, ["preview"]);
