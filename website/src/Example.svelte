<!-- A document beside the page it renders: its snapshot, made live once it scrolls into view. A live example with
     inputs renders in this page; one with components in code renders in the playground's sandboxed frame. -->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Snapshot } from "../build/snapshots.ts";
  import { files } from "virtual:snapshots";

  /** The playground's frame, from this page; `framed.ts` loads only when it does. */
  const FRAME_URL = "playground/frame.html";

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
  let iframe: HTMLIFrameElement | undefined = $state();
  /** Whether the frame has rendered, and shows in place of the snapshot. */
  let framed = $state(false);
  /** Whether the page, live in this page, has replaced the snapshot. */
  let ready = $state(false);
  let close: (() => void) | undefined;

  onMount(() => {
    const observer =
      live === "page"
        ? new IntersectionObserver(([entry]) => {
            if (!entry?.isIntersecting) return;
            observer?.disconnect();
            void import("./live.ts").then(async ({ liven }) => {
              close = (await liven(page, name, snapshot.text, files)).destroy;
              ready = true;
            });
          })
        : undefined;
    observer?.observe(page);
    return () => {
      observer?.disconnect();
      close?.();
    };
  });

  async function frameLoaded(): Promise<void> {
    if (!iframe) return;
    const { renderInFrame } = await import("./framed.ts");
    close = await renderInFrame(iframe, name, snapshot.text, files, () => (framed = true));
  }
</script>

<div class="pair">
  <figure class="manuscript">
    <figcaption>{name}</figcaption>
    <pre>{snapshot.text}</pre>
  </figure>
  <div class={["proof", live === "page" && "live", ready && "ready", live === "frame" && "framing", framed && "framed"]}>
    <div class="compmd-page" bind:this={page} hidden={framed}>{@html snapshot.html}</div>
    {#if live === "frame"}
      <!-- Not `hidden` until it renders, which would keep it from loading: it waits invisible over the snapshot. -->
      <iframe
        bind:this={iframe}
        title="The rendered page, in a sandbox"
        sandbox="allow-scripts"
        loading="lazy"
        src={FRAME_URL}
        style:height
        onload={() => void frameLoaded()}></iframe>
    {/if}
  </div>
</div>
