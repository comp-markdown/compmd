+++
id = "01a1fe25-8eef-77c8-b9ef-b967352ba983"
number = 40
title = "No-code components: the spec, conformance, processor and renderer"
status = "approved"
tasks = ["CMD-360"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T12:47:16Z
updated = 2026-11-21T13:39:58Z
+++

[What `compmd` definitions need for the no-code components](docs/no-code-components.md), decisions 1 to 6: the spec, its conformance cases, the processor, and the renderer's part. **The spec text needs your go before it is pushed.**

## The spec

- **§7.2:** an inline invocation is one inline box, styled by its fence's params under the caller's, `box`'s layout params left out; with no params for a box it is its paragraph's content alone, as before. `md.fallback`, the fallback of a leaf or inline use, in its body, also in an image's alt; F014 in a definition that declares slots. `md.slots.<name>` in a definition's expressions.
- **§8.1:** the paths table, and interpolation in an image's alt text. **§8.6:** a destination that starts with a param (a field of one, an `each` name over one) resolves against the caller.
- **§9.2, §9.4:** `border`'s four logical sides (`border` covers them in the cascade), `shadow` (`none s m l`, `--compmd-shadow-*`, tokens only), `rotate` (an angle from `-15deg` to `15deg`, types `shadow` and `angle`), `stretch-link` (a flag, no breakpoint). **§10:** W124, `stretch-link` with no link, at the block. **§12.2, §12.3:** `altParts`; the inline `box` holds its paragraph as its `fallback`.
- The changelog under *Unreleased*, and `spec/llms.txt`.

## Two choices to check

1. **`md.fallback` is F014 in a definition that declares slots,** found at validating, at the path: the design said "in a container's body", and a definition with slots is the one used as a container. It means the check is where the path is written, not at each use.
2. **The inline box holds its text as its `fallback`** in the expanded tree: an inline node has no `children`, and the text is what a renderer that doesn't draw the box shows anyway.

## Conformance

Seven new cases: `validate/shadow-rotate-and-border-sides`, `validate/stretch-link-W124`, `expand/border-sides-cascade`, `expand/md-fallback`, `expand/md-slots`, `expand/inline-invocation-box`, `expand/param-destination-base`. Every existing case is unchanged and passes. `params.json` gains the new params (written by hand, as `COMPMD_WRITE=1` is an env prefix; the vocabulary test confirms it equals what the processor gives). `compmd-tree.schema.json` takes `altParts`.

## The renderer: CMD-361's scope, done here

The renderer's vocabulary test requires it to style every param the processor knows, so the processor's new params can't pass ci without their CSS. This commit has the renderer's part: the border sides, `shadow`, `rotate`, `stretch-link` (the box marks its first link, `stretch-link.ts`, and the base rules stretch it), and the inline box drawn by `BoxView`. Browser tests in Chrome and WebKit: a click on a card's text reaches its first link, another link stays its own, the marking follows an update, and an inline box sits in its line. **CMD-361 has nothing left; I'd mark it skipped as done here, if you agree.**

## The guides

Author's Guide chapters 3 (the params), 12 (the inline box, `md.fallback`, links from a use), 13 (`md.slots`), 20 (W124); Developer's Guide chapter 22 (W124), with its sample in the test.

## Found on the way

A pack's own relative links (`./icons/star.svg`) are kept as written, so they don't reach the pack's files: the spec makes that the processor's job, and nothing does it. Decision 1 doesn't depend on it. It is CMD-362, which needs a decision first.
