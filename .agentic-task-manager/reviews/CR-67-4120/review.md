+++
id = "01a20453-a4f8-710f-b9c7-8054fa734120"
number = 67
title = "A component's box: inline styles apply, the spec's promise, the sandbox's report and the Mermaid pack"
status = "approved"
tasks = ["CMD-374"]
author = "claude:9ea81469"
reviewers = []
created = 2026-11-22T17:35:20Z
updated = 2026-11-22T17:37:28Z
+++

CMD-374 builds the design approved in CR-64, [what a component may assume about its box](docs/component-environment.md). The task's commits are 652fb24 (`draw`'s CSSOM transfer, kept by decision 8), 6623c0f and 069d23c; the range also holds other tasks' commits made in between.

Please check:

- **Spec** (§7.4, §15, the changelog): the promise that inline styles apply in a component's box, kept by the processor wherever it mounts one; a host with a strict page keeps such components in frames, and SHOULD NOT loosen its page for them; trusted code takes no CSS from a param; the browser note says the frames allow inline styles, and why that's safe.
- **`@compmd/sandbox`**: `refusedStyle` (decision 6, changed as you chose: its own callback, naming the component when the refused element is in its box, and no name otherwise). `src/refused-styles.ts` watches each trusted mount's box while it is mounted, and listens only then. A mount `rendersSlots` marks is left unwatched, as `timed` leaves it. `test/refused-style.test.ts`, in Chrome and WebKit: nothing reported in a page that allows inline styles; under `style-src 'self'`, a trusted component's style reported by name, one written in the page's body with no name, and both frames applying a `style` attribute and a `<style>` element.
- **Mermaid pack**: `htmlLabels: false` reverted and its version back to 0.1.3 (0.1.4 was never tagged); the in-page strict test removed.
- **Designs and docs**: decision 6 updated with why it changed; the sandbox design's host mode; the Mermaid pack's design; the sandbox README; the Developer's Guide's "Your page's policy".

Checks: `cargo make ci` and `cargo make site`. One flaky test seen once, unrelated: the Shiki engine comparison in `src/highlight/highlight.test.ts` failed under a full run's load and passes alone.
