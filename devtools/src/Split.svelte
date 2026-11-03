<!-- The text and the views side by side, with a handle between them: drag it, or move it with the arrow keys; Home and
     End fold one side away, Enter opens it again, and the buttons on the handle do the same. The browser remembers
     where it was. On a narrow screen the two stack, and the handle goes. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { clamp, keyed, load, save, type Split } from "./split.ts";

  let { text, views }: { text: Snippet; views: Snippet } = $props();

  const storage = (() => {
    try {
      return window.localStorage;
    } catch {
      return undefined;
    }
  })();
  let split: Split = $state(load(storage));
  let dragging = $state(false);
  let container: HTMLElement | undefined = $state();

  function set(next: Split): void {
    split = next;
    save(storage, next);
  }

  function drag(event: PointerEvent): void {
    if (!dragging || !container) return;
    const box = container.getBoundingClientRect();
    set({ at: clamp((event.clientX - box.left) / box.width), folded: undefined });
  }

  const columns = $derived(split.folded === "text" ? "0fr" : split.folded === "views" ? "1fr" : `${String(split.at)}fr`);
  const rest = $derived(split.folded === "views" ? "0fr" : split.folded === "text" ? "1fr" : `${String(1 - split.at)}fr`);
</script>

<div class="split" class:dragging bind:this={container} style:--text={columns} style:--views={rest} data-folded={split.folded}>
  <section class="side text-side" aria-label="Text" hidden={split.folded === "text"}>
    {@render text()}
  </section>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="handle"
    role="separator"
    aria-orientation="vertical"
    aria-label="Split between the text and the views"
    aria-valuemin={0}
    aria-valuemax={100}
    aria-valuenow={split.folded === "text" ? 0 : split.folded === "views" ? 100 : Math.round(split.at * 100)}
    tabindex="0"
    onpointerdown={(event) => {
      if ((event.target as Element).closest("button")) return;
      dragging = true;
      event.currentTarget.setPointerCapture(event.pointerId);
    }}
    onpointermove={drag}
    onpointerup={() => (dragging = false)}
    onpointercancel={() => (dragging = false)}
    ondblclick={() => set({ ...split, folded: undefined })}
    onkeydown={(event) => {
      const next = keyed(split, event.key);
      if (!next) return;
      event.preventDefault();
      set(next);
    }}
  >
    <button
      type="button"
      class="fold"
      title={split.folded === "views" ? "Show the views" : "Fold the text away"}
      onclick={() => set({ ...split, folded: split.folded === "views" ? undefined : "text" })}
      aria-label={split.folded === "views" ? "Show the views" : "Fold the text away"}
    >
      <svg viewBox="0 0 8 12" aria-hidden="true"><path d="M6 1 1 6l5 5" /></svg>
    </button>
    <span class="grip" aria-hidden="true"></span>
    <button
      type="button"
      class="fold"
      title={split.folded === "text" ? "Show the text" : "Fold the views away"}
      onclick={() => set({ ...split, folded: split.folded === "text" ? undefined : "views" })}
      aria-label={split.folded === "text" ? "Show the text" : "Fold the views away"}
    >
      <svg viewBox="0 0 8 12" aria-hidden="true"><path d="m2 1 5 5-5 5" /></svg>
    </button>
  </div>
  <section class="side views-side" aria-label="Views" hidden={split.folded === "views"}>
    {@render views()}
  </section>
</div>
