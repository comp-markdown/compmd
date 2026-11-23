+++
id = "01a1fb26-cc23-737a-ae99-1ed296263b12"
number = 38
title = "The snapshot test loads the site without the components’ analysis"
status = "approved"
tasks = ["CMD-358"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T22:49:46Z
updated = 2026-11-20T23:05:18Z
+++

The website's snapshot test timed out in its `beforeAll` (vitest's 10 s hook limit) while ci ran every package's tests at once. The setup now does less, rather than getting a longer limit.

## The cause

`beforeAll` runs `loadApp` on the website only for its `request`. Timed alone, `loadApp`'s first call takes 553 ms, of which 322 ms is loading TypeScript, which it needs only for the listed packs' component analysis (W104). Under ci's load the first call grew past 10 s.

## The fix

- `loadApp(root, given, { analysis: false })` (`LoadOptions`, exported): it leaves the analysis out, and so loads neither TypeScript nor Svelte's compiler. The default stays `true`, so the plugin and the bundle still warn.
- The snapshot test asks for it: it checks the rendering, not the warnings.

## Tests

`app.test.ts`: *leaves the analysis out when asked, with the same request*: no W104, and the same request as with the analysis. The snapshot test passes (785 ms for the file). I didn't make the timeout happen on demand: the timing above is the evidence, and the first call now skips the slowest part.
