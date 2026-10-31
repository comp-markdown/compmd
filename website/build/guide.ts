// The CompMD Developer Guide as pages of the site (website/docs/guide.md): its book, its parts and chapters, read from
// its contents page (`guide/README.md`), and where its links go: a chapter to its page, `/guide/<slug>/`, anything
// else to its file on GitHub.

/** A chapter of the guide; `file` and `slug` are absent for one not written yet. */
export interface Chapter {
  number: number;
  title: string;
  file?: string;
  slug?: string;
}

/** A part of the guide, its chapters in order; `later` when none is written yet. */
export interface Part {
  title: string;
  chapters: Chapter[];
  later: boolean;
}

/** A chapter's page on the site, from its file's name: `02-quick-start.md` is `quick-start`. */
export const slugOf = (file: string): string => file.replace(/^\d+-/, "").replace(/\.md$/, "");

/**
 * The book's parts and chapters, from the contents page's `## Contents`: a part is a line in bold, `**Part I. …**`,
 * with `*(to come)*` after it when none of its chapters is written; a chapter is a numbered line, linked to its file
 * when it is written.
 */
export function bookOf(contents: string): Part[] {
  const section = contents.split(/^## Contents$/m)[1]?.split(/^## /m)[0] ?? "";
  const parts: Part[] = [];
  for (const line of section.split("\n")) {
    const part = /^\*\*(Part [^*]+)\*\*(.*)$/.exec(line);
    if (part) {
      parts.push({ title: part[1] ?? "", chapters: [], later: (part[2] ?? "").includes("to come") });
      continue;
    }
    const chapter = /^(\d+)\. (?:\[([^\]]+)\]\(([^)]+)\)|(.+))$/.exec(line);
    if (!chapter || parts.length === 0) continue;
    const [, number = "0", linked, file, plain] = chapter;
    const entry: Chapter = { number: Number(number), title: linked ?? plain ?? "" };
    if (file !== undefined) Object.assign(entry, { file, slug: slugOf(file) });
    parts.at(-1)?.chapters.push(entry);
  }
  return parts;
}

/** The chapters written, in order. */
export const writtenOf = (book: readonly Part[]): Required<Chapter>[] =>
  book.flatMap((part) => part.chapters).filter((chapter): chapter is Required<Chapter> => chapter.file !== undefined);

/**
 * Where a relative link of the guide goes on the site, from a page whose path to the guide's own page is `up` (`""`
 * for `/guide/`, `"../"` for a chapter): the contents to `/guide/`, a chapter to its page; anything else is
 * `undefined`, for its file on GitHub.
 */
export function guideLink(href: string, up: string): string | undefined {
  const [path = "", hash] = href.split("#");
  const anchor = hash === undefined ? "" : `#${hash}`;
  if (path === "README.md" || path === "./README.md") return `${up || "./"}${anchor}`;
  if (/^(\.\/)?\d+-[^/]+\.md$/.test(path)) return `${up}${slugOf(path.replace(/^\.\//, ""))}/${anchor}`;
  return undefined;
}
