// The Developer's Guide's Part II, documents in your app, against what its chapters say: the check script run in Node
// on its document and on a broken copy, and the live editor, the orders page and the styled page built with Vite and
// opened in Chrome and WebKit.
import { execFileSync } from "node:child_process";
import { cp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { execPath } from "node:process";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Page } from "playwright";
import { build, preview, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "@compmd/vite";

const EXAMPLES = join(import.meta.dirname, "../developers/examples");
const OUT = join(import.meta.dirname, ".out/part-2");

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

/** Runs `node check.ts <folder> <document>` in `folder`: what it printed, and its exit code. */
function runCheck(folder: string, document: string): { output: string; status: number } {
  try {
    return { output: execFileSync(execPath, [join(folder, "check.ts"), folder, document], { encoding: "utf8", stdio: "pipe" }), status: 0 };
  } catch (error) {
    const failed = error as { stdout: string; status: number };
    return { output: failed.stdout, status: failed.status };
  }
}

describe("processing documents: the check", () => {
  const example = join(EXAMPLES, "check-docs");
  const broken = join(OUT, "check-docs");

  beforeAll(async () => {
    await rm(broken, { recursive: true, force: true });
    await cp(example, broken, { recursive: true });
    const text = await readFile(join(broken, "quarter.comp.md"), "utf8");
    await writeFile(join(broken, "quarter.comp.md"), text.replace("border=subtle", "border=subtl"));
  });

  it("finds nothing wrong with the chapter's document", () => {
    expect(runCheck(example, "quarter.comp.md")).toEqual({ output: "quarter.comp.md: no diagnostics\n", status: 0 });
  });

  it("prints a typo's diagnostic with its fix, and fails, as the chapter says", () => {
    expect(runCheck(broken, "quarter.comp.md")).toEqual({
      output:
        "quarter.comp.md:15:43 F009 `border` takes `none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?\n  fix: write `subtle` at 15:43-48\n",
      status: 1,
    });
  });
});

/**
 * Builds the example `name` into `.out/part-2/<name>-<port>` and serves it on `port`, with a Vite cache of its own: the
 * plugin empties the frames' folder in it, which builds running at once would share.
 */
async function serve(name: string, port: number): Promise<PreviewServer> {
  const out = join(OUT, `${name}-${String(port)}`);
  const config = { root: join(EXAMPLES, name), configFile: false as const, logLevel: "warn" as const, cacheDir: join(out, ".vite"), plugins: [compmd()], build: { outDir: join(out, "site"), emptyOutDir: true, target: "es2024" } };
  await build(config);
  return preview({ ...config, preview: { port, strictPort: true } });
}

/** Runs `check` in a new page of each browser, at `port`'s address with `search`, and expects no page errors. */
function inBrowsers(name: string, port: number, checks: Record<string, (page: Page) => Promise<void>>, search = ""): void {
  let server: PreviewServer;

  beforeAll(async () => {
    server = await serve(name, port);
  });

  afterAll(async () => {
    await server.close();
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    for (const [title, check] of Object.entries(checks)) {
      it(title, async () => {
        const page = await browser.newPage();
        const errors: string[] = [];
        page.on("pageerror", (error) => errors.push(error.message));
        await page.goto(`http://localhost:${String(port)}/${search}`);
        await check(page);
        expect(errors).toEqual([]);
        await page.close();
      });
    }
  });
}

const textOf = (page: Page, frame = "iframe.compmd-page-frame") => page.frameLocator(frame).locator("body").innerText();

describe("live editing: the editor", () => {
  inBrowsers("live-editor", 4210, {
    "follows the text as it is typed, and an input the reader sets": async (page) => {
      await expect.poll(() => textOf(page), { timeout: 20_000 }).toContain("Dinner for 4");
      const editor = page.locator("#text");
      await editor.focus();
      await page.keyboard.press("ControlOrMeta+End");
      await page.keyboard.type("\n\nAnd a salad.");
      await expect.poll(() => textOf(page)).toContain("And a salad.");
      await page.frameLocator("iframe.compmd-page-frame").locator("input[type=number]").fill("6");
      await expect.poll(() => textOf(page)).toContain("Dinner for 6");
      expect(await textOf(page)).toContain("900 g of rice");
      expect(await editor.inputValue()).toContain("And a salad.");
    },
    "lists a typo's diagnostic, and its fix mends the text": async (page) => {
      await expect.poll(() => textOf(page), { timeout: 20_000 }).toContain("Dinner for 4");
      const editor = page.locator("#text");
      await editor.fill((await editor.inputValue()).replace("border=subtle", "border=subtl"));
      await expect.poll(() => page.locator("#problems").innerText()).toContain("F009");
      await page.locator("#problems button").click();
      expect(await editor.inputValue()).toContain("border=subtle");
      await expect.poll(() => page.locator("#problems li").count()).toBe(0);
    },
    "says what a param takes at the caret": async (page) => {
      await expect.poll(() => textOf(page), { timeout: 20_000 }).toContain("Dinner for 4");
      const editor = page.locator("#text");
      const at = (await editor.inputValue()).indexOf("border=") + 2;
      await editor.evaluate((element: HTMLTextAreaElement, offset) => {
        element.focus();
        element.setSelectionRange(offset, offset);
      }, at);
      await page.keyboard.press("ArrowRight");
      await expect.poll(() => page.locator("#hint").innerText()).toContain("border takes `none`, `subtle`, `strong`");
    },
  });
});

describe("data and inputs: the orders", () => {
  const listed = async (page: Page) => page.frameLocator("iframe.compmd-page-frame").locator("li").allInnerTexts();
  inBrowsers("orders", 4211, {
    "shows the host's open orders and the delivery file, then the shipped ones, kept in the address": async (page) => {
      await expect.poll(() => listed(page), { timeout: 20_000 }).toEqual(["A-1041, Ada: 42.5 EUR", "A-1042, Grace: 18 EUR", "Europe: 2 days", "Elsewhere: 6 days"]);
      expect(await textOf(page)).toContain("2 orders in all.");
      await page.frameLocator("iframe.compmd-page-frame").locator("select").selectOption("shipped");
      await expect.poll(() => listed(page)).toEqual(["A-1039, Edsger: 7.25 EUR", "Europe: 2 days", "Elsewhere: 6 days"]);
      expect(new URL(page.url()).search).toBe("?status=shipped");
    },
  });
});

describe("data and inputs: values from the address", () => {
  const listed = async (page: Page) => page.frameLocator("iframe.compmd-page-frame").locator("li").allInnerTexts();
  inBrowsers(
    "orders",
    4212,
    {
      "uses the default for a value the input doesn't take": async (page) => {
        await expect.poll(() => listed(page), { timeout: 20_000 }).toContain("A-1041, Ada: 42.5 EUR");
      },
    },
    "?status=lost",
  );
});

describe("rendering and styling: the styled page", () => {
  const frame = (page: Page) => page.frameLocator("iframe.compmd-page-frame");
  inBrowsers("styled-page", 4213, {
    "styles the page with the app's stylesheet and tokens, and follows the app's scheme": async (page) => {
      await expect.poll(() => textOf(page), { timeout: 20_000 }).toContain("Welcome aboard");
      const heading = frame(page).locator("h1");
      await expect.poll(() => heading.evaluate((element) => getComputedStyle(element).color)).toBe("rgb(11, 114, 133)");
      expect(await heading.evaluate((element) => getComputedStyle(element).fontFamily)).toContain("Georgia");
      expect(await frame(page).locator("html").evaluate((element) => getComputedStyle(element).colorScheme)).not.toBe("dark");
      await page.locator("#scheme").click();
      await expect.poll(() => frame(page).locator("html").evaluate((element) => getComputedStyle(element).colorScheme)).toBe("dark");
    },
    "shows a link's address before it opens": async (page) => {
      await expect.poll(() => textOf(page), { timeout: 20_000 }).toContain("Welcome aboard");
      // Once the page has settled, its stylesheet applied among the rest, the link stays where the click goes.
      await page.locator("#page[data-compmd-settled]").waitFor({ timeout: 20_000 });
      await frame(page).locator("a", { hasText: "the spec" }).click();
      // The click crosses from the frame to the page as a message, which a busy machine delivers late.
      await expect.poll(() => page.locator("#link output").innerText(), { timeout: 10_000 }).toBe("https://compmd.hakanserce.com/spec/");
      await page.locator("#link button[value=cancel]").click();
      expect(await page.locator("#link").evaluate((element: HTMLDialogElement) => element.open)).toBe(false);
    },
    "shows what a plain renderer shows": async (page) => {
      await expect.poll(() => textOf(page, "#plain"), { timeout: 20_000 }).toContain("::: box {padding=m border-radius=m background=accent}");
      expect(await textOf(page, "#plain")).toContain("Your first week, from :value[the docs team]{of=(md.doc.team)}.");
    },
  });
});
