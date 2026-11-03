// What every page of the site links to.

/** The site's own address, which documents read elsewhere (the guides on GitHub) link to. */
export const SITE = "https://compmd.hakanserce.com/";

/** The repository on GitHub. */
export const REPOSITORY = "https://github.com/comp-markdown/compmd";

/** A book's PDF, in its folder of the guides: `compmd-authors-guide.pdf`. */
export const guidePdf = (book: string): string => `compmd-${book}-guide.pdf`;

/** A version of the spec's PDF, in its page's folder: `compmd-spec-1.0.pdf`, or `compmd-spec-draft.pdf`. */
export const specPdf = (version: string | undefined): string => `compmd-spec-${version ?? "draft"}.pdf`;
