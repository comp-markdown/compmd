# CompMD Developer Guide

How to build with [CompMD](https://compmd.hakanserce.com/): documents people write as text, rendered as pages in your application, with components of your own and from packs, and safe to open from anyone. The [spec](../compmd-spec-v1.md) says what CompMD is; the package READMEs say what each package has; this guide shows how to build with them, from a first page to a published pack.

It is written for the `@compmd` packages at version 0.1.1, and follows them as they change. On the website it is at [compmd.hakanserce.com/guide/](https://compmd.hakanserce.com/guide/).

## Contents

**Part I. Getting started**

1. [Introduction](01-introduction.md)
2. [Quick start](02-quick-start.md)
3. [How it fits together](03-how-it-fits-together.md)

**Part II. Documents in your app** *(to come)*

4. Processing documents
5. Live editing
6. Data and inputs
7. Rendering and styling

**Part III. Components** *(to come)*

8. Components and trust
9. Packs from npm
10. Your app's own components
11. The sandbox, closer

**Part IV. Writing packs** *(to come)*

12. Your first pack
13. Signatures and params
14. React, Svelte and libraries
15. Code renderers
16. Testing and publishing

**Part V. Beyond Vite** *(to come)*

17. Other hosts
18. The Rust crate

**Appendices** *(to come)*: the configuration's reference, the diagnostics, a glossary.

## The examples

Each chapter's code is a file in [`examples/`](examples), which this folder's tests type-check, build and run in Chrome and WebKit, and compare with the chapter's text, so the code you read is code that works.
