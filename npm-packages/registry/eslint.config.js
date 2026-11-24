import { lintConfig } from "../../eslint.shared.js";

// The fixtures are packs as published, not this package's code.
export default lintConfig(import.meta.dirname, ["test/fixtures"]);
