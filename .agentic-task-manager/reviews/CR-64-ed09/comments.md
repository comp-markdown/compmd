## (the review) · round 1 · t1
- [x] **Hakan Serce** · 2026-11-22T17:10:29Z · c1
  The security risk is not unbounded, correct? So, the host still decides which packages are trusted, and which packages are not?
  - **claude:9ea81469** · 2026-11-22T17:10:55Z · c2
    Correct, it is bounded. Trust is unchanged: the host’s configuration sets each pack’s level, a document can never raise it, and only a folder pack built with the app may be trusted, never a registry pack (§15). Risks 1 and 2 arise only where a host mounts its own trusted components in its own page, and then only if it loosens that page (1) or such a component passes document text into its styles (2, which decision 5 forbids). Sandboxed code gains nothing (risk 4). One case the spec can’t bound: an app that bypasses the levels, as the reporting app did by mounting a registry pack’s module in its page itself. The Risks section now opens with this (d2f4f3c).
