+++
id = "01a1f8e2-ce3b-7593-ab5c-e4bb0d2d5b75"
number = 27
title = "A frame’s time counts only while the page’s thread is awake"
status = "approved"
tasks = ["CMD-346"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T12:16:15Z
updated = 2026-11-20T20:41:26Z
+++

Two browser tests failed once each while 0.9.0 was released: the Mermaid pack's test page in WebKit on Linux (CI), where two of six diagrams' frames had been removed, and the observer's O6 in Chromium (a local `cargo make ci`). Both passed again on the same code.

What to check:

- **The cause I fixed:** a parent removed a frame that hadn't said `ready` within 5 s of its document loading, or hadn't answered a ping within `timeout`, by the clock. A page and its frames may share a thread (WebKit; headless Chromium's frames in the page's process): while it is busy, with another frame's large module say, a frame can neither load nor answer, and the time counted against it.
- **The fix** (`protocol/awake.ts`, new): `afterAwake` counts time in ticks of 250 ms, so a busy stretch of any length counts as one tick. The never-loaded timers of `connectChild` and `connectCompiler` use it; the ping check counts unanswered pings (`silentFor`) in place of the clock since the last pong. A frame in a process of its own that really hangs is stopped as before, since the parent's ticks go on.
- **Shown for the mechanism:** `test/host.test.ts`, a page that keeps its thread busy for 6 s once a frame's document has loaded. Without the fix the frame is removed as never loaded, in Chrome and WebKit (I ran it against the old timers); with it, it draws.
- **Not shown for CI's failure itself:** the Mermaid test passed in the Linux container (`compmd-linux`, 9 s in WebKit) and on CI's rerun, so the original failure wasn't caught in the act. The Mermaid test's error now prints the page's diagnostics (7e25a18), which name why a frame was removed, should it come back.
- **O6:** its wait for a frame goes from 8 s to 12 s, within the 15 s test limit, for a frame that is only slow while every package's tests run at once.
- **Not looked at further:** whether 0.9.0's highlighting in the host's page adds a long task there; the timers no longer depend on it.

Checks: `cargo make ci` green.
