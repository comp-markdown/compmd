## website/blog/blog.config.json:15 · round 1 · t1
- [x] **claude:79612601** · 2026-11-20T09:21:50Z · c1
  From CR-22 (c1): this is the default social image, which every page shows when no post sets its own (the list, the tags’ pages, and a post without one). Renamed from `image` to `defaultSocialImage`, and a post’s own field to `socialImage`, so neither reads as any image. camelCase, as the file’s other keys (`repoId`, `categoryId`). The module is `social-image.ts` (`SocialImage`, `socialImageOf`, `socialImageTags`).

## website/blog/posts/hello-world.comp.md:1-7 · round 1 · t2
- [x] **claude:79612601** · 2026-11-20T09:21:51Z · c2
  From CR-22 (c2): a post can set its own, `socialImage: {url, alt}` here, which takes the default’s place for that post. This one leaves it out: its image would be the site’s card, which `defaultSocialImage` already gives every page, so writing it here would repeat the default. Easy to add if each post should name its own.

## npm-packages/blog/README.md:44-47 · round 1 · t3
- [x] **claude:79612601** · 2026-11-20T09:21:51Z · c3
  From CR-22 (c1, c3): a post’s own image, `socialImage`, with the blog’s `defaultSocialImage` otherwise; the frontmatter example (line 64) says “the blog’s default otherwise”.

## guides/developers/21-the-configurations-reference.md:158-159 · round 1 · t4
- [x] **claude:79612601** · 2026-11-20T09:21:51Z · c4
  From CR-22 (c4): the `> [!WARNING]` callout after the two cases. `"origins": "any"` is only for documents you publish yourself: with it every document reaches any server its author names with no question, each fetch telling that server the reader’s address and the time, and a data `from` can carry the reader’s inputs; an app that shows documents others write (an editor, a viewer, a site taking contributions) must not set it, and names the origins it trusts instead.

## guides/developers/21-the-configurations-reference.md:163-175 · round 1 · t5
- [x] **claude:79612601** · 2026-11-20T09:21:51Z · c5
  From CR-22 (c5): yes, hosts can show their own dialog, in code rather than the configuration: `createSandbox`’s `origins` takes an `ask`, `(origin, document, kinds) => Promise<"allow" | "deny">` (`originQuestion(origin, kinds)` gives the words, for data, images or both), and `answers`, where the answers are kept (`localStorage` by default); `loadData` takes the same, so one answer covers an origin’s data and images. Chapter 6 shows both. Changed after CR-22’s approval: the example types `ask` as `AskOrigin` and calls the app’s own `showOriginDialog`, in place of an undefined `myConfirm`.
