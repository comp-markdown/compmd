+++
id = "01a1f9f9-495f-7382-af67-556e44d45934"
number = 29
title = "Wrapping an npm package as a pack: the design"
status = "open"
tasks = ["CMD-8"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T17:20:26Z
updated = 2026-11-20T19:30:11Z
+++

Round 3: the generated signature is marked with comments before and after it, as decided in conversation (G7).

- **Decision 4:** the build rewrites the signature fence between `<!-- compmd-pack:generated … -->` and `<!-- /compmd-pack:generated -->`; the opening comment says what it's generated from and that edits are overwritten. The build fails on a pair holding anything but one signature fence, or on an unpaired comment. Removing the pair hands the fence to the author: the build leaves it and reports where it differs from the types. No spec change.
- **"In use"** and the spike's three generated signatures use the comments; the fences lose `generated`. The spike ran again with the same results.
- **G7** is settled.
