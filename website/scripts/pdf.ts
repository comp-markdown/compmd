// Prints the PDFs into `dist/`, after the prerender (`build/print.ts`): each book of the guides from its own page,
// which is then removed, and each version of the spec, with its companion, from its page. Chrome (the installed one,
// through Playwright) prints them.
import { execFileSync } from "node:child_process";
import { rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { chromium } from "playwright-core";
import { BOOKS } from "../build/guide-pages.ts";
import { PRINT, printJobs, printPdf } from "../build/print.ts";
import { serveFolder } from "../build/static.ts";
import { specPages } from "../build/versions.ts";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");
const PORT = 4194;

const tags = execFileSync("git", ["tag", "--list", "spec-v*"], { cwd: REPO, encoding: "utf8" }).split("\n").filter(Boolean);
const jobs = printJobs(specPages(tags));

const serve = serveFolder("/", DIST);
const server = createServer((request, response) => {
  serve(request, response, () => {
    response.statusCode = 404;
    response.end();
  });
});
await new Promise<void>((resolve) => server.listen(PORT, resolve));
const browser = await chromium.launch({ channel: "chrome" });
try {
  const page = await browser.newPage({ colorScheme: "light" });
  for (const job of jobs) writeFileSync(join(DIST, job.pdf), await printPdf(page, `http://localhost:${String(PORT)}/${job.page}`));
} finally {
  await browser.close();
  server.close();
}
for (const book of BOOKS) rmSync(join(DIST, `guides/${book}/${PRINT}`), { recursive: true, force: true });
console.log(`Printed ${jobs.map(({ pdf }) => pdf).join(", ")}`);
