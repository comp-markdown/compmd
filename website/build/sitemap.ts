// What the site tells search engines (`scripts/prerender.ts` writes them): `sitemap.xml`, every page the build makes,
// and `robots.txt`, which lets every crawler in and points at the sitemap.
import { SITE } from "../src/site.ts";

const escaped = (text: string): string => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

/**
 * The sitemap of the pages at `folders`, each a folder from the site's root (`""` for the landing page, `spec/`),
 * once each, in order.
 *
 * @example
 * sitemapOf(["", "spec/"]);
 * // <?xml …?><urlset …><url><loc>https://compmd.hakanserce.com/</loc></url><url><loc>…/spec/</loc></url></urlset>
 */
export function sitemapOf(folders: readonly string[]): string {
  const urls = [...new Set(folders)].map((folder) => `  <url><loc>${escaped(`${SITE}${folder}`)}</loc></url>`);
  return `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.join("\n")}\n</urlset>\n`;
}

/** The site's `robots.txt`: every crawler welcome, and where the sitemap is. */
export const ROBOTS = `User-agent: *\nAllow: /\n\nSitemap: ${SITE}sitemap.xml\n`;
