import { describe, expect, it } from "vitest";
import { escape, inScript, inStyle } from "./html.ts";

describe("what the blog writes into its pages", () => {
  it("escapes text and attribute values", () => {
    expect(escape(`<a href="x">Tom & Jerry's</a>`)).toBe("&lt;a href=&quot;x&quot;&gt;Tom &amp; Jerry&#39;s&lt;/a&gt;");
  });

  it("writes a script's code and a style's CSS so nothing in them ends their element", () => {
    const code = `s.setAttribute("data-repo",${JSON.stringify("acme</script><script>alert(1)")});`;
    expect(inScript(code)).not.toContain("</script");
    expect(inScript(code)).toContain("\\u003c/script>");
    expect(inStyle(`.a::after { content: "</style><img src=x>"; }`)).toBe(`.a::after { content: "\\3c /style>\\3c img src=x>"; }`);
  });
});
