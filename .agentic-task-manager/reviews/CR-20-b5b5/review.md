+++
id = "01a1f526-12a2-7353-a254-f28756dcb5b5"
number = 20
title = "The blog at /blog/, on @compmd/blog, in the site layout"
status = "approved"
tasks = ["CMD-13"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T18:51:15Z
updated = 2026-11-19T20:58:44Z
+++

The blog at `/blog/`, built with `@compmd/blog` in a layout made from the site's header and footer (decision 2 changed as agreed: `buildSite` in the site's build, and the dev server building it for each page, instead of `cblo build` and `cblo dev`).

What to check:

- **The first post,** `website/blog/posts/the-compmd-blog.comp.md`: what the blog is for, the three posts to come, and how posts are written. Its wording is yours to approve before it is published.
- **`@compmd/blog`:** `buildSite` takes a `layout` in place of the blog's `layout.html`; `CONTENT_MARKER`, `fileFor` and `sendBlogFile` are exported, so a site makes its layout and serves the blog from its own dev server. `serve` uses `sendBlogFile`.
- **The site:** `src/BlogLayout.svelte` and "Blog" in the header; `build/blog.ts`, the build and the layout; the prerender writing `dist/blog/`, its posts in search and its pages in the sitemap; the dev server's `/blog/`; `blog/theme.css` mapping the tool's colours onto the site's tokens.
- **Comments:** giscus with the repository's "Blog" category (its ids in `blog.config.json`), loaded once a reader scrolls to them. `pnpm check` answers giscus's script itself, so it never reaches giscus.
- **The checks:** `pnpm check` covers the blog's pages (light and dark, desktop and phone, first paint, links), its feed, search, the comments loading late, and a post's size (11.5 KB gzipped, budget 100 KB). The dev server's test paints the blog's list and post in Chrome and WebKit. The Pages workflow runs when `npm-packages/blog/` changes.

Left out: the site's pack (`content/app/`) in posts. `@compmd/blog`'s page script doesn't compile a trusted Svelte pack (`Stars.svelte` failed to parse), so the blog has no `compmd.config.json` yet; a follow-up for the blog tool.
