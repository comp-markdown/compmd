// Makes the site's card, `dist/og.png` (`build/card.ts`), after the prerender: the card's page, shot in Chrome (the
// installed one, through Playwright), as the PDFs are printed.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { chromium } from "playwright-core";
import { CARD, CARD_FILE, cardHtml } from "../build/card.ts";

const REPO = join(import.meta.dirname, "../..");
const DIST = join(import.meta.dirname, "../dist");

const browser = await chromium.launch({ channel: "chrome" });
try {
  const page = await browser.newPage({ viewport: CARD, colorScheme: "light" });
  await page.setContent(cardHtml(readFileSync(join(REPO, "logo/compmd-light.svg"), "utf8")), { waitUntil: "load" });
  await page.screenshot({ path: join(DIST, CARD_FILE), type: "png" });
} finally {
  await browser.close();
}
console.log(`Made ${CARD_FILE}`);
