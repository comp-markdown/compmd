# @compmd/blog

A blog in [CompMD](https://compmd.dev/): a folder of CompMD documents made a static site, with a page per post, tags,
an Atom feed and comments. Every post is rendered by CompMD when the blog is built, so it reads as a page before any
script runs. The design is [docs/blog-tool.md](https://github.com/comp-markdown/compmd/blob/main/docs/blog-tool.md).

**Status: in development.** A post's host-language components, its own or from the blog's packs (`compmd.config.json`,
as any app's, read by [`@compmd/vite`](https://github.com/comp-markdown/compmd/tree/main/vite)), show their fallback in
the page as it was built, and are drawn live once its script loads, through the sandbox; a post without them loads no
script. Its tests open a live post in Chrome and WebKit.

## A blog

```text
my-blog/
  blog.config.json          # the blog's title, address, author, language, comments
  posts/
    why-compmd.comp.md      # a post: its slug is its file's name
    why-compmd/             # its own files: images, data
  compmd.config.json        # optional: the packs its posts use, as any app's
  theme.css                 # optional: the blog's look, after the default's
  layout.html               # optional: its own page, with <!-- content --> where each page goes
```

```json
{
  "title": "Notes",
  "url": "https://example.com/blog/",
  "author": "Ada Lovelace",
  "lang": "en",
  "description": "Things written down.",
  "comments": { "repo": "ada/notes", "repoId": "R_…", "category": "Blog", "categoryId": "DIC_…" }
}
```

`comments` is optional: each post's comments are then a discussion of the repository's GitHub Discussions, through
[giscus](https://giscus.app) (its page gives the ids), one per post, loaded when a reader scrolls to them.

A post's frontmatter:

```yaml
---
title: Why CompMD
date: 2026-11-20             # a day
summary: Markdown with the components it was missing.
tags: [design, intro]        # optional
author: Grace Hopper         # optional: the blog's author otherwise
updated: 2026-12-01          # optional: when it changed after it was published
draft: true                  # optional: shown by `cblo dev`, left out of `cblo build`
---
```

## Commands

Run in the blog's folder:

```sh
cblo new "Why CompMD"        # posts/why-compmd.comp.md, a draft with its frontmatter filled in (--slug names it)
cblo dev                     # the blog at http://localhost:4321/, drafts included (--no-drafts, --port)
cblo build                   # the blog as a static site, in dist/ (--out), without drafts
```

`compmd-blog` is the same command.

## What it writes

- `/`, the posts, newest first; `/<slug>/`, each post; `/tags/` and `/tags/<tag>/`, the tags and their posts;
- `feed.xml`, an Atom feed of the latest 20 posts, with their HTML; `posts.json`, the posts as data, for a site to
  show the latest elsewhere; `sitemap.xml`; and each post's own files beside its page;
- when a post has host-language components, the page's script, and the sandbox's frames and the blog's packs under
  `compmd/`, with a `_headers` file giving them the headers they are served with, for a static host that reads one.

A post with an error fails the build, with its line and column. Its code blocks are highlighted by
[Shiki](https://shiki.style), light and dark.

## Licence

MIT.
