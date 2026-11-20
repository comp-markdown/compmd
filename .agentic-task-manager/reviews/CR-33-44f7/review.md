+++
id = "01a1fae6-fc79-728f-bd65-758beec244f7"
number = 33
title = "Sizing: the spec, its conformance cases and the processor"
status = "open"
tasks = ["CMD-353"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T21:40:04Z
updated = 2026-11-20T21:40:04Z
+++

Sizing a component in its box ([the design](docs/sizing-components.md)): the spec, its conformance cases and the processor. `cargo make ci` is green. **The spec text needs your go before it is pushed**; one change is breaking (below).

What to check:

- **The spec** (`spec/CHANGELOG.md`, *Unreleased*, names every section):
  - §9.2: `object-fit` and `object-position` in the table and a rule for them; "padding and border".
  - §7.4: `fit` in `mount`'s context; three new rules beside *The outer box is the processor's*: a component fits its box, its natural size, and "where a component runs is invisible to layout" (MUST). §7.6: the bindings pass `fit`.
  - §7: a signature fence's param list; F036's new meaning, moved from parsing to resolving (§10.2, §11). §12.1 and §12.3.
  - §5.1 and F013: **breaking** for a signature that declares a common param, `lang` or `dir`.
- **A correction to the design:** a breakpoint on the two params is F008, which the spec already defines as "a breakpoint the param doesn't take", not F009; the design says so now.
- **The processor:** the two params (`validate/universal.rs`, a `POSITIONS` kind); the signature fence's params kept in the tree, taken by its component at resolving unless its definition fence has some (F036); validated from the signature's node only for a component no definition fence defines (a built module's), so nothing is checked twice; F013 for the names, with a message of its own for the two `fit` params.
- **Conformance:** the design's three changed cases and six new ones; the schema's `componentSignature` with `params`; seven more parse trees gain `"params": []`.
- **Kept green beside it:** the renderer knows the two params and writes no CSS for them (`fit` itself is CMD-354); `params.json`, `llms.txt`, five guide chapters and the guides' F036 sample; a processor test's `max-width` param became `bar-width`.
