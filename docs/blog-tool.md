# A blog in CompMD (proposed)

A small tool that turns a folder of CompMD documents into a blog, a static site anyone can host: a list of posts, a
page per post, tags and a feed, with every post rendered by CompMD at build time and its components live in the page.
Anyone who writes in CompMD gets a blog from it in a few minutes; the CompMD website's own blog is its first user
([the blog's design](../website/docs/blog.md)). It is kept small on purpose: a blog, not a general site generator.

## Decisions

1. **Its name: `@compmd/blog`, with the command `compmd-blog`** (proposed). A blog is all it does, so the name says
   that; a site generator would be a different, larger tool, and if this one grows into it, it takes that name then.
   The other names considered: `compmd-blogger` (the same, less like the other packages' names) and `compmd-site-gen`
   (a promise of more than it does).

2. **A blog is a folder, and the tool reads it as it is:**

   ```text
   my-blog/
     blog.config.json          # the blog's title, its address, its author, its language, comments
     posts/
       why-compmd.comp.md      # a post: its slug is its file's name
       why-compmd/             # a post's own files: data, images
     theme.css                 # optional: the blog's look, over the default's
     compmd.config.json        # optional: packs, as any app's (docs/vite.md)
   ```

   A post's frontmatter: `title`, `date` (a day), `summary`, and optional `author` (the blog's by default), `updated`,
   `tags` (a list of names) and `draft` (shown by `dev`, left out of `build`). Anything else is the post's own, as
   `md.doc` (spec §2.1).

3. **Two commands:** `compmd-blog dev`, a server that renders a post as it is saved, drafts included; and
   `compmd-blog build`, which writes the static site into `dist/`. `npm create @compmd/blog` writes a new blog's folder
   with one post, as `npm create @compmd/pack` writes a pack.

4. **What it writes:**
   - `/`, the posts, newest first, each with its title, date, author, summary and tags;
   - `/<slug>/`, a post: its title, dates, author, tags, the post, the posts before and after it, and its comments when
     the blog has them (decision 7);
   - `/tags/<tag>/`, the posts with a tag, and `/tags/`, every tag with its count;
   - `/feed.xml`, an Atom feed of the latest posts, each with its tags as `category` and its HTML as `content`, named
     by every page in a `<link rel="alternate">`;
   - `sitemap.xml`, and each page's title, description and Open Graph tags from the post.

5. **Built on CompMD's packages, as a host is** ([hosting](hosting.md)): each post is processed by
   `@compmd/processor` in strict mode, so an error fails the build, and rendered to static HTML by `@compmd/render`, the
   page before any script runs. A post's host-language components (from the blog's packs, through `@compmd/vite`'s
   bundle and `@compmd/sandbox`) show their fallback in the HTML and are mounted live once the page's script loads; a
   post with none loads no script at all. Code blocks are highlighted by Shiki at build time, CompMD's with its own
   grammar.

6. **Its look is one stylesheet and one layout.** The default theme is plain, readable and light or dark by the
   reader's preference, with CompMD's own styles for components. A blog changes it with `theme.css`, and, for its own
   header and footer, with `layout.html`: an HTML page with `<!-- content -->` where the tool puts each page. No theme
   system or plugins: a blog that needs more is a site, not this tool's job.

7. **Comments, from a provider the blog names**, or none (the default). `blog.config.json`'s `comments` names one:
   - **`giscus`** (proposed for most blogs): comments as GitHub Discussions of a repository, by a small open-source
     script, with no tracking or ads; readers sign in with GitHub. It suits a blog for developers;
   - **`disqus`**: a hosted service, readers sign in with an account of their own or as guests; its free plan shows ads
     and its script tracks readers across sites, which a blog's privacy statement should say.
   The comments load only when a reader scrolls to them, so a post's page stays within its budget, and the page's
   content security policy names only the provider's origins.

8. **In this repository, a workspace package beside `@compmd/vite`**, released with the other `@compmd` packages: it
   depends on them at their own version and is tested against them, and the CompMD website builds its blog with it,
   so the site checks it on every build. Its tests: a blog of a few posts built and checked (the pages, the tags, the
   feed's XML, a draft left out, a post with an error failing the build), and the pages in Chrome and WebKit, light and
   dark, desktop and phone, within the landing page's budgets.

## Not in this change

- Pagination of the list, search, and posts in more than one language: when a blog needs them.
- Other comment providers: each is a small adapter, added when a blog asks for it.
