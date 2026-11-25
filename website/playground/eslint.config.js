import { lintConfig } from "../../eslint.shared.js";

// `packs/` holds packages of their own, each with its own build and tests.
export default lintConfig(import.meta.dirname, ["public", "examples", "packs", "test/.out"]);
