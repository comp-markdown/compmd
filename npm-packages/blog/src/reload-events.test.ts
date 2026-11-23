import { describe, expect, it } from "vitest";
import { RELOAD_PATH, withReload } from "./reload-events.ts";

describe("withReload", () => {
  it("puts the script that follows the stream before the page's </body>, or at its end without one", () => {
    const page = withReload("<html><body><p>Post</p></body></html>");
    expect(page).toMatch(/^<html><body><p>Post<\/p><script>new EventSource\("\/_cblo\/reload"\)\.onmessage = \(\) => location\.reload\(\);<\/script><\/body><\/html>$/);
    expect(withReload("<pre>error</pre>")).toMatch(/^<pre>error<\/pre><script>.*<\/script>$/);
    expect(RELOAD_PATH.startsWith("/_cblo/")).toBe(true);
  });
});
