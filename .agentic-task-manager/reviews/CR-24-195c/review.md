+++
id = "01a1f86d-cb82-7761-a103-4e41b144195c"
number = 24
title = "A document’s host components: only those it uses"
status = "open"
tasks = ["CMD-268"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T10:08:27Z
updated = 2026-11-20T10:08:27Z
+++

G10 of the first components: the processor listed every host-language component a document could name, an ambient pack's included, so a host that makes packs built in for every document paid for all of them (the sandbox compiled each, the blog gave every post its script). It now lists those the document uses.

What to check:

- **The rule** (`crates/processor/src/resolve/reached.rs`): from the processed document's body (not its definitions), each component it invokes or draws a code block with, in every `if` branch and `each` template whatever the inputs, then the components the `compmd` definitions it reaches invoke, each name read in the document that writes it. An example is resolved on its own, so its list is its own.
- **Why not the expanded tree:** my first version listed the expanded tree's components. The site check caught it: the book club's rating is in a branch the first render doesn't show, so when the reader picked a read book the rating had no code. What the document uses doesn't change with its inputs; a host gives the list once.
- **The spec** says the processor lists host-language components (§12.3) but not which, so no spec change.
- **A name two documents give:** the processed document's still wins when both are used; when only a pack's is (through one of its definitions), the pack's is listed now, where before the processed document's unused one was.
- **Tests:** a document with components in an untaken branch, through an unused definition, and in an example; an ambient pack's two components with a document using one, none, and one in an example; a blog post that uses no component of an ambient pack gets no script. Tests whose documents imported packs without using them now use them.
- **No sandbox test:** the sandbox compiles the components it is given and doesn't change; the processor's tests cover the list.

Checks: `cargo make ci`, `cargo make site`.
