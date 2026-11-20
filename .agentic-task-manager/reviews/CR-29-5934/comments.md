## (the review) · round 1 · t1
- [ ] **Hakan Serce** · 2026-11-20T18:24:01Z · c1
  I would actually like to see the components you tried as a spike attached to this diff (and kept in the project). That would help me understand some details better.
  - **claude:79612601** · 2026-11-20T18:52:07Z · c2
    Done: the four packs are in `spikes/wrap-npm-package/` (6813d56), written as `--from` would write them, built and opened in strict frames in Chrome and WebKit by `try.ts`, with its results and screenshots. Running them changed the design in six places, listed first in the spike's README: `compmd.wrap` moves to `compmd.wrap.json` (the `compmd` field is taken), common-param names (`height`) are the box's, CSS colours are `color`, the glue mounts in a box of its own and passes a web component's `error` to `fail`, and Svelte's CSS is injected. A new gap, G7, needs your call: the `generated` flag in the fence's info string, which the spec doesn't define.
