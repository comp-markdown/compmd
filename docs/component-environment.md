# What a component may assume about its box: inline styles apply

**Status: approved (2026-11-22, CR-64), CMD-377; built by CMD-374.** Whether a component may write inline styles in its box, and whose job it
is that they apply. Found when an app mounted the Mermaid pack's component in its own page, whose content security
policy refuses inline styles (`style-src 'self'`): the diagram's styles were refused, and since Mermaid measures its
labels while it renders, its nodes came out the wrong size. The host contract (spec §7.4) says what values a component
gets (params, slots, scheme, type); it says nothing about the document its box is in. Mermaid assumed inline styles
work there, as most libraries that draw in a page do, and nothing said it could or couldn't.

**Terms.**

- An **inline style** is CSS a script or markup puts in the document itself: a `style` attribute (in markup, or set
  with `setAttribute("style", …)`), or a `<style>` element. A page's `style-src` refuses both unless it has
  `'unsafe-inline'`. Styles set through the CSSOM (`element.style.width = …`, a constructed stylesheet in
  `adoptedStyleSheets`) aren't inline styles, and no `style-src` refuses them.
- A component's **environment** is what it may assume about the document its `target` is in, beyond the values the
  contract gives it.
- The **environment promise** is the spec stating one such assumption, and making the processor responsible for it.
- A **strict page** is a host's page whose policy refuses inline styles.

## Decisions

1. **The contract promises that inline styles apply in the box's document.** §7.4 gains one point: in the document
   `target` is in, a `style` attribute and a `<style>` element the component writes apply, and a processor MUST
   provide that wherever it mounts a component. A component, and any library it uses, styles what it draws as it
   would in any page. Nothing is declared, and no component is written differently for a strict page.

2. **The promise is the processor's to keep, whatever the trust level.** How it keeps it follows from where the
   component runs (spec §15), and nothing about the levels changes:

   | Where the component runs | How the promise is kept |
   | --- | --- |
   | A sandbox's frame (strict, permissive) | The frame's own policy allows inline styles. `@compmd/sandbox`'s frames do today (`style-src 'self' 'unsafe-inline'`, `src/build/policy.ts`). |
   | The host's page (trusted) | The page allows inline styles where components run; or the host keeps the component in a frame. |

   So a registry pack, which is never trusted (§15), is unaffected by its host's policy: it runs in a frame in both
   page modes. Only a host that mounts components in its own strict page has something to do.

3. **The promise covers inline styles, and nothing else for now.** Not that the box is laid out when the component
   draws: §7.4 already says a component may be laid out hidden or at zero width, and lays itself out again when it
   is resized. Not fonts: the box's type and the document's fonts have their own rules (§7.4, §9.4). Other
   assumptions (compiling WebAssembly, workers) are added to the promise when a component needs one, each with its
   own decision.

4. **A frame is the preferred way to keep the promise in a strict page; a looser page is the host's own choice.**
   The spec says a host SHOULD NOT loosen its page's policy only to mount a component there, and the guide says why
   (risks 1 and 2). No frame for trusted code is built now (rejected alternative G).

5. **Trusted code puts no CSS from a param in the page.** §15 already says trusted code treats its params as
   untrusted input, with "no HTML from a param". It gains "and no CSS": a param's text in a `style` attribute or a
   `<style>` element of the host's page is a document's author styling the application (risk 2).

6. **The host is told when its page breaks the promise.** `@compmd/sandbox`'s host runtime listens for the browser's
   `securitypolicyviolation` event while trusted components are mounted, and reports each refused inline style
   through a callback of its own, `refusedStyle` (the view's event of that name): the component whose box holds what
   was refused, when one does, the directive, and the browser's sample. It probes nothing, so a page that keeps the
   promise reports nothing. It is not a diagnostic of the spec, since the document is not at fault, nor an `error`,
   which means a use shows its fallback.
   - *Changed while building it (2026-11-22, CMD-374):* first written as a refusal inside a trusted component's box,
     through `error`. A library that measures does it in a temporary element in the page's body (Mermaid without a
     container), which is removed before the browser reports the refusal, against the document; so a refusal outside
     every box is reported too, naming no component, rather than named by a guess at which component was drawing.

