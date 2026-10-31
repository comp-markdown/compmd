// The guide's pages, rendered: the contents at `guide/`, each chapter written at `guide/<slug>/`, each with the book
// beside it and the chapters before and after it. The prerender writes them; the dev server renders one per request.
import type { ComponentProps } from "svelte";
import type Guide from "../src/Guide.svelte";
import { bookOf, guideLink, writtenOf } from "./guide.ts";
import { renderMarkdown } from "./spec.ts";

/** A page of the guide: its folder in the site, its path to the site's root, and what its component is given. */
export interface GuidePage {
  folder: string;
  root: string;
  props: ComponentProps<typeof Guide>;
}

/**
 * The guide's pages, from its files: `read` gives a file of `guide/` by its name. `blob` is the repository's file URL,
 * ending in `/`.
 */
export async function guidePages(read: (file: string) => string, blob: string): Promise<GuidePage[]> {
  const contents = read("README.md");
  const book = bookOf(contents);
  const chapters = writtenOf(book);
  const source = { blob: `${blob}guide/`, path: "" };
  const page = async (file: string, up: string, root: string): Promise<Omit<GuidePage["props"], "current" | "previous" | "next" | "title">> => {
    const { html, toc } = await renderMarkdown(read(file), { ...source, path: file }, { rules: false, link: (href) => guideLink(href, up) });
    return { html, toc, root, book, source: `${blob}guide/${file}` };
  };
  const first = chapters[0];
  const start = first && { title: `${String(first.number)}. ${first.title}`, href: `${first.slug}/` };
  const pages: GuidePage[] = [{ folder: "guide/", root: "../", props: { ...(await page("README.md", "", "../")), title: "CompMD Developer Guide", next: start } }];
  for (const [index, chapter] of chapters.entries()) {
    const link = (other: (typeof chapters)[number] | undefined) => other && { title: `${String(other.number)}. ${other.title}`, href: `../${other.slug}/` };
    const previous = index === 0 ? { title: "Contents", href: "../" } : link(chapters[index - 1]);
    pages.push({
      folder: `guide/${chapter.slug}/`,
      root: "../../",
      props: { ...(await page(chapter.file, "../", "../../")), current: chapter.slug, title: `${String(chapter.number)}. ${chapter.title}`, previous, next: link(chapters[index + 1]) },
    });
  }
  return pages;
}
