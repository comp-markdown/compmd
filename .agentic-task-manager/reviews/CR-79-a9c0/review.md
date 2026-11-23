+++
id = "01a20a36-2d1c-73aa-8e74-8461c4fda9c0"
number = 79
title = "Processor: a signature fence below its definition moves with an edit"
status = "approved"
tasks = ["CMD-389"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T21:00:52Z
updated = 2026-11-23T21:14:04Z
+++

An edited processor could keep a box param a new one drops, when an edit fell between a host-language component's definition and its signature fence.

## Cause
After an edit, `Resolved::splice` moved a resolved component whole, and only when its definition's position moved. A host-language component's signature and box params come from its signature fence, which can sit below the definition. A blank line typed before the fence moved the fence in the tree and in validation's refused params, but not in the component. The component's `title` then no longer matched the refused one, and the next edit that re-expanded its use kept `title=x`.

Found by `tests/edit.rs`'s random edits on `conformance/validate/signature-box-unknown-F008`: the new gradient cases (CMD-388) shifted the case's index, and so its seed.

## Fix
- `Component::signature_at`: where its signature is written, its definition or the signature fence.
- `Component::follow(moved)`, in place of the `Shift` impl: the signature moves with `signature_at`, each fence param by its own position, and the definition and its body with the definition.

## Tests
`a_signature_fences_refused_param_stays_refused_after_edits_around_it`: the two edits that showed it, each checked against a new processor. `cargo make ci` passes.
