# CompMD Developer's Guide

How to build with [CompMD](https://compmd.hakanserce.com/): documents people write as text, rendered as pages in your application, with components of your own and from packs, and safe to open from anyone. The [spec](../../compmd-spec-v1.md) says what CompMD is; the package READMEs say what each package has; this guide shows how to build with them, from a first page to a published pack. Writing documents themselves is [the Author's Guide](../authors/README.md).

It is written for the `@compmd` packages at version 0.1.1, and follows them as they change. On the website it is at [compmd.hakanserce.com/guides/developers/](https://compmd.hakanserce.com/guides/developers/).

## Who it is for

Developers who put CompMD in an application, or write packs for it. You know JavaScript or TypeScript and a bundler such as Vite. You want to know about the packages, the sandbox and its trust levels, components in React, Svelte or JavaScript, and publishing. You don't need to know CompMD's syntax first: each chapter explains what it uses, and links to the Author's Guide for the rest.

If you write documents rather than code, [the Author's Guide](../authors/README.md) is the book for you. The two meet at components: a component written in CompMD is in the Author's Guide, and one written in code is in this guide.

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

Each chapter's code is a file in [`examples/`](examples), which the guides' tests type-check, build and run in Chrome and WebKit, and compare with the chapter's text, so the code you read is code that works.

## Reading it on your machine

The chapters read as they are on GitHub. To see them as the website shows them, run these from the repository's root, after `pnpm install`. Each guide page's **Try it** links then open a playground on your machine.

- **Live, as you edit:** `pnpm --dir editor build` once, for the playground, then `pnpm --dir website dev`. Open [localhost:5173/guides/developers/](http://localhost:5173/guides/developers/); a page reloads when its chapter changes.
- **As it will be published:** `pnpm --dir editor build`, `pnpm --dir website build`, then `pnpm --dir website preview`, and open the address it prints.
- **The tests:** `pnpm --dir guides test` checks every chapter against its examples and runs the quick start in Chrome and WebKit.
