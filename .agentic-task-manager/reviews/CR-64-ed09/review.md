+++
id = "01a20438-5bf2-75b6-83a4-435902a2ed09"
number = 64
title = "What a component may assume about its box: the design"
status = "approved"
tasks = ["CMD-377"]
author = "claude:9ea81469"
reviewers = []
created = 2026-11-22T17:05:32Z
updated = 2026-11-22T17:11:23Z
+++

The design for CMD-374's question, written from scratch: [docs/component-environment.md](docs/component-environment.md). Instead of each component adapting to a strict page, the host contract promises that inline styles apply in a box's document, and the processor keeps it: the frames already do, and a host that mounts trusted components in a strict page keeps them in frames or allows inline styles.

Please check:

- **The decisions to confirm:** the promise covers inline styles only (decision 3); no frame for trusted code now (decision 4, alternative G); reverting the Mermaid pack's `htmlLabels: false`, never released (decision 7).
- **The security risks:** a strict host loosening its own page (risk 1), and a document's CSS reaching the host's page through a trusted component (risk 2, decision 5).
- **The alternatives rejected**, A to I, each with why; A and B were tried.
- **The work needed**: the spec (§7.4, §15), two designs, `@compmd/sandbox`, the Mermaid pack and the guide. Once approved, CMD-374 is re-scoped to it; it now waits on CMD-377.

Untested, and marked so: how much Mermaid's `securityLevel: "strict"` filters styles in diagram code, and why the class diagram stayed 3px narrower with the double render.
