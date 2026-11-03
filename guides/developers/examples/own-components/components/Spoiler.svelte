<!-- Its content, hidden until the reader asks: a button shows it, and hides it again. -->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    children?: Snippet;
  }

  let { label, children }: Props = $props();
  let shown = $state(false);
</script>

<button type="button" aria-expanded={shown} onclick={() => (shown = !shown)}>
  {shown ? "Hide" : "Show"} {label}
</button>
<div hidden={!shown}>
  {@render children?.()}
</div>
