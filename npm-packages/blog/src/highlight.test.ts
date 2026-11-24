import { describe, expect, it } from "vitest";
import { highlighter } from "./highlight.ts";

describe("the build's highlighter", () => {
  it("loads no language named after what every object inherits: a post's fence in `constructor` stays plain", async () => {
    const highlight = await highlighter();
    await expect(highlight.load(["constructor", "toString", "__proto__", "ts"])).resolves.toBeUndefined();
  });
});
