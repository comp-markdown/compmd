// @vitest-environment node
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { fileOf, serveFolder } from "./static.ts";

const root = mkdtempSync(join(tmpdir(), "compmd-static-"));
mkdirSync(join(root, "spec"));
writeFileSync(join(root, "index.html"), "<p>home</p>");
writeFileSync(join(root, "spec", "index.html"), "<p>spec</p>");
writeFileSync(join(root, "a.js"), "export {};");

describe("fileOf", () => {
  it("finds a file, and a folder's index.html", async () => {
    expect(await fileOf(root, "/a.js")).toBe(join(root, "a.js"));
    expect(await fileOf(root, "/spec/")).toBe(join(root, "spec", "index.html"));
    expect(await fileOf(root, "/")).toBe(join(root, "index.html"));
  });

  it("finds nothing outside the root, or missing", async () => {
    expect(await fileOf(root, "/../etc/passwd")).toBeUndefined();
    expect(await fileOf(root, "/%2e%2e/x")).toBeUndefined();
    expect(await fileOf(root, "/missing.js")).toBeUndefined();
  });
});

describe("serveFolder", () => {
  let server: Server;
  let base = "";
  beforeAll(async () => {
    const serve = serveFolder("/site", root);
    server = createServer((request, response) => {
      serve(request, response, () => {
        response.statusCode = 404;
        response.end();
      });
    });
    await new Promise<void>((resolve) => server.listen(0, resolve));
    const address = server.address();
    base = `http://localhost:${String(typeof address === "object" && address ? address.port : 0)}`;
  });
  afterAll(() => {
    server.close();
  });

  it("serves files under its prefix with their type, open to any origin", async () => {
    const response = await fetch(`${base}/site/a.js`);
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/javascript; charset=utf-8");
    expect(response.headers.get("access-control-allow-origin")).toBe("*");
    expect(await (await fetch(`${base}/site/spec/`)).text()).toBe("<p>spec</p>");
  });

  it("passes on other paths and missing files", async () => {
    expect((await fetch(`${base}/other/a.js`)).status).toBe(404);
    expect((await fetch(`${base}/site/missing`)).status).toBe(404);
  });
});
