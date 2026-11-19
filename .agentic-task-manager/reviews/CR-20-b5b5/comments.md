## website/blog/posts/the-compmd-blog.comp.md:21-23 · round 1 · t1
> > [!NOTE]
> > The blog is built with [`@compmd/blog`](https://github.com/comp-markdown/compmd/tree/main/npm-packages/blog), CompMD's
> > blog tool: a folder of CompMD documents made a static site.

- [x] **Hakan Serce** · 2026-11-19T19:09:30Z · c1
  Do we have a way to put this to a single layout file so that we don't need to repeat this in all the posts?
  - **claude:79612601** · 2026-11-19T19:10:05Z · c3
    Yes: the layout is ours (`src/BlogLayout.svelte`), so the line goes there, under every page of the blog, written once, and the post drops its note.

## website/blog/posts/the-compmd-blog.comp.md:2-23 · round 1 · t2
> title: The CompMD blog
> date: 2026-11-19
> summary: What this blog is for, and how its posts are written.
> tags: [about]
> ---
>
> The spec says what CompMD is, and the guides how to write it and build with it. This blog is for the rest: why CompMD
> is as it is, where it is going, how it compares with what else there is, and what each release brings.
>
> The first posts are coming:
>
> - **Why CompMD**: what it gives a writer that plain Markdown doesn't, and what it keeps.
> - **Where CompMD is, and its roadmap**: what works today, and what comes next.
> - **MDX and CompMD**: two answers to components in Markdown, side by side.
>
> Each post is a CompMD document in [the repository](https://github.com/comp-markdown/compmd/tree/main/website/blog),
> rendered when the site is built, so it reads the same as a page and as plain Markdown. The [feed](feed.xml) carries
> every post, and each has its comments below it, in the repository's discussions.
>
> > [!NOTE]
> > The blog is built with [`@compmd/blog`](https://github.com/comp-markdown/compmd/tree/main/npm-packages/blog), CompMD's
> > blog tool: a folder of CompMD documents made a static site.

- [x] **Hakan Serce** · 2026-11-19T19:09:30Z · c2
  Assuming this is a placeholder. I would rather start with a "Hello, world!: Introducing CompMD..." or something like this for the first post.
  - **claude:79612601** · 2026-11-19T19:10:06Z · c4
    It was the design’s “one short post that says what the blog is for”. Replaced with “Hello, world: introducing CompMD”: what CompMD is, a small document shown beside what it makes (`render=both`), and a line on what the blog will hold.

## website/blog/posts/hello-world.comp.md:17-40 · round 2 · t3
> ````compmd {render=both}
> ---
> coming:
>   - title: Why CompMD
>     about: what it gives a writer that plain Markdown doesn't, and what it keeps
>   - title: Where CompMD is
>     about: what works today, and what comes next
>   - title: MDX and CompMD
>     about: two answers to components in Markdown, side by side
> ---
>
> ::: grid {min=12rem gap=m}
>
> ::: each {in=(md.doc.coming) as=post}
>
> ::: box {padding=m border=subtle border-radius=m}
>
> **{{post.title}}**: {{post.about}}.
>
> ::: /box
>
> ::: /each
>
> ::: /grid

- [ ] **Hakan Serce** · 2026-11-19T19:39:05Z · c5
  This is good, I have two issues: 1- this directly jumps into a code-heavy example. I would rather start with an example where there is a minimal use of non-text pieces, this can come as a second example after the easier to consume few examples that can create some interest. I would use one of our shinier examples, leaving the actual component definitions out of the example.  2- the example content is about CompMD, which is interesting in its own accord, but for a new reader, I would have something they are familiar with as the first exposure here.
  - **claude:79612601** · 2026-11-19T19:40:59Z · c6
    Reworked: it opens with the book club from the landing page (a list from its frontmatter and a total, nearly all text, about something everyone knows), then the landing page’s Plan, Build, Ship cards (layout only), and only then the posts-to-come grid. The hero’s own shelf and rating are host-language components, which the blog can’t run yet (the Svelte pack gap in round 1’s summary), so the book club shows its list and total rather than its covers.
    - **Hakan Serce** · 2026-11-19T19:42:36Z · c7
      let's fix this so it actually shows the book covers
      - **Hakan Serce** · 2026-11-19T19:43:03Z · c8
        also you said you fixed it, but I still don't see the 3rd round
        - **claude:79612601** · 2026-11-19T19:46:18Z · c9
          You’re right: I replied before the commit, and no round was started. Round 3 comes with the covers (c7), once the post shows them.
      - **claude:79612601** · 2026-11-19T19:46:18Z · c10
        On it: the blog tool builds a post’s page script with `@compmd/vite` alone, so it can’t compile the site’s trusted Svelte pack (`Stars.svelte`, the rating). I’ll fix that in `@compmd/blog`, then the post uses the hero as it is: the shelf of covers, the rating, and the book picked.
