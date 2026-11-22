// The test page `compmd-pack dev` serves (`demo.comp.md`), as a host renders it: in host mode, each wireframe in a
// strict frame of its own with the pack's built module loaded by URL, in Chrome and WebKit. Every example's wireframe draws, as
// a block and as a leaf, with nothing the strict policy refuses; the one with a mistake shows its code and is reported
// as W118 with where the mistake is; the drawing follows the page's scheme and its switch; on paper it is kept whole.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type Frame, type LaunchOptions, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4222;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

/** The demo's drawings, by their text alternatives. */
const PHONE = "The basket on a phone: two items, step 2 of 3, and a Pay now button that stays disabled until an address is chosen";
const SETTINGS = "The settings window: General, Privacy and Advanced tabs; notification toggles; a theme chosen with radio buttons; Cancel and Save";
const STEPS = "The checkout's steps as tabs: Basket, Address, Payment";
const FILES = "A file manager: a menu bar, a folder tree and a grid of files";
const SIGN_UP = "A sign-up form on a tablet: fields, a country, an avatar placeholder and a note on the email field";
const LABELS = [PHONE, SETTINGS, STEPS, FILES, SIGN_UP];

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** The frame whose content matches `selector`, once there is one. */
async function frameWith(page: Page, selector: string): Promise<Frame> {
  let found: Frame | undefined;
  await expect
    .poll(
      async () => {
        // The frames alone: the page shows each example's source, which has the sketches' code too.
        for (const frame of page.frames().filter((frame) => frame !== page.mainFrame())) if ((await frame.locator(selector).count().catch(() => 0)) > 0) found = frame;
        return found !== undefined;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  return found as Frame;
}

/** The drawing labelled `label`. */
const drawing = (label: string) => `[role="img"][aria-label="${label}"] .wf-wireframe`;

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;
  // The page in light, opened by the first test that needs it and kept: no test changes it.
  let lightOnce: Promise<{ page: Page; problems: string[] }> | undefined;
  const lightPage = () => (lightOnce ??= open("light"));

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  /** The page in `scheme` once every wireframe is drawn, with what went wrong in it: errors, and the policy's refusals. */
  async function open(scheme: "light" | "dark"): Promise<{ page: Page; problems: string[] }> {
    const page = await browser.newPage({ colorScheme: scheme });
    const problems: string[] = [];
    page.on("pageerror", (error) => problems.push(error.message));
    page.on("console", (message) => {
      if (/Content.Security.Policy|Refused to/i.test(message.text())) problems.push(message.text());
    });
    await page.goto(`http://localhost:${String(PORT)}/`);
    for (const label of LABELS) await frameWith(page, drawing(label));
    return { page, problems };
  }

  /** The texts `selector` finds in the drawing labelled `label`. */
  async function textsIn(page: Page, label: string, selector: string): Promise<string[]> {
    return (await frameWith(page, drawing(label))).locator(`${drawing(label)} ${selector}`).allTextContents();
  }

  it("draws every wireframe, as a block and as a leaf from the document's data, with nothing refused", async () => {
    const { page, problems } = await lightPage();
    expect(await textsIn(page, SETTINGS, ".wf-tab")).toEqual(["General", "Privacy", "Advanced"]);
    expect(await textsIn(page, STEPS, ".wf-tab")).toEqual(["Basket", "Address", "Payment"]);
    expect(await textsIn(page, FILES, ".wf-menu-title")).toEqual(["File", "Edit"]);
    expect(await textsIn(page, SIGN_UP, ".wf-named .wf-name")).toEqual(["chip"]);
    // Each example (`render=both`) shows its source once, above what it draws, even the one with a mistake.
    const both = page.locator(".compmd-both");
    expect(await both.count()).toBe(6);
    expect(await both.evaluateAll((all) => all.map((box) => box.querySelectorAll(":scope > pre, :scope > * > pre").length >= 1))).toEqual([true, true, true, true, true, true]);
    // In the frames, only the sketch with a mistake shows its code, as its fallback.
    const framed = await Promise.all(page.frames().slice(1).map((frame) => frame.locator("pre").count()));
    expect(framed.reduce((total, count) => total + count, 0)).toBe(1);
    expect(problems).toEqual([]);
  }, 60_000);

  it("lays the parts out with CSS: a row along, a fixed column, a grid, a phone's width", async () => {
    const { page } = await lightPage();
    const frame = await frameWith(page, drawing(FILES));
    const fixed = await frame.locator(`${drawing(FILES)} .wf-col`).first().evaluate((col) => col.getBoundingClientRect().width);
    expect(Math.round(fixed)).toBe(220);
    const cells = await frame.locator(`${drawing(FILES)} .wf-cell`).evaluateAll((all) => all.map((cell) => Math.round(cell.getBoundingClientRect().top)));
    expect(new Set(cells).size).toBe(1);
    const footer = await frameWith(page, drawing(SETTINGS));
    const [cancel, save] = await footer.locator(`${drawing(SETTINGS)} .wf-footer .wf-button`).evaluateAll((all) => all.map((button) => button.getBoundingClientRect()));
    expect(save && cancel && save.left > cancel.left && Math.abs(save.top - cancel.top) < 2).toBe(true);
  }, 60_000);

  it("numbers each annotation beside the window and at its target", async () => {
    const { page } = await lightPage();
    expect(await textsIn(page, PHONE, ".wf-notes-bottom .wf-note")).toEqual(["1Disabled until an address is chosen"]);
    expect(await textsIn(page, PHONE, ".wf-button .wf-marker")).toEqual(["1"]);
    expect(await textsIn(page, SIGN_UP, ".wf-input .wf-marker")).toEqual(["1", "2"]);
    expect(await textsIn(page, SIGN_UP, ".wf-notes-left .wf-note")).toEqual(["2Shown before the form is sent"]);
  }, 60_000);

  it("shows the code of a source with a mistake, and reports where it is as W118", async () => {
    const { page } = await lightPage();
    const frame = await frameWith(page, "pre >> text=butn");
    expect(await frame.locator("pre", { hasText: 'butn "Go"' }).count()).toBe(1);
    await expect.poll(() => page.locator("#diagnostics li").allInnerTexts(), { timeout: 30_000 }).toEqual(["W118: wireframe: line 2, column 3: no part `butn`: did you mean `button`?"]);
  }, 60_000);

  it("draws in the page's dark scheme, and back when switched", async () => {
    const scheme = async (page: Page) => (await frameWith(page, drawing(SETTINGS))).locator(drawing(SETTINGS)).evaluate((root) => [root.getAttribute("data-scheme"), getComputedStyle(root).colorScheme]);
    const dark = await open("dark");
    expect(await scheme(dark.page)).toEqual(["dark", "dark"]);
    await dark.page.selectOption("#scheme", "light");
    await expect.poll(() => scheme(dark.page), { timeout: 30_000 }).toEqual(["light", "light"]);
    await dark.page.close();
  }, 60_000);

  it("prints as it shows, each wireframe kept whole", async () => {
    const page = await browser.newPage();
    await page.goto(`http://localhost:${String(PORT)}/`);
    const frame = await frameWith(page, drawing(SETTINGS));
    await page.emulateMedia({ media: "print" });
    const printed = await frame.locator(drawing(SETTINGS)).evaluate((root) => [getComputedStyle(root).display, getComputedStyle(root).breakInside]);
    expect(printed).toEqual(["grid", "avoid"]);
    expect(await frame.locator(`${drawing(SETTINGS)} .wf-button.wf-primary`).isVisible()).toBe(true);
    await page.close();
  }, 60_000);
});
