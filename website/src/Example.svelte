<!-- A document beside the page it renders: its snapshot, made live once it scrolls into view. A live example with
     inputs renders in this page; one with components in code renders in a sandbox, as the playground does. -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Snapshot } from "../build/snapshots.ts";
  import { files } from "virtual:snapshots";

  interface Props {
    /** The document's file name, shown on its sheet. */
    name: string;
    snapshot: Snapshot;
    /** How it goes live: in this page, so the reader can change its inputs, or in the sandboxed frame. */
    live?: "page" | "frame" | undefined;
    /** The frame's height, for `live="frame"`, since a frame doesn't take its content's. */
    height?: string | undefined;
  }

  let { name, snapshot, live, height = "8rem" }: Props = $props();
  let page: HTMLElement;
  /** Where the sandbox puts the page's frame, for `live="frame"`. */
  let sandboxed: HTMLElement | undefined = $state();
  /** Whether the frame has rendered, and shows in place of the snapshot. */
  let framed = $state(false);
  /** Whether the page, live in this page, has replaced the snapshot. */
  let ready = $state(false);
  let close: (() => void) | undefined;

  /** Makes the example live: in this page, or in a sandbox; `live.ts` and `framed.ts` load only when one does. */
  async function liven(): Promise<void> {
    if (live === "page") {
      const { liven } = await import("./live.ts");
      close = (await liven(page, name, snapshot.text, files)).destroy;
      ready = true;
    } else if (live === "frame" && sandboxed) {
      const { renderInFrame } = await import("./framed.ts");
      close = await renderInFrame(sandboxed, name, snapshot.text, files, () => (framed = true));
    }
  }

  onMount(() => {
    const observer = live
      ? new IntersectionObserver(([entry]) => {
          if (!entry?.isIntersecting) return;
          observer?.disconnect();
          void liven();
        })
      : undefined;
    observer?.observe(page);
    return () => {
      observer?.disconnect();
      close?.();
    };
  });
</script>

<div class="pair">
  <figure class="manuscript">
    <figcaption>{name}</figcaption>
    <pre>{snapshot.text}</pre>
  </figure>
  <div class={["proof", live === "page" && "live", ready && "ready", live === "frame" && "framing", framed && "framed"]}>
    <div class="compmd-page" bind:this={page} hidden={framed}>{@html snapshot.html}</div>
    {#if live === "frame"}
      <!-- Not `hidden` until it renders, which would keep its frame from loading: it waits invisible over the snapshot. -->
      <div class="sandboxed" bind:this={sandboxed} style:height></div>
    {/if}
  </div>
</div>
