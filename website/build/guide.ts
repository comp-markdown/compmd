// The CompMD guides as pages of the site (website/docs/guide.md): each book's parts and chapters, read from its
// contents page (`guides/<book>/README.md`), and where their links go: a chapter to its page,
// `/guides/<book>/<slug>/`, anything else to its file on GitHub.

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
 * The book's parts and chapters, from the contents page's `## Contents`: a part is a line in bold, `**Part I. …**` or
 * `**Appendices**`, with `*(to come)*` after it when none of its chapters is written; a chapter (or an appendix,
 * numbered on from the chapters) is a numbered line, linked to its file when it is written.
 */
export function bookOf(contents: string): Part[] {
  const section = contents.split(/^## Contents$/m)[1]?.split(/^## /m)[0] ?? "";
  const parts: Part[] = [];
  for (const line of section.split("\n")) {
    const part = /^\*\*(Part [^*]+|Appendices)\*\*(.*)$/.exec(line);
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

/** A guide's page on the site for a file of its folder: its contents (`README.md`) or a chapter; `undefined` otherwise. */
function pageOf(file: string): string | undefined {
  if (file === "README.md") return "";
  return /^\d+-[^/]+\.md$/.test(file) ? `${slugOf(file)}/` : undefined;
}

/**
 * Where a relative link of a guide goes on the site, from a page whose path to its book's page is `up` (`""` for the
 * book's contents, `"../"` for a chapter; for the shelf, `undefined`): its book's contents and chapters to their pages,
 * another book's (`../authors/…`) to theirs, and from the shelf, each book's (`authors/…`). Anything else is
 * `undefined`, for its file on GitHub.
 */
export function guideLink(href: string, up: string | undefined): string | undefined {
  const [path = "", hash] = href.replace(/^\.\//, "").split("#");
  const anchor = hash === undefined ? "" : `#${hash}`;
  const at = (page: string | undefined, prefix: string) => (page === undefined ? undefined : `${prefix}${page}${anchor}` || `./${anchor}`);
  if (up === undefined) {
    const [book, file = ""] = path.split("/");
    return book !== undefined && /^[a-z-]+$/.test(book) ? at(pageOf(file), `${book}/`) : undefined;
  }
  const other = /^\.\.\/([a-z-]+)\/([^/]+)$/.exec(path);
  if (other) return at(pageOf(other[2] ?? ""), `${up}../${other[1] ?? ""}/`);
  return at(pageOf(path), up);
}
