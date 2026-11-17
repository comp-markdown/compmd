# Remote images, asked about

- **Status:** Approved and built (2026-11-17): `ImageGate` and `originQuestion`'s kinds in `@compmd/sandbox`, the refused image and `imagesAllowed` in `@compmd/render`, `allow-origin` and `images-allowed` in the frame protocol.

A document's images with a remote URL are fetches at view time (spec §15): a processor asks the reader about the
image's origin, with the same answer as for data, and shows the alt text until the origin is allowed, with a way to
allow it. This is how `@compmd/render` and `@compmd/sandbox` do it, in frame mode and in host mode.

## Decisions

1. **One gate, the image source.** Every image the renderer draws already goes through an `ImageSource`
   (`render/src/images.ts`): the URL to show for a `src`, or `undefined`, and the alt text shows meanwhile. The sandbox
   gives one in both modes now: in host mode too, where the page loads images itself today. A relative `src`, one of
   the host's own schemes (`ownSchemes`, loaded through `ownImages`, spec §15) and one of a configured origin pass as
   they are; a remote one of an origin the reader allowed passes; any other is asked about first.
2. **The answers are the data's.** The gate uses `origins.ts` as it is: `OriginAnswers` (by default `localStorage`,
   under `compmd-origins`), `AskOrigin` (by default the host page's `<dialog>`), the open questions shared by origin,
   so a page that needs one origin for data and for three images asks once. A host passes the same `answers`, `ask`
   and configured origins it passes `loadData`; `useCompmd` and `createSandbox` take them as options.
3. **The question's words name images.** `originQuestion(origin, kinds)` says "load data", "show images" or "load
   data and show images" from what the page needs of the origin, and what the origin learns: that the page was
   opened, and for data the values picked.
4. **The question is asked when the image is first drawn**, not when the reader clicks: the spec's "the first time a
   document needs an origin", as for data. A page with images from an origin the reader hasn't decided on opens the
   dialog at once.
5. **Refused, the image is its alt text with a button.** `ImageSource` can answer `{ refused: origin }`. The renderer
   then draws the alt text (the image's text, or "Image" without one) and a button, "Show images from
   example.com", which calls the surface's `allowOrigin(origin)`. In host mode that is the gate: it forgets the
   answer and asks again. In frame mode the frame asks its parent (below).
6. **A frame asks its parent, never draws the question.** The button in a page frame sends `allow-origin` with the
   origin; the host's page asks with its own dialog, and only for an origin of an image it sent that frame
   (`PageImages` knows them), so code in the frame can't raise a question about anything else. It asks at most once
   per origin while a question is open, and not again for an origin the reader refused in the last minute, so a frame
   can't put the dialog up again and again.
7. **Allowed, the images come.** When the reader allows an origin, the gate tells every view drawing images of it,
   and the renderer asks its `ImageSource` again for those images: in host mode the `<img>` gets its `src`, in frame
   mode the parent fetches the file as now (`PageImages.file`, no credentials, no `Referer`).
8. **No diagnostic.** A refused image is the reader's choice, not a mistake in the document: no F028 for it, unlike
   data, whose `fallback` and F028 the processor reports while validating.
9. **No reader, no question.** A host that renders without a reader (a build, a server) passes an `ask` that refuses,
   as for `loadData`: its pages show remote images only from configured origins. A host that wants them at build time
   configures their origins.
10. **A host that never fetches data still asks about images**, by default, as the spec says. One that doesn't want the
    dialog configures the origins, or passes its own `ask`.

## Changes

- `render`: `ImageSource` may answer `{ refused: origin }`; the image view draws the alt text and the allow button,
  and redraws when told an origin was allowed; the surface gains `allowOrigin`.
- `sandbox/src/origins.ts`: the gate (`imageGate`), shared with `allowOrigins`, and `originQuestion` by kind.
- `sandbox/src/images.ts`: `PageImages.file` goes through the gate; `allow-origin` from a frame, checked against the
  frame's images.
- `sandbox` host mode: `createRegion` gets the gate as its `images`.
- The protocol: `allow-origin` (frame to parent) and `images-allowed` (parent to frame), in `messages.ts` and the
  schema.
- Tests: both modes, an image of an unasked origin asks, refused shows the alt text and the button, allowed shows the
  image, a configured origin and a relative image never ask, a frame can't ask about an origin it wasn't sent.
- Docs: the sandbox README, `guides/developers/08` (trust) and the author's guide's images note.
