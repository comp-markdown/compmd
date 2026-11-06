<!-- The hero's document beside its page. The page is the snapshot until the processor and the sandbox load; then the
     text is editable, the page follows it, its covers and stars can be clicked, and the first error, if any, shows under
     the text. It renders through the sandbox in this page, as an app with components of its own does: the site's
     `stars` inline, the document's `shelf.js` in a frame. -->
<script lang="ts">
  import type { Diagnostic } from "@compmd/processor";
  import { onMount } from "svelte";
  import snapshots, { files } from "virtual:snapshots";
  import type { Live } from "./live.ts";
  import Renders from "./Renders.svelte";

  const NAME = "club.comp.md";
  const { text, html } = snapshots.hero;

  let sandboxed: HTMLElement;
  let live = $state<Live>();
  /** Whether the sandbox's page shows, with its components, in place of the snapshot. */
  let shown = $state(false);
  let diagnostics = $state<readonly Diagnostic[]>([]);
  const first = $derived(diagnostics.find((d) => d.severity === "error") ?? diagnostics[0]);

  /** The frontmatter lines left in view above the heading, so the reader sees there is some. */
  const ABOVE = 3;

  /** Scrolls the text so its heading is near the top, the end of its frontmatter above it. */
  function toHeading(area: HTMLTextAreaElement): void {
    const heading = text.split("\n").findIndex((line: string) => line.startsWith("# "));
    const line = Number.parseFloat(getComputedStyle(area).lineHeight);
    if (heading > ABOVE && line > 0) area.scrollTop = (heading - ABOVE) * line;
  }

  onMount(() => {
    let destroyed = false;
    void import("./framed.ts").then(async ({ keepInSandbox }) => {
      const made = await keepInSandbox(sandboxed, NAME, text, { ...files, [NAME]: text }, () => (shown = true));
      if (destroyed) made.destroy();
      else live = made;
    });
    return () => {
      destroyed = true;
      live?.destroy();
    };
  });
</script>

<div class="pair hero-pair">
  <figure class="manuscript">
    <figcaption>
      {NAME}
      <span class="hint" aria-live="polite">{live && shown ? "Edit the text, or pick a cover: the page follows." : "Loading the processor…"}</span>
    </figcaption>
    <textarea
      {@attach toHeading}
      aria-label="The document's text"
      spellcheck="false"
      readonly={!live}
      oninput={(event) => {
        if (live) diagnostics = live.edit(event.currentTarget.value);
      }}>{text}</textarea>
    {#if first}
      <p class="diagnostic" role="status">
        <code>{first.code}</code> line {first.range.start.line}: {first.message}
      </p>
    {/if}
  </figure>
  <Renders />
  <div class={["proof", "live", "framing", "hosted", shown && "ready", shown && "framed"]}>
    <div class="compmd-page" hidden={shown}>{@html html}</div>
    <!-- Not `hidden` until it renders, which would keep its frames from loading: it waits invisible over the snapshot. -->
    <div class="sandboxed" bind:this={sandboxed}></div>
  </div>
</div>
