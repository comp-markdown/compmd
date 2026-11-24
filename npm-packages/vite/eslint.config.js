import { lintConfig } from "../../eslint.shared.js";

export default lintConfig(import.meta.dirname, ["example", "test/.out"]);
