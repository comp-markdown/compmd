// The prerender: a built page's HTML with the page rendered into it, and its stylesheets inlined, so it paints from the
// HTML alone, without waiting for a script or another request.

/** What the server rendered. */
export interface Rendered {
  head: string;
  body: string;
}

/** A stylesheet the page links, by a relative path. */
const STYLESHEET = /<link rel="stylesheet"[^>]*href="((?:\.{1,2}\/)+[^"]+\.css)"[^>]*>/g;

/**
 * `html` with `rendered` in place of its `<!--head-->` and `<!--app-->` markers, and each stylesheet it links inlined:
 * `read` gives a stylesheet's text by its link, as written.
 *
 * @throws Error when a marker is missing: the page would go out without its content.
 */
export function prerendered(html: string, rendered: Rendered, read: (href: string) => string): string {
  for (const marker of ["<!--head-->", "<!--app-->"]) {
    if (!html.includes(marker)) throw new Error(`the page has no ${marker}`);
  }
  return html
    .replace(STYLESHEET, (_link, href: string) => `<style>${read(href)}</style>`)
    .replace("<!--head-->", () => rendered.head)
    .replace("<!--app-->", () => rendered.body);
}

/**
 * `html`, built for a page one folder down from the site's root, moved to a page whose path to the root is `root`:
 * its `href` and `src` paths that start `../` start with `root` instead.
 */
export function relocated(html: string, root: string): string {
  return html.replace(/\b(href|src)="\.\.\//g, (_match, attribute: string) => `${attribute}="${root}`);
}
