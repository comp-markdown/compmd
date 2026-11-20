+++
id = "01a1f842-d146-71bd-ae94-d6c01bbb1583"
number = 23
title = "Every origin for a host’s own documents, and the blog’s social images"
status = "approved"
tasks = ["CMD-308"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T09:21:30Z
updated = 2026-11-20T09:22:26Z
+++

A host may allow every origin, for documents it publishes itself, and the blog uses it; and the blog's social images. CR-22, the live review of the same change, was approved at round 3; this review is the task's, with the answers to CR-22's comments below and the one change made after its approval.

What to check:

- **Spec (§8.6, §15):** a host MAY allow every origin: no origin of a document's data or images is then disallowed (F028) or asked about. It fits documents the host publishes itself; a host that shows documents others write SHOULD NOT. URL schemes, a data `from`'s literal scheme and host, credentials and sandboxes' network are unchanged. Changelog under Unreleased; conformance case `expand/every-origin`.
- **Processor:** `AllowedOrigins` (`Listed` or `Every`, `crates/processor/src/allowed_origins.rs`); `validate` takes `impl Into<AllowedOrigins>`, so `.validate(&[])` and `.validate(&origins)` compile unchanged. The WebAssembly request takes `allowedOrigins: "any"` and refuses another string.
- **`@compmd/sandbox`:** `ConfiguredOrigins`, `"any"` in the image gate, `allowOrigins` and `loadData`.
- **`@compmd/vite`:** `origins` in `compmd.config.json` (a list of https origins, or `"any"`), given to the processor's `allowedOrigins` and the sandbox's `origins.configured`; its schema, README and `docs/vite.md`.
- **`@compmd/blog`:** its own origin always allowed (`origins.ts`); at build time an image of another origin shows its alt text, with a warning (`post-images.ts`); the live page gets the same origins. A post's `socialImage`, and the blog's `defaultSocialImage`, written as `og:image`, its alt and a large card (`social-image.ts`).
- **Developer Tools and the playground:** `"any"` kept when the reader allows more.
- **The website:** the blog allows every origin, its default social image is the site's card, and the first post has the review's corrections.
- **Docs:** chapter 21's `origins` section with its warning and the question's own dialog; `docs/blog-tool.md`, `docs/remote-images.md`, the sandbox's README.

After CR-22's approval: chapter 21's dialog example types `ask` as `AskOrigin` and calls an app's own `showOriginDialog`, in place of an undefined `myConfirm`.

Checks: `cargo make ci` (its package tests twice stopped by load: the Shiki highlighter tests, CMD-265, and a browser-close timeout in the wireframe pack; every package then passed on its own), `cargo make site`.
