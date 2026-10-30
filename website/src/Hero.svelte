<!-- The hero's document beside its page. The page is the snapshot until the processor loads; then the text is
     editable and the page follows it, and the first error, if any, shows under the text. -->
<script lang="ts">
  import type { Diagnostic } from "@compmd/processor";
  import { onMount } from "svelte";
  import snapshots, { files } from "virtual:snapshots";
  import type { Live } from "./live.ts";

  const NAME = "launch.comp.md";
  const { text, html } = snapshots.hero;

  let page: HTMLElement;
  let live = $state<Live>();
  let diagnostics = $state<readonly Diagnostic[]>([]);
  const first = $derived(diagnostics.find((d) => d.severity === "error") ?? diagnostics[0]);

  onMount(() => {
    let destroyed = false;
    void import("./live.ts").then(async ({ liven }) => {
      const made = await liven(page, NAME, text, files);
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
      <span class="hint" aria-live="polite">{live ? "Edit the text: the page follows." : "Loading the processor…"}</span>
    </figcaption>
    <textarea
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
  <div class={["proof", "live", live && "ready"]}>
    <div class="compmd-page" bind:this={page}>{@html html}</div>
  </div>
</div>
