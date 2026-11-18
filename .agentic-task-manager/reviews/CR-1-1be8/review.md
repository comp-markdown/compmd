+++
id = "01a1ebb5-a58c-75f5-8c8c-53967aa61be8"
number = 1
title = "`render=source|output|both`, and examples"
status = "approved"
tasks = ["~CMD-37"]
author = "claude:e2fd71c2"
reviewers = []
created = 2026-11-17T22:51:52Z
updated = 2026-11-18T00:37:11Z
+++

The build of the accepted design `docs/code-and-output.md`: `render=output|source|both` on rendered code blocks (spec §7.7), and examples, ` ```compmd ` blocks that render as documents of their own (spec §7.8).

## What to check

- **`render`** (6d95eea, 445885e): bare values only (F009 otherwise); `source` needs no `alt` (no W101); `both` is one box, code first, and in a frame the drawing shows alone, so the code is shown once.
- **Examples** (456c35f): a ` ```compmd ` block is `render=source` by default; with `output` or `both` the processor runs it as a nested document, with diagnostics and origins mapped to the outer file; W123 for an example inside an example; an edit of a document with examples expands it whole.
- **An example's own components** (option a: 69b33ed, 8132016): numbered by the example through the processor, `@compmd/render` and the sandbox (`MountSet`, `HostComponent.example`).
- **An example's inputs** (da4a466, f443e76): `Processor::example_inputs`, the wasm request's `exampleInputs`, the `exampleSet` message; the blog and the website pass the values back. Includes the fix for a framed use in an example setting the document's input.
- **Spec, conformance and guides** (8e0fb53 and the commits above): §5.1, §6.3, §7.7, §7.8, §8.7, §10.2, §12.3, §14; four new conformance cases; Author's Guide chapters 17 and 20; developers' chapters 06, 16 and 22.

## Not in this task

- The website's `Example.svelte` stays: the landing page is Svelte, not a CompMD document.
- Editor grammars and the language server completing `render`: notes on CMD-20, CMD-21 and CMD-24.

## In the range, already approved

`4383025`, `d2121e4`, `15b0004` and `8f9465b`, the wireframe pack's work, sit between this task's commits; they were approved in their own tasks.

All Rust and JS tests pass.
