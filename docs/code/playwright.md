# Playwright

The repository's own rules for browser tests with Playwright: what a test asserts, how it waits, and what it may cost.

## What a test asserts

- **What a user sees and does.** A test acts as a user would (click, type, press keys, choose from a menu) and asserts what they would see: text, roles and labels, which element has focus, what is enabled. Find elements as a user does, by role and accessible name (`getByRole("button", { name: "Run" })`), then by label or text; a test ID only where nothing a user sees names the element.
- **Computed styles only where the style is the behaviour:** the theme's colours in light and dark, a hidden element that must stay hidden, the colour scheme a frame follows. Not to pin a look.
- **Geometry only where layout is the point:** a page no wider than the window, a frame as tall as its content. Each such check says its tolerance (`within 1px`) and why.
- **No screenshot comparison by default.** A screenshot fails on any change of fonts, antialiasing or browser version, so its baseline gets updated until the test passes, and then it guards nothing. It is kept for a few critical places only, agreed with the maintainer and listed here. An updated baseline is reviewed like code.

*Critical places with a screenshot:* none yet.

## When a check fails

- **Find why before changing the test.** A failing layout or geometry check is a bug until shown otherwise: "the page is 1px wider than the window" on CI's Linux runner was a header that overflowed with Linux's fonts, fixed in the CSS, not in the check.
- **A check that fails only on CI's machine is reproduced there** (`cargo make linux`), with its fonts and browser, before anything changes.
- Never loosen a tolerance, raise a timeout or skip a test to make CI green without saying so in the commit and agreeing it.

## Waiting

- **Wait for a condition, never for time.** Use Playwright's web-first assertions, which retry until they hold (`await expect(locator).toHaveText(…)`), or `expect.poll` for a value outside the page. No `page.waitForTimeout`, no fixed `setTimeout` before an assertion: a wait that holds alone fails when tests run at once on CI's four cores.
- **Timeouts are for a load, not a feature.** A condition that holds within a second alone can take ten while every test runs; give a load (a browser, a bundle, a parser) the time it takes then, and the test its own limit above it.
- A test asserts the last thing it waits for: after a render reports `rendered`, wait for the report, not for a moment after the text appears.

## Cost

Browser tests are CI's slowest part, most of it in setup, not in the checks.

- **One browser per file,** launched in `beforeAll` and shared by its tests, each in a new page or context of its own.
- **A page that no test changes is opened once** and shared, by the first test that needs it (`expect.poll` can't run in a hook). A test that changes it puts it back, or opens its own.
- **An app is built or served once per file,** not once per group of tests.
- Each test still passes alone (`-t <name>`), so a shared page or server is opened on demand, not by a test before it.

## Browsers

- **Chrome and WebKit:** what runs in a page is tested in both. WebKit on Linux (CI) and on macOS differ in fonts and some timing.
