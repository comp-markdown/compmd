+++
id = "01a20827-e5f3-743d-9b34-9f3bcc923dc8"
number = 72
title = "Installing and using cblo: README, website README, Authors Guide chapter 19"
status = "approved"
tasks = ["CMD-382"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T11:26:02Z
updated = 2026-11-23T15:14:43Z
+++

How to get and use `cblo`, written down in three places.

- **`npm-packages/blog/README.md`**: a new *Install* section: `npm create @compmd/blog`, `npm install -g @compmd/blog`, and `npx -p @compmd/blog cblo …`.
- **`website/README.md`**: in this repository, `cblo new` from `website/blog/` with the workspace's own copy (`../node_modules/.bin/cblo`), which needs no install. This stays out of the published README.
- **The Author's Guide**: a new *Part VI. Publishing* with chapter 19, *A blog*: getting the tool, a blog's folder, `blog.config.json`, a post's frontmatter, `cblo new`/`dev`/`build`, publishing `dist/`, packs, pictures from other sites, the look, and comments. The appendices are now 20 and 21, with every link to them updated (their slugs, and so their addresses on the site, don't change).

## Tests
- The chapter's example blog, `guides/authors/examples/blog/`, is shown and processed like every chapter's example.
- `guides/test/blog.test.ts` builds it with `@compmd/blog`'s `buildSite`: no warnings, and the post, the list, the tag page and the feed are written (`@compmd/blog` is now a dev dependency of the guides).
- `cargo make site` and `cargo make ci` pass. The first `ci` run timed out in the Mermaid pack's browser test under load; it passed alone, and the second full run passed.

## Worth checking
- Both of these are written as the tool behaves today:
  - **No `compmd.config.json`, no packs.** A blog with no `compmd.config.json` gives its posts no packs at all, installed ones included (`packsOf`, `npm-packages/blog/src/live.ts`). The chapter says to add one, and that `{}` is enough. In a Vite app, every installed pack is available without the file. Should the blog do the same?
  - **The title shows twice.** `cblo new` writes the title as a `# heading` too, but a post's page already shows the title as its `<h1>`. The site's posts have no heading of their own. The chapter doesn't mention the heading.
