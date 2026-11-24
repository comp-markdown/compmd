import { describe, expect, it } from "vitest";
import { remoteOrigin } from "./remote-origin.ts";

const BASE = "https://docs.example.com/notes/";

describe("remoteOrigin", () => {
  it("is the origin an image loads from when it isn't the page's, as the browser reads its src", () => {
    expect(remoteOrigin("https://photos.example.com/a.png", BASE)).toBe("https://photos.example.com");
    expect(remoteOrigin("//tracker.example/p.png", BASE)).toBe("https://tracker.example");
    expect(remoteOrigin("\\\\tracker.example/p.png", BASE)).toBe("https://tracker.example");
    expect(remoteOrigin("/\\tracker.example/p.png", BASE)).toBe("https://tracker.example");
    expect(remoteOrigin("//tracker.example/p.png", "http://docs.example.com/"), "the base's scheme").toBe("http://tracker.example");
    expect(remoteOrigin("http://docs.example.com/a.png", BASE), "another scheme is another origin").toBe("http://docs.example.com");
  });

  it("is none for the page's own origin, another scheme, or text that isn't a URL", () => {
    for (const src of ["./logo.png", "logo.png", "/assets/a.png", "?v=2", "https://docs.example.com/a.png"]) expect(remoteOrigin(src, BASE), src).toBeUndefined();
    expect(remoteOrigin("app:file/3", BASE)).toBeUndefined();
    expect(remoteOrigin("data:image/png;base64,AA", BASE)).toBeUndefined();
    expect(remoteOrigin("https://[bad", BASE)).toBeUndefined();
  });
});
