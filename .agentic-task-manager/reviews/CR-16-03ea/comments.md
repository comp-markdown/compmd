## spec/compmd-spec-v1.md:1661 · round 1 · t1
> - *Informative:* in a browser, a cross-origin iframe with `sandbox="allow-scripts"` and a content security policy that allows only the processor's runtimes and libraries and the pack's own files; on a server, an isolated JavaScript runtime with no I/O. Such a policy may allow compiling WebAssembly (`'wasm-unsafe-eval'`), as for a highlighter's regex engine: WebAssembly has no I/O of its own and reaches only what the frame's scripts reach.

- [ ] **Hakan Serce** · 2026-11-19T10:07:58Z · c1
  why do we need to mention regex here?
  - **claude:79612601** · 2026-11-19T10:08:57Z · c2
    We don’t: it was an example from our own use, and the spec shouldn’t name an implementation’s parts. Removed from the note and its changelog entry; the sentence now says only that the policy may allow it and why that is safe.
