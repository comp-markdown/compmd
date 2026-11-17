// The pictures of what a book's examples render, for its PDF: a reader on paper sees the page each example makes. Each
// is taken as the PDF is printed, from the example's "Try it" link, in the playground the build serves beside the book.
import type { Page } from "playwright-core";
import { SITE } from "../src/site.ts";

/** A chapter's link to an example in the playground, as a book's page has it: to the site, in full. */
const TRY_IT = `.chapter a[href^="${SITE}playground/?doc="]`;

/** How much finer than the screen a picture is taken: at a book's column, about 300 dots an inch. */
const SCALE = 3;

/**
 * A picture of an example: its link, the PNG as a data URL, its size in CSS pixels (so its box is known before the
 * image decodes, as Paged.js measures the page), and the page's text, for its `alt`.
 */
interface Preview {
  href: string;
  src: string;
  width: number;
  height: number;
  alt: string;
}

/**
 * Puts before each "Try it" link's paragraph in the book's page `page` a picture of what its example renders: the
 * playground at the same address on `origin` (light), its rendered page alone.
 *
 * @throws Error when an example's page doesn't render: the book would go out without its picture.
 */
export async function addPreviews(page: Page, origin: string): Promise<void> {
  const hrefs = [...new Set(await page.locator(TRY_IT).evaluateAll((links) => links.map((link) => (link as HTMLAnchorElement).href)))];
  if (hrefs.length === 0) return;
  const browser = page.context().browser();
  if (!browser) throw new Error("the book's page has no browser to take its examples' pictures in");
  const context = await browser.newContext({ viewport: { width: 1200, height: 2400 }, deviceScaleFactor: SCALE, colorScheme: "light" });
  const previews: Preview[] = [];
  try {
    const playground = await context.newPage();
    for (const href of hrefs) {
      const { pathname, search } = new URL(href);
      await playground.goto(new URL(`${pathname}${search}`, origin).href, { waitUntil: "load" });
      const rendered = playground.frameLocator("iframe[title='The rendered page']").locator("#page");
      await rendered.locator("> *").first().waitFor({ timeout: 15000 });
      // Drawn: no use still pending (a pack still on its way from the registry).
      await rendered.locator('[aria-busy="true"]').first().waitFor({ state: "detached", timeout: 30000 });
      // Then whatever a drawn component still loads (a font, an image), for as long as it takes to start and finish:
      // a registry that keeps a connection open doesn't hold the book back.
      await playground.waitForLoadState("networkidle", { timeout: 5000 }).catch(() => undefined);
      const png = await rendered.screenshot({ animations: "disabled" });
      const box = await rendered.boundingBox();
      if (!box) throw new Error(`the example at ${href} rendered nothing to picture`);
      const alt = (await rendered.innerText()).replace(/\s+/g, " ").trim();
      previews.push({ href, src: `data:image/png;base64,${png.toString("base64")}`, width: Math.round(box.width), height: Math.round(box.height), alt });
    }
  } finally {
    await context.close();
  }
  await page.evaluate(
    ({ previews, selector }) => {
      for (const link of document.querySelectorAll<HTMLAnchorElement>(selector)) {
        const preview = previews.find(({ href }) => href === link.href);
        const block = link.closest("p");
        if (!preview || !block) continue;
        const figure = document.createElement("figure");
        figure.className = "preview";
        const image = document.createElement("img");
        image.src = preview.src;
        image.width = preview.width;
        image.height = preview.height;
        image.alt = preview.alt;
        const caption = document.createElement("figcaption");
        caption.textContent = "What it renders";
        figure.append(image, caption);
        block.before(figure);
      }
    },
    { previews, selector: TRY_IT },
  );
}
