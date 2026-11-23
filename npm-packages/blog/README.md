# @compmd/blog

A blog in [CompMD](https://compmd.dev/): a folder of CompMD documents made a static site, with a page per post, tags,
an Atom feed and comments. Every post is rendered by CompMD when the blog is built, so it reads as a page before any
script runs. The design is [docs/blog-tool.md](https://github.com/comp-markdown/compmd/blob/main/docs/blog-tool.md).

**Status: in development.** A post's host-language components, its own or from the blog's packs (`compmd.config.json`,
as any app's, read by [`@compmd/vite`](https://github.com/comp-markdown/compmd/tree/main/npm-packages/vite)), show their fallback in
the page as it was built, and are drawn live once its script loads, through the sandbox; a post without them loads no
script. Its tests open a live post in Chrome and WebKit.

**Versions:**
- **0.9.0:** a post's share image, `socialImage` in its frontmatter or the blog's `defaultSocialImage`, as `og:image` and a large Twitter card; the blog's own origin always allowed, and with `origins: "any"` in its `compmd.config.json` every origin, so no image asks the reader, and an image of an origin not allowed is a warning at build; a post's script only for the components it uses; a trusted pack's Svelte components built into the page's script; `sharedFiles`, files a site shares with its posts, and `fileFor`, `sendBlogFile` and `CONTENT_MARKER`, for a site that builds and serves its blog itself.
- **0.8.0:** the first version: `cblo new`, `cblo dev` and `cblo build`, the posts' pages, tags, an Atom feed and comments, each page within its size budget, and a post's host-language components drawn live. (`0.0.0` is a placeholder with no code.)

## Install

A new blog, in a folder of its own, with `@compmd/blog` as its dependency and the scripts to write, see and build it
([`@compmd/create-blog`](https://www.npmjs.com/package/@compmd/create-blog)):

```sh
npm create @compmd/blog my-blog
cd my-blog
npm install
npm run dev
```

Or the command on its own, for a blog you already have, run in the blog's folder:

```sh
npm install -g @compmd/blog   # then `cblo` in any folder
npx -p @compmd/blog cblo dev  # or without installing it: the package's name isn't the command's, hence `-p`
```

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
  "comments": { "giscus": { "repo": "ada/notes", "repoId": "R_…", "category": "Blog", "categoryId": "DIC_…" } },
  "defaultSocialImage": { "url": "card.png", "alt": "Notes, by Ada Lovelace" }
}
```

A post's images and remote data from the blog's own origin show without asking its readers; those of the origins its
`compmd.config.json` names in `origins` too, and with `"origins": "any"` every one, as for a site's own pages (spec
§15). An image from another origin shows its alt text, and the build says so.

A post's social image is the image a link to it shows on social sites and in chats (its Open Graph image, as a large
card): its own `socialImage`, in its frontmatter. `defaultSocialImage` is optional: the one every other page shows,
and a post without its own. Each has a `url`, an http(s) address or one in the blog (`card.png` is `posts/card.png`,
written at the blog's root), and an `alt`.

`comments` is optional, and names its provider: with `giscus`, the one there is so far, each post's comments are a
discussion of the repository's GitHub Discussions, through [giscus](https://giscus.app) (its page gives the ids), one
per post, loaded when a reader scrolls to them.

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
socialImage: {url: why-compmd/cover.png, alt: A page and its source}   # optional: the blog's default otherwise
---
```

## Commands

Run in the blog's folder:

```sh
cblo new "Why CompMD"        # posts/why-compmd.comp.md, a draft with its frontmatter filled in (--slug names it)
cblo dev                     # the blog at http://localhost:4321/, drafts included, its pages following each save (--no-drafts, --port)
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
[Shiki](https://shiki.style), light and dark. A post that explains a block a pack draws, a diagram say, shows its code
and its drawing together with `render=both` on the block, so the code is written once:

````md
```mermaid {render=both alt="Draft, then review, then published"}
graph LR; Draft --> Review --> Published
```
````

A post about CompMD shows CompMD and what it makes the same way, with ```` ```compmd {render=both} ````: the block is
a document of its own, whose inputs the reader can change.

## Licence

MIT.
