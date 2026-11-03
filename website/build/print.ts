// Printing a page of the site as a PDF, as the build does (`scripts/pdf.ts`) and the dev server does when a PDF is
// asked for: what each PDF is printed from, and how Chrome prints it.
import type { Page } from "playwright-core";
import { guidePdf, specPdf } from "../src/site.ts";
import { BOOKS } from "./guide-pages.ts";
import type { SpecPage } from "./versions.ts";

/** A book's own page, which is printed into its PDF, in the book's folder. */
export const PRINT = "print/";

/** What to print: a page of the site, and the PDF it becomes, by their paths from the site's root. */
export interface PrintJob {
  page: string;
  pdf: string;
}

/** Every PDF of the site: each book's, from its own page, and each version of the spec's, from its page. */
export function printJobs(specs: readonly SpecPage[]): PrintJob[] {
  return [
    ...BOOKS.map((book) => ({ page: `guides/${book}/${PRINT}`, pdf: `guides/${book}/${guidePdf(book)}` })),
    ...specs.map(({ folder, version }) => ({ page: folder, pdf: `${folder}${specPdf(version)}` })),
  ];
}

/** The foot of each page: its number of all. Chrome's template is outside the page, so it is styled here. */
const FOOTER = '<div style="width: 100%; font: 8px system-ui, sans-serif; color: #5b6776; text-align: center"><span class="pageNumber"></span> of <span class="totalPages"></span></div>';

/** Prints the page at `url` in `page` (light), with an outline from its headings and its pages numbered. */
export async function printPdf(page: Page, url: string): Promise<Buffer> {
  await page.goto(url, { waitUntil: "load" });
  return page.pdf({
    format: "A4",
    margin: { top: "18mm", bottom: "18mm", left: "18mm", right: "18mm" },
    printBackground: true,
    outline: true,
    tagged: true,
    displayHeaderFooter: true,
    headerTemplate: "<span></span>",
    footerTemplate: FOOTER,
  });
}
