## website/blog/blog.config.json:15 · round 2 · t1
>   "image": { "url": "https://compmd.dev/og.png", "alt": "CompMD: Markdown with components. Readable as text, rendered as a page." }

- [x] **Hakan Serce** · 2026-11-20T09:02:12Z · c1
  why do we need this? is this the social image for the blog post? if so, why is it not part of the post's parameters? Or is it the default social media image if it is not defined in the blog front-matter? if so, let's change the property name to "default-social-image" or something like that.

## website/blog/posts/hello-world.comp.md:7 · round 2 · t2
> ---

- [x] **Hakan Serce** · 2026-11-20T09:02:12Z · c2
  I thought we would define the social image here in the frontmatter?

## npm-packages/blog/README.md:62-64 · round 2 · t3
> draft: true                  # optional: shown by `cblo dev`, left out of `cblo build`
> image: {url: why-compmd/cover.png, alt: A page and its source}   # optional: in place of the blog's image
> ---

- [x] **Hakan Serce** · 2026-11-20T09:02:13Z · c3
  I see, so it is actually, just like how it should be.

## guides/developers/21-the-configurations-reference.md:155-156 · round 2 · t4
> - **A list**: a document's data and images from those origins load, and show, without a question; any other origin is asked about, and is F028 until the reader allows it. An origin alone: no path, query or credentials, no wildcard.
> - **`"any"`**: no origin is asked about. For documents the app publishes itself, such as a static site's pages or a blog's posts, which a reader opens as any web page (spec §15). An app that shows documents others write leaves it out: the question is what keeps such a document from reaching an origin its reader hasn't agreed to.

- [x] **Hakan Serce** · 2026-11-20T09:02:14Z · c4
  Put a callout or something to make this stand out as important. We don't want folks to allow any origin without realizing what they are doing.

## guides/developers/21-the-configurations-reference.md:147 · round 2 · t5
> ### `origins`

- [x] **Hakan Serce** · 2026-11-20T09:02:15Z · c5
  Can they also customize the dialog shown? Do we provide such an option for host developers? If so let's explain it here.
