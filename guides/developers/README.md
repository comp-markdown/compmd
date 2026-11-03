# CompMD Developer's Guide

How to build with [CompMD](https://compmd.hakanserce.com/): documents people write as text, rendered as pages in your application, with components of your own and from packs, and safe to open from anyone. The [spec](../../compmd-spec-v1.md) says what CompMD is; the package READMEs say what each package has; this guide shows how to build with them, from a first page to a published pack. Writing documents themselves is [the Author's Guide](../authors/README.md).

It is written for the `@compmd` packages at version 0.3.0, and follows them as they change. On the website it is at [compmd.hakanserce.com/guides/developers/](https://compmd.hakanserce.com/guides/developers/).

## Who it is for

Developers who put CompMD in an application, or write packs for it. You know JavaScript or TypeScript and a bundler such as Vite. You want to know about the packages, the sandbox and its trust levels, components in React, Svelte or JavaScript, and publishing. You don't need to know CompMD's syntax first: each chapter explains what it uses, and links to the Author's Guide for the rest.

If you write documents rather than code, [the Author's Guide](../authors/README.md) is the book for you. The two meet at components: a component written in CompMD is in the Author's Guide, and one written in code is in this guide.

## Contents

**Part I. Getting started**

1. [Introduction](01-introduction.md)
2. [Quick start](02-quick-start.md)
3. [How it fits together](03-how-it-fits-together.md)

**Part II. Documents in your app**

4. [Processing documents](04-processing-documents.md)
5. [Live editing](05-live-editing.md)
6. [Data and inputs](06-data-and-inputs.md)
7. [Rendering and styling](07-rendering-and-styling.md)

**Part III. Components**

8. [Components and trust](08-components-and-trust.md)
9. [Packs from npm](09-packs-from-npm.md)
10. [Your app's own components](10-your-apps-own-components.md)
11. [The sandbox, closer](11-the-sandbox-closer.md)
12. [The Developer Tools](12-the-developer-tools.md)

**Part IV. Writing packs**

13. [Your first pack](13-your-first-pack.md)
14. [Signatures and params](14-signatures-and-params.md)
15. [React, Svelte and libraries](15-react-svelte-and-libraries.md)
16. [Code renderers](16-code-renderers.md)
17. [Testing and publishing](17-testing-and-publishing.md)

**Part V. Beyond Vite**

18. [Other hosts](18-other-hosts.md)
19. [The Rust crate](19-the-rust-crate.md)

**Appendices**

20. [The configuration's reference](20-the-configurations-reference.md)
21. [The diagnostics](21-the-diagnostics.md)
22. [A glossary](22-a-glossary.md)

## The examples

Each chapter's code is a file in [`examples/`](examples), which the guides' tests type-check, build and run in Chrome and WebKit, and compare with the chapter's text, so the code you read is code that works.