7. **The Mermaid pack carries no workaround.** Its `htmlLabels: false` (0.1.4, not released) is reverted: it gave up
   HTML labels on every page to serve pages that break the promise, and didn't fix them (below). The pack's test of
   an in-page mount under a strict policy is removed, since that is no longer a case the pack supports.

8. **`draw` keeps its CSSOM transfer.** `@compmd/pack/component`'s `draw` parses a drawing given as markup inert and
   brings its styles in through the CSSOM (CMD-374's first commit). Nothing depends on it once the promise holds,
   and it costs nothing where it does; a drawing that never measures still shows right in a page that breaks the
   promise.

## What was found

- **The CSSOM transfer is not enough for a library that measures.** With the transfer in place and SVG labels, a
  three-node flowchart's nodes matched a page that allows inline styles. A wider comparison, of every shape of four
  diagrams against such a page, in Chrome and WebKit, did not:

  | Diagram | In a strict page |
  | --- | --- |
  | Sequence | the same |
  | Flowchart | its nodes the same, its whole width 577px against 552px |
  | Class | its boxes 165px wide against 181px |
  | Mindmap | its shapes up to about 6% smaller (the root's circle 102px against 109px) |

  Mermaid writes a `<style>` and `style` attributes in the page while it lays a diagram out, and measures with them;
  the transfer only sees the finished markup.
- **Most libraries that draw write inline styles while they run**, not only in what they return: a map, an editor, a
  chart that reacts. A fix on the finished markup can't reach them.
- **The frames already keep the promise**, and a registry pack always runs in one. The report came from a page that
  mounted a registry pack's module itself, which none of the three trust levels describes.

## Work needed

Spec changes and every push need the maintainer's go.

1. **Spec.** §7.4: the promise (decision 1) and who keeps it (decisions 2 and 4). §15: "no CSS from a param" for
   trusted code (decision 5); in the informative note on browsers, that a sandbox's policy allows inline styles, and
   that a strict sandbox's closed network is what makes that safe. `spec/CHANGELOG.md`, under Unreleased. No
   conformance cases: the set compares trees, and what a processor that renders must do is not tested by it (§13.1).
2. **Designs.** [Running components in the sandbox](running-components-in-the-sandbox.md): in host mode, a trusted
   use needs the host's page to keep the promise, and decision 6's report. [The Mermaid pack](mermaid-pack.md): the
   pack relies on the promise.
3. **`@compmd/sandbox`.** The report of decision 6, with a browser test in Chrome and WebKit: a trusted component
   that writes an inline style in a strict page is reported by name, one written in the page's body with no name,
   and nothing in a page that allows them. A test
   that both frames keep the promise under a strict host page (a `style` attribute and a `<style>` element apply),
   beside `test/strict-style.test.ts`, which checks what CompMD itself puts in such a page.
4. **The Mermaid pack.** Revert `htmlLabels: false`; remove `test/strict-style.test.ts` and `test/strict-style/`;
   the README's 0.1.4 note, the pack's version and `compmd-packs/versions.json`.
5. **The guides.** The Developer's Guide, "Your page's policy" (`guides/developers/11-the-sandbox-closer.md`): what
   CompMD itself puts in the page needs no inline styles, as now; a component's drawing may, so a strict page keeps
   its components in frames, or allows inline styles knowing risks 1 and 2. Where the guide writes a trusted
   component: no CSS from a param.
6. **Checks.** `cargo make ci`, and `cargo make site` for the guide and the spec.

## Risks

The risks are bounded by trust, which the promise doesn't change: the host's configuration sets each pack's level, a
document can never raise it, and only a folder pack built with the application may be trusted, never a registry pack
(§15). Risks 1 and 2 arise only where a host mounts its own trusted components in its own page; sandboxed code gains
nothing (risk 4).

1. **The promise can push a strict host to loosen its own page.** With inline styles allowed, whoever gets markup
   into the application's page can draw over its interface, and read attribute values out through requests a style
   makes (`url()`), as far as the page's other directives let them go. Allowing only `style` attributes
   (`style-src-attr`) is much weaker for an attacker, since an attribute has no selectors, but it isn't enough for
   a library that also writes a `<style>`, as Mermaid does. Decision 4 and the guide answer this: keep the component
   in a frame.
2. **A document's CSS in the host's page.** A trusted component's params come from documents, which may be someone
   else's. A component that passes a param into its styles, or a library that reads styles from its input, puts a
   document author's CSS in the application's page: it can draw outside its box and make requests. Mermaid's
   diagram code carries styles (`style`, `classDef`); how much its `securityLevel: "strict"` filters was not tested.
   In a frame the same CSS is contained: no network, and clipped to its box (§7.4). This is the reason a library
   that takes styles from its input belongs in a frame, and the reason for decision 5.
3. **The promise binds every implementation.** Its wording is scoped to the document `target` is in, so a processor
   without a browser, which renders fallbacks (§7.4), promises nothing by it.
4. **Sandboxed code gains nothing.** The frames allow inline styles today; a strict sandbox has no network for a
   style to reach (§15, rule 2), and a permissive one reaches only the origins its pack was given.

## Alternatives rejected

- **A. Accept the browser's refusals as harmless, and document them.** First chosen, on a test that compared only a
  flowchart's nodes. The wider comparison showed the refusals change the drawing (above).
- **B. Render twice**, the second time with the first render's stylesheet adopted on the page, so Mermaid measures
  with its styles. Tried: the flowchart, the sequence diagram and the mindmap then match; the class diagram stays
  3px narrower (not traced; probably `style` attributes, which the stylesheet doesn't carry). It doubles each render, fixes one library, and
  leaves the refusals logged.
- **C. Hook the library's style writes**, turning them into CSSOM writes while it renders. It patches DOM methods in
  the host's page for the length of a render, and depends on how one version of one library writes its styles.
- **D. Components style only through the CSSOM**, as a rule of the contract, with or without a stylesheet the
  contract gives (`adopt(css)`). Consistent, and it works under any policy, but it rules out most libraries that
  draw: a pack can't make Mermaid, a map or an editor comply.
- **E. Components declare their kind** (a finished drawing that never measures; one that measures; one that keeps
  writing styles), and the host picks a mechanism for each. It moves every host's question into every pack, adds a
  declaration to the signature, and its middle kind needs alternative F.
- **F. Draw in a frame, show in the page:** the component draws in a document that allows inline styles, and the
  finished markup is shown in the host's page through the CSSOM. A pack can't make such a document itself, since a
  frame from `srcdoc`, `about:blank`, `blob:` or `data:` inherits its maker's policy; so the host must serve it.
  Markup from a sandbox would cross into the host's page, which then has to sanitize it, and the drawing leaves the
  box §15 counts on. It serves only finished drawings.
- **G. A frame for trusted code**, on the application's origin with a looser style policy, so a strict page keeps
  the promise for its own components without loosening itself. A second document of the application's with a weaker
  policy, mounting through messages where trusted code mounts directly today, for a case no host has asked for: a
  host with a strict page lists such a pack as permissive, or keeps it strict. To be reconsidered when one does.
- **H. A nonce for the component's styles.** A nonce covers a `<style>` element, not a `style` attribute, and a
  library doesn't put one on what it writes.
- **I. Ask Mermaid to render without inline styles.** Worth asking, and the only way Mermaid itself stops needing
  them, but it is theirs to schedule, and the next library has the same need.
