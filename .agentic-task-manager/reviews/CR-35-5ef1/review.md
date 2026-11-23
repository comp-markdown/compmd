+++
id = "01a1fb03-5d24-72c0-8e29-8012732a5ef1"
number = 35
title = "A frame records a component as mounted once its code has run"
status = "approved"
tasks = ["CMD-357"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T22:11:04Z
updated = 2026-11-20T22:12:52Z
+++

The O6 failure came from a bug in the observer, not in the test: a frame recorded a component as `mounted` before the component's code had run.

## The cause

A frame's components go through `Components`' stand-in mount (`#indirect`), which runs the component's code once that code has loaded, and again when it changes. The frame's `Mounts` wrapped that stand-in with `timed`, so `mounted` was recorded when the renderer called the stand-in. When the part comes before the code has loaded, which is the usual order (`code`, then `part`, at once), that's before the component's code runs. The `ms` was the stand-in's time, near 0.

In O6 the component's code patches `postMessage`, and the forged events ride on the next report after the patch. The only event the frame records after rendering is `mounted`, and its report goes out 100 ms later. When the module loaded within those 100 ms (alone, always), the report went after the patch. Under ci's load it went first, and no report followed. It failed four times in today's runs, in Chrome and WebKit.

## The fix

- `timed` moves to `src/timed.ts`, unchanged.
- `Components` takes the frame's observer and times each component's own mount as its code loads. So `mounted` comes once the code has run, with its time, and a remount after new code shows as `unmounted`, then `mounted` again, which wasn't recorded before.
- The frame's `Mounts` no longer gets the observer. The host's page's does, for its trusted components, whose mounts are their own code.

## Tests

`components.test.ts`, *records an instance as mounted once its code has mounted, not when its use is rendered, and a remount as another*. It fails without the fix (no `mounted`, `updated` or `unmounted` in the order the code ran) and passes with it. The sandbox's 517 tests pass in Chrome and WebKit, O6 among them. I didn't reproduce O6's own failure on demand: the unit test pins the order it depended on.
