# The CompMD blog

**Status: proposed (2026-11-17).** A blog on [the website](design.md), at `/blog/`, for what doesn't fit the spec or
the guides: why CompMD is as it is, where it is going, how it compares with what else there is, and what each release
brings. Its first posts are *Why CompMD*, *Where CompMD is, and its roadmap*, and *MDX and CompMD*. It is built with
[`@compmd/blog`](../../docs/blog-tool.md), CompMD's blog tool, as its first user.

## Proposed decisions

1. **The blog is a folder of the website, `website/blog/`, as `@compmd/blog` reads one**: its posts in `posts/`, each a
   CompMD document (`why-compmd.comp.md`), its `blog.config.json` (the title "The CompMD blog", the address
   `https://compmd.dev/blog/`, the author), and the site's `compmd.config.json` for packs, so a post can use the site's
   own pack (`content/app/`) as the landing page does. A post's frontmatter is the tool's: `title`, `date`, `summary`,
   and optional `author`, `updated`, `tags` and `draft`.

2. **The site's build runs `compmd-blog build`** into `dist/blog/`, beside the guides and the spec, and its dev server
   serves the blog's pages from `compmd-blog dev`, as it serves the playground from the playground's.

3. **It looks like the rest of the site** with a `layout.html` of its own: the site's header, with "Blog" beside the
   guides and the spec, and its footer; and a `theme.css` that takes the site's fonts and colours, light and dark.

4. **Tags from the start**: each post names a few (`release`, `design`, `comparison`), the tool writes their pages at
   `/blog/tags/<tag>/`, and the feed carries them.

5. **Comments with giscus** (proposed): each post's comments are a discussion in this repository's GitHub Discussions,
   in a category of their own, "Blog", so readers talk where the project's developers already are, with no tracking
   and no ads. It needs Discussions turned on for the repository and the giscus app installed on it, both by the
   repository's owner. Disqus is what the tool offers too, if that is preferred: it lets readers comment without a
   GitHub account, but its free plan shows ads and tracks readers.

6. **Part of the site, as the guides are**: its pages in the sitemap and in the site's search (each post from one
   heading to the next), checked by `pnpm check` (light and dark, desktop and phone, styled before their script, their
   links, the feed found from the pages) and within the landing page's budgets: at most 100 KB gzipped before their live
   parts, and the largest paint within 1 s on a mid-range phone over fast 3G. The comments load only when scrolled to,
   so they count against neither.

7. **This change sets the blog up, with one short post that says what it is for**; the three first posts are a change
   each, so each is reviewed as it is written. It comes after `@compmd/blog` is built.

## Not in this change

- A "Latest from the blog" line on the landing page, once there are posts worth pointing to.
