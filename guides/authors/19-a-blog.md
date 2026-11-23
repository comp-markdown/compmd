# 19. A blog

A folder of CompMD documents can be a blog: a page for each post, a list of them newest first, tags, a feed readers subscribe to, and comments if you want them. `cblo`, CompMD's blog tool, makes it into a site any web host can serve. This chapter turns the garden's newsletter into a blog that comes out between issues.

## Getting the tool

`cblo` runs in a terminal, with [Node.js](https://nodejs.org/) installed. The quickest start is a new blog, written for you:

```sh
npm create @compmd/blog garden-news
cd garden-news
npm install
npm run dev
```

The folder `garden-news/` then holds a blog with one post, and `npm run dev` shows it in your browser at http://localhost:4321/. In that folder, `npm run new`, `npm run dev` and `npm run build` are the three commands below.

For a blog you already have, install the command itself, once, and run it in the blog's folder:

```sh
npm install -g @compmd/blog
```

The package is `@compmd/blog`; the command it installs is `cblo` (or `compmd-blog`, the same command by a longer name).

## A blog's folder

```text
garden-news/
  blog.config.json            # the blog's title, address, author, language
  posts/
    the-first-frost.comp.md   # a post: its address is its file's name
    the-first-frost/          # its own files: pictures, data
```

Each `.comp.md` file in `posts/` is a post, and its file's name is its address: `the-first-frost.comp.md` is published at `the-first-frost/`. A post's pictures and data go in a folder beside it, and the post links to them as any document does (chapter 6): `![The frosted beds](the-first-frost/beds.jpg)`.

`blog.config.json` says what the blog is:

```json snippet=blog/blog.config.json
{
  "title": "Elm Street Garden",
  "url": "https://example.org/garden/",
  "author": "The garden committee",
  "lang": "en",
  "description": "News from the plots, between the newsletters."
}
```

- **`title`**, **`url`** and **`author`** are needed. `url` is where the blog will be published, ending in `/`: the feed and the links people share are made from it.
- **`lang`** is the posts' language, `en` when it doesn't say, and **`description`** a sentence on what the blog is, shown in the feed.

## A post

A post is a CompMD document like any other, with a few things in its frontmatter that the blog reads:

```md snippet=blog/posts/the-first-frost.comp.md
---
md:
  version: 1
title: The first frost
date: 2026-11-20
summary: The frost came early this year. What to cover tonight, and what to pick before the weekend.
tags: [news, winter]
pick: [Courgettes, Beans, Tomatoes]
cover: [Leeks, Kale, Parsnips]
---

The forecast says minus three tonight, two weeks before anyone expected it.

::: row {gap=m}
::: box {padding=m border=subtle border-radius=m}
**Pick today**

::: each {in=(md.doc.pick) as=crop join=list}
- {{crop}}
::: /each
::: /box
::: box {padding=m border=subtle border-radius=m}
**Cover tonight**

::: each {in=(md.doc.cover) as=crop join=list}
- {{crop}}
::: /each
::: /box
::: /row

The fleece is in the tool shed, on the shelf by the door. Take what you need, and bring it back on Sunday.
```

| Key | What it is |
| --- | --- |
| `title` | the post's title; its page shows it as the heading above the post, so the post starts with its first paragraph |
| `date` | the day it is published, `2026-11-20` |
| `summary` | a sentence or two, for the list of posts and the feed |
| `tags` | optional: a few words it is about; each tag gets a page of its posts |
| `author` | optional: who wrote it, when it isn't the blog's `author` |
| `updated` | optional: the day it last changed, after it was published |
| `draft` | optional: `true` keeps it out of the published blog while you write |
| `socialImage` | optional: the picture a link to it shows on social sites and in chats, with its `url` and `alt` |

Anything else in the frontmatter is the post's own, as in any document: `pick` and `cover` here are lists the post shows with `each` (chapter 5).

## Writing, seeing, publishing

Run each command in the blog's folder:

```sh
cblo new "The first frost"   # posts/the-first-frost.comp.md, a draft
cblo dev                     # the blog at http://localhost:4321/, drafts included
cblo build                   # the blog as a site, in dist/, without drafts
```

- **`cblo new`** starts a post: the file, its frontmatter filled in with the title, today's date and the blog's author, and `draft: true`. `--slug frost` names the file yourself, `posts/frost.comp.md`.
- **`cblo dev`** shows the blog as you write, drafts and all. Save a post, and its open page follows, as do the list and the tags' pages. A mistake in a post shows on its page, with its line and column, until you fix it. `--no-drafts` shows the blog as it will be published; `--port 4000` uses another port.
- **`cblo build`** writes the published blog into `dist/`: every post that isn't a draft, the list, the tags' pages, the feed (`feed.xml`) and a sitemap. A post with a mistake stops the build and says where it is, so nothing broken is published. `--out public` writes it somewhere else.

To publish, put what is in `dist/` on any web host that serves files, at the address `url` names: GitHub Pages, Netlify, or your own server. The site is plain pages; it needs nothing running on the host.

## Packs in a post

A post imports packs as any document does (chapter 16). A published blog fetches nothing while it is read, so the packs its posts use are installed in the blog's folder, and every pack installed there is the posts' to import:

```sh
npm install @comp-markdown/mermaid
```

A `compmd.config.json` beside `blog.config.json`, as any app's, is only for more: a pack every post gets without importing it, or one trusted with more than the sandbox.

A post that explains a drawing shows its code beside it with `render=both` (chapter 17), so the code is written once:

````md
```mermaid {render=both alt="Scraps go in the bin, then on the heap, then on the beds"}
graph LR; Scraps --> Bin --> Heap --> Beds
```
````

A post that uses no pack's code components loads no script at all: its page is ready as soon as it arrives.

## Pictures from elsewhere

A post's own pictures, beside it in `posts/`, always show. A picture from another site shows only if the blog allows that site, in `compmd.config.json`; otherwise the post shows its alt text, and the build says which picture it left out. `"origins": "any"` allows every site, for a blog whose posts are all its own writer's:

```json
{
  "origins": "any"
}
```

## Its look

The blog comes with a plain, readable look, light or dark as the reader's device is. Two optional files beside `blog.config.json` change it:

- **`theme.css`**, a stylesheet applied after the blog's own, for its colours and type;
- **`layout.html`**, a page of your own around every page of the blog, with `<!-- content -->` where each page goes: for a header and footer that match your site.

## Comments

`comments` in `blog.config.json` gives each post a discussion, through [giscus](https://giscus.app): the comments are kept in a GitHub repository's Discussions, one discussion per post, and readers sign in with GitHub to write. giscus's page gives the four values to copy in:

```json
{
  "comments": { "giscus": { "repo": "you/garden-news", "repoId": "R_…", "category": "Blog", "categoryId": "DIC_…" } }
}
```

The comments load only when a reader scrolls down to them.

Next: the appendices, starting with [the syntax at a glance](20-the-syntax-at-a-glance.md), everything a writer types in one place.
