+++
id = "01a20a7d-24a8-724c-bc72-68f8151e5b1b"
number = 81
title = "Sandbox: the O3 test counts settled once"
status = "approved"
tasks = ["CMD-392"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T22:18:23Z
updated = 2026-11-23T22:36:35Z
+++

The observer's O3 test (the messages are the same with the observer as without) failed twice in a row under `cargo make ci`'s load while checking 0.10.0: one run had one `settled` more than the other, either way round. It passed alone each time.

`settled` comes as often as the page lays out, as `size` and `box` do, which the test already leaves out. Now the test counts `settled` once: it still compares that the sandbox settles in both runs, but not how many times.

`cargo make ci` passes with it.
