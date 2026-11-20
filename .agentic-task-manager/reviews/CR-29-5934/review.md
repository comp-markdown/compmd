+++
id = "01a1f9f9-495f-7382-af67-556e44d45934"
number = 29
title = "Wrapping an npm package as a pack: the design"
status = "open"
tasks = ["CMD-8"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T17:20:26Z
updated = 2026-11-20T17:20:26Z
+++

The design of wrapping an npm package as a pack, with the three choices settled in conversation (2026-11-20): both kinds served, the signature generated at each build, and the pack's own version.

What to check:

- **Decisions 1 to 3:** the command (`--from`), how the kind is read from the package, and the glue written once in the plain binding.
- **Decisions 4 and 5, mine to propose:** the build rewrites only the signature fence marked `generated`, and the params are an allowlist in `compmd.wrap.params`, so a package's new prop is reported, never added on its own. The spike showed why: react-qr-code inherits several hundred SVG props, layerchart's `BarChart` has about 90 of its own.
- **Decision 6, the mapping table,** and that defaults come from JSDoc `@default` only (react-qr-code's comment says 128, its code uses 256).
- **Decision 7, the overrides:** names, types, defaults, an input with its callback, the fallback's prop, plain children as a slot.
- **Decision 9:** what can't run in a strict frame is a warning at once, not a silent pack (model-viewer).
- **The spike's four packages** and **the gaps** (G1 to G6); G6 is a question for the spec: whether `?` after an enum is valid, which no example shows.
- **For CMD-9:** the work lands in `@compmd/create-pack` and `@compmd/pack` (the build's signature generation). It may be worth two tasks rather than one; say if you'd like it split.
