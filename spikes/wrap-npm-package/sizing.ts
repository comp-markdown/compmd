// The prototype of docs/sizing-components.md, decision 2: `sizing/page.html` draws each sizing case twice, in the page
// and in a sandboxed frame, and this compares them in Chrome and WebKit: the use's box, what is drawn in it, that the
// frame is the box's inside, and how many times each frame reported; then again after the tall boxes of the grid rows
// are made short. It writes `results-sizing.json` and a screenshot per browser.
//
//   pnpm --dir spikes/wrap-npm-package sizing

import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { chromium, webkit, type BrowserType, type LaunchOptions, type Page } from "playwright";

const HERE = import.meta.dirname;
const HEIGHT = Number(process.argv.find((argument) => argument.startsWith("--height="))?.slice("--height=".length) ?? 7000);
/** The default run's files; a run at another height prints only. */
const KEEP = HEIGHT === 7000;
const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

interface Size {
  w: number;
  h: number;
}

interface Use {
  id: string;
  title: string;
  kind: string;
  plain: { box: Size; element?: Size; drawing?: Size; aspect?: string | null };
  framed: { box: Size; frame: Size; inside: Size; natural?: Size; element?: Size; drawing?: Size; aspect?: string | null; clipped?: boolean; messages?: number };
}

interface Checked extends Use {
  /** What differs between the page and the frame; empty when the frame is invisible to layout. */
  differs: string[];
}

const same = (a: Size | undefined, b: Size | undefined) => a !== undefined && b !== undefined && Math.abs(a.w - b.w) <= 1 && Math.abs(a.h - b.h) <= 1;
const text = (size: Size | undefined) => (size ? `${String(size.w)}×${String(size.h)}` : "none");

function check(use: Use): Checked {
  const differs: string[] = [];
  if (!same(use.plain.box, use.framed.box)) differs.push(`box ${text(use.plain.box)} in the page, ${text(use.framed.box)} framed`);
  if (!same(use.plain.drawing, use.framed.drawing)) differs.push(`drawing ${text(use.plain.drawing)} in the page, ${text(use.framed.drawing)} framed`);
  if (!same(use.framed.frame, use.framed.inside)) differs.push(`the frame is ${text(use.framed.frame)}, its box's inside ${text(use.framed.inside)}`);
  return { ...use, differs };
}

/** The page's uses once two snapshots a moment apart are the same. */
async function settled(page: Page): Promise<Use[]> {
  let last = "";
  let before: Use[] = [];
  for (let tries = 0; tries < 40; tries += 1) {
    await page.waitForTimeout(250);
    const uses = (await page.evaluate(() => (window as unknown as { snapshot(): unknown }).snapshot())) as Use[];
    const now = JSON.stringify(uses);
    if (now === last) return uses;
    last = now;
    before = uses;
  }
  await page.waitForTimeout(250);
  const after = (await page.evaluate(() => (window as unknown as { snapshot(): unknown }).snapshot())) as Use[];
  const moving = after.filter((use, index) => JSON.stringify(use) !== JSON.stringify(before[index])).map((use) => `${use.id} (${String(use.framed.messages)} reports, natural ${text(use.framed.natural)}, frame ${text(use.framed.frame)})`);
  throw new Error(`the page never settled: still reporting: ${moving.join("; ")}`);
}

function print(step: string, uses: Checked[]) {
  console.log(`  ${step}: ${String(uses.filter((use) => use.differs.length === 0).length)} of ${String(uses.length)} uses the same in the page and framed`);
  for (const use of uses) {
    const reports = use.framed.messages === undefined ? "" : `, ${String(use.framed.messages)} reports`;
    console.log(`    ${use.id} (${use.kind}): box ${text(use.framed.box)}, drawing ${text(use.framed.drawing)}, natural ${text(use.framed.natural)}${reports}${use.framed.clipped ? ", clipped" : ""}${use.differs.length > 0 ? `; DIFFERS: ${use.differs.join("; ")}` : ""}`);
  }
}

const results: Record<string, { drawn: Checked[]; shrunk: Checked[] }> = {};
for (const [name, type, options] of BROWSERS) {
  const browser = await type.launch({ headless: true, ...options });
  // By default tall enough for every case to be on screen; `--height=900` leaves most frames off screen, which a
  // browser renders less eagerly.
  const page = await browser.newPage({ viewport: { width: 1100, height: HEIGHT } });
  await page.goto(pathToFileURL(join(HERE, "sizing", "page.html")).href);
  const drawn = (await settled(page)).map(check);
  if (KEEP) await page.screenshot({ path: join(HERE, "screenshots", `sizing-${name.toLowerCase()}.png`), fullPage: true });
  await page.evaluate(() => {
    (window as unknown as { shrink(): void }).shrink();
  });
  const shrunk = (await settled(page)).filter((use) => use.id.startsWith("grid-")).map(check);
  results[name] = { drawn, shrunk };
  console.log(name);
  print("as drawn", drawn);
  print("the grid rows, once their tall box is short", shrunk);
  await browser.close();
}
if (KEEP) writeFileSync(join(HERE, "results-sizing.json"), `${JSON.stringify(results, null, 2)}\n`);
