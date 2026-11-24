import { describe, expect, it } from "vitest";
import { BlogError } from "./config.ts";
import { SiteFiles } from "./site-files.ts";

describe("SiteFiles", () => {
  it("keeps each file by its path, and refuses a second one at a path, naming both owners", () => {
    const site = new SiteFiles();
    site.set("tags/c/index.html", "<h1>C</h1>", 'the page of the tag "C"');
    site.set("feed.xml", "<feed/>", "the feed");
    expect([...site.files.keys()]).toEqual(["tags/c/index.html", "feed.xml"]);
    expect(() => {
      site.set("tags/c/index.html", "<h1>C++</h1>", 'the page of the tag "C++"');
    }).toThrow(new BlogError('tags/c/index.html would be written twice: by the page of the tag "C", and by the page of the tag "C++"; rename one of them'));
  });
});
