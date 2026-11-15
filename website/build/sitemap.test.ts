import { describe, expect, it } from "vitest";
import { ROBOTS, sitemapOf } from "./sitemap.ts";

describe("sitemapOf", () => {
  it("lists each page once, at the site's address, in order", () => {
    const sitemap = sitemapOf(["", "spec/", "guides/", "spec/"]);
    expect([...sitemap.matchAll(/<loc>([^<]*)<\/loc>/g)].map(([, loc]) => loc)).toEqual([
      "https://compmd.dev/",
      "https://compmd.dev/spec/",
      "https://compmd.dev/guides/",
    ]);
    expect(sitemap).toMatch(/^<\?xml version="1.0" encoding="UTF-8"\?>\n<urlset xmlns="http:\/\/www.sitemaps.org\/schemas\/sitemap\/0.9">/);
  });

  it("escapes what XML must", () => {
    expect(sitemapOf(["a&b/"])).toContain("<loc>https://compmd.dev/a&amp;b/</loc>");
  });
});

describe("ROBOTS", () => {
  it("lets every crawler in and points at the sitemap", () => {
    expect(ROBOTS).toBe("User-agent: *\nAllow: /\n\nSitemap: https://compmd.dev/sitemap.xml\n");
  });
});
