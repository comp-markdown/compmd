# The CompMD blog

**Status: proposed (2026-11-17).** A blog on [the website](design.md), at `/blog/`, for what doesn't fit the spec or
the guides: why CompMD is as it is, where it is going, how it compares with what else there is, and what each release
brings. Its first posts are *Why CompMD*, *Where CompMD is, and its roadmap*, and *MDX and CompMD*.

## Proposed decisions

1. **A post is a CompMD document in the repository, `website/blog/<slug>.comp.md`**, so the blog is written in what
   it writes about and every post shows it working. Its frontmatter says what the pages need:

   ```yaml
   ---
   title: Why CompMD
   date: 2026-11-20
   summary: Markdown that people already write, with the components it was missing, safe to open from anyone.
   author: Hakan Serce        # the site's author when left out
   updated: 2026-12-01        # when a post is changed after it is published; optional
   draft: true                # shown by the dev server, left out of the build; optional
   ---
   ```

   The slug is the file's name, and stays: it is the post's address. A date is a day, as the changelog's are.

2. **Rendered at build time by the processor and `@compmd/render`**, as the landing page's examples are
   (`build/snapshots.ts`): each post is processed in strict mode, so an error in a post fails the build, and rendered to
   static HTML, which is the page before any script runs. A post may use everything CompMD has: `@compmd/core`'s
   components, data files beside it (`website/blog/<slug>/`), inputs, and the site's own pack (`content/app/`, as the
   landing page). Its host-language components show their fallback in the HTML and are mounted live once the page's
   script loads, as the landing page's are; a post with none loads no sandbox at all. Code blocks are highlighted by
   Shiki at build time, CompMD's with the site's grammar.

3. **Three kinds of page:**
   - **`/blog/`, the list**: every post, newest first, each with its title (a link), its date, its author and its
     summary; one page, until there are enough posts to need more.
   - **`/blog/<slug>/`, a post**: its title, its date (and "updated" with its date when it has one), its author, the
     post, a link to its source on GitHub (as the spec page's headings have), and links to the posts before and after
     it.
   - **`/blog/feed.xml`, an Atom feed**: each post with its title, its address as its `id`, its dates (`published`,
     `updated`), its author, its summary and its rendered HTML as `content`; the list page and every post name it in a
     `<link rel="alternate">`, so a feed reader finds it. Atom rather than RSS: it says what each field is, and every
     reader takes it.

4. **The blog is part of the site, as the guides are**: "Blog" in the header beside the guides and the spec, its
   pages in the sitemap and in search (each post from one heading to the next, as the guides' pages are), light and
   dark, desktop and phone, styled before their script, and printed well from the browser (no PDF of its own). A post
   page has the landing page's budgets: at most 100 KB gzipped before its live parts, and its largest paint within
   1 s on a mid-range phone over fast 3G.

5. **This change builds the blog, with one short post that says what the blog is for**; the three first posts are a
   change each, so each is reviewed as it is written.

## Tests

- `build/blog.test.ts`: posts read from their frontmatter, newest first, a draft left out of the build and kept by
  the dev server, a post with an error failing the build, the feed's XML (well formed, each entry's fields, its
  `content` the post's HTML), the sitemap's and the search index's entries.
- `pnpm check`: the list and a post, light and dark, desktop and phone, styled before their script, their links,
  the feed found from the pages, the budgets; and the site check run after the work.

## Questions, with proposed answers

- **Comments?** None: a post links to the repository for talking about it (its issues, or its discussions if they are
  turned on).
- **Tags or categories?** Not until there are enough posts to need them.
- **The landing page?** A "Latest from the blog" line under the hero, once there are posts worth pointing to; not in
  this change.
