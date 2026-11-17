// A blog's layout (docs/blog-tool.md, decision 6): an HTML page with `<!-- content -->` where each page goes, its own
// `layout.html` or the default. Each page's head (its title, description, feed, stylesheets) goes before `</head>`.

import { BlogError, type BlogConfig } from "./config.ts";
import { escape } from "./html.ts";

/** Where a page's content goes in the layout. */
export const CONTENT = "<!-- content -->";
/** A blog's own layout's file. */
export const LAYOUT = "layout.html";

/** The default layout: the blog's title, linked to its list, above each page, and a line below. */
export function defaultLayout(config: BlogConfig): string {
  const home = new URL(config.url).pathname;
  return `<!doctype html>
<html lang="${escape(config.lang)}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
<header class="blog-header"><a href="${escape(home)}">${escape(config.title)}</a></header>
<main class="blog-main">
${CONTENT}
</main>
<footer class="blog-footer">Written in <a href="https://compmd.dev/">CompMD</a>.</footer>
</body>
</html>
`;
}

/** `layout` with a page's `head` before `</head>` and its `content` where it says. @throws BlogError when it lacks either. */
export function applyLayout(layout: string, { head, content }: { head: string; content: string }): string {
  if (!layout.includes(CONTENT)) throw new BlogError(`${LAYOUT} has no \`${CONTENT}\` for each page's content`);
  if (!layout.includes("</head>")) throw new BlogError(`${LAYOUT} has no \`</head>\` for each page's head`);
  return layout.replace("</head>", () => `${head}\n</head>`).replace(CONTENT, () => content);
}
