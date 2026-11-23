## (the review) · round 1 · t1
- [x] **Hakan Serce** · 2026-11-21T13:35:58Z · c1
  Can you give a quick summary/walkthrough of processor changes, it changed a lot more than I anticipated as part of this
  - **claude:79612601** · 2026-11-21T13:36:37Z · c2
    The processor part is 391 lines added in 18 files, most of them small. By the design's decisions:

    **Decision 6, the new params (about 110 lines, the most files but the least logic).** Each new param has to exist at every stage that knows params:
    - `validate/universal.rs`: the new params (`border-inline-start` and the other three sides, `shadow`, `rotate`, `stretch-link`) with their types and breakpoint rules.
    - `values.rs`, `values/forms.rs`: the two new value types: `shadow` (tokens `none s m l`) and `angle` (`Form::Angle`, at most 15 degrees either way).
    - `cel/types.rs`, `tree/definitions.rs`: `shadow` and `angle` as type names a signature can declare.
    - `expand/params.rs`: the cascade, where a caller's `border` covers the fence's four sides as `padding` covers its sides, with a test.

    **Decision 3, `stretch-link` and W124 (20 lines).** `diagnostic.rs` gets W124, and `expand/layout.rs` reports it when a box with `stretch-link` holds no link (`holds_link`).

    **Decision 2, `md.fallback` (about 90 lines).**
    - `expand/env.rs`: `Slots` carries the invocation's fallback.
    - `expand/components.rs`: `fallback_text()` finds it on a leaf or inline invocation.
    - `expand/text.rs`: placing it, as inline content or a paragraph (`placed_fallback`), and as plain text inside an image's alt (`image_alt`, `plain_text`).
    - Alt text can interpolate now. That needed `parse/inlines.rs` and `tree.rs` (`Image.alt_parts`, as URLs have `url_parts`), plus `tree/shift.rs` and `validate/body.rs`, which check the paths in it.
    - `validate/scope.rs`: `md.fallback` is known in a definition without slots, and is F014 in one with slots.

    **Decision 5, `md.slots` (about 35 lines).** `validate/scope.rs` types `md.slots` as a record of the declared slots. `expand/components/compmd.rs` gives its value (`given_slots`) through `with_md` in `env.rs`.

    **Decision 4, the inline invocation's box (about 70 lines in `expand/components/compmd.rs`).** `inline_box` and `box_params`: an inline use of a definition fenced with params becomes a `componentInline` named `box`. Its params are the fence's under the caller's, with layout params left out, and the paragraph's content goes in its `fallback`. With no params for a box, it is the paragraph's content alone, as before. `tests/expand/components.rs`: the badge test lost its fence params, which now make a box, and there's a new test for an inline box.

    **Decision 1, a destination starting with a param resolves against the caller.** No processor code changed in CMD-360: nothing rebased any destination then, so the caller's base was already what came out. `expand/param-destination-base` records that. CMD-362 (CR-42) adds the real resolution, and that's where the code for it is.

    So decision 6 is wide but shallow, and decisions 2 and 4 are the real logic.
