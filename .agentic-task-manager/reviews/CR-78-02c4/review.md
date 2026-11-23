+++
id = "01a2098b-3c6a-729a-86bd-6d94ea1002c4"
number = 78
title = "Spec: gradients in background (§9.2, §9.4)"
status = "approved"
tasks = ["CMD-387"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T17:54:09Z
updated = 2026-11-23T18:13:25Z
+++

The spec text for gradients in `background`, as the design approved in CR-77 says (bfd6c12).

- **§9.2's table:** `background` takes "a token, a CSS color, or a linear or radial gradient (section 9.4)".
- **§9.4, a new item, *Gradients*:**
  - the four functions and their grammar, written out;
  - what is F009: another function, a quote, a backslash, `;`, a brace, a conic gradient, fewer than two stops, or mixed logical and physical sides;
  - directions that follow the text's: logical `to` and `at` sides, and logical angles, mirrored in right-to-left text;
  - the 45° cut corner as an example;
  - W126 not checking a gradient, and a gradient printing as a color does.
- **`spec/CHANGELOG.md`:** an entry under *Unreleased*, not breaking.
- **The design** (`docs/background-gradients.md`): marked accepted, decision 4 no longer proposed.

The conformance cases go in with the processor's change (CMD-388), so CI stays green: a case accepting a gradient would fail against today's processor. `cargo make site` passes.
