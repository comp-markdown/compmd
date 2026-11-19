+++
id = "01a1f1ab-7170-7533-84d4-1b47a32c9911"
number = 7
title = "Spike: code highlighting's cost in WebKit"
status = "approved"
tasks = ["CMD-252"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T02:38:27Z
updated = 2026-11-19T07:25:40Z
+++

The spike's test page, its runner, and its results: `spikes/highlight-perf/README.md` leads with what they support.

## What to check

- **The page** (`src/main.ts`): 440 TypeScript runs, each a `createPage` of its own, mounted within a screen of view and destroyed once out of it, as a virtual list's are; frames recorded with `requestAnimationFrame`, since WebKit has no long-task API.
- **The variants** (`src/highlighters.ts`, `src/shiki.ts`, `src/worker.ts`): Shiki per view or shared, three engines, the page's thread or a worker, eager or sliced, with or without a cache. Drawing is the sandbox's (`src/draw.ts`).
- **The runner** (`measure.ts`): Playwright's WebKit and Chromium, three phases, each variant three times.
- **The workspace**: `spikes/*` in `pnpm-workspace.yaml`, with no build, lint or test script, so CI's recursive runs leave it out; its Shiki packages at the sandbox's version, 4.5.0.

## The results

In WebKit, one Shiki per page with Oniguruma meets the host's targets (83 ms blocked on opening, its longest frame 133 ms, 10 ms over the skim); today's blocks 1.1 s on opening and 5.9 s over the skim. Precompiled grammars and a cache gain nothing; a worker keeps the page smooth but a run waits 1.1 s with the JavaScript engine. Apart from highlighting: hundreds of mounted pages in one document cost 43 s over the skim in WebKit, from a stylesheet per page.
