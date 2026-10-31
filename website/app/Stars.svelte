<!-- A rating of `of` stars, `value` of them filled; a click sets the input `value` is bound to. -->
<script lang="ts">
  interface Props {
    value: number;
    of: number;
    compmd: { set(param: string, value: unknown): void; fallback: { text: string } };
  }

  let { value, of, compmd }: Props = $props();
</script>

<span class="stars" role="radiogroup" aria-label={compmd.fallback.text}>
  {#each { length: of } as _, i (i)}
    <button type="button" role="radio" aria-checked={i + 1 === value} aria-label="{i + 1} of {of}" onclick={() => compmd.set("value", i + 1)}>
      {i < value ? "★" : "☆"}
    </button>
  {/each}
</span>

<style>
  .stars button {
    padding: 0 2px;
    border: 0;
    background: none;
    color: var(--accent, #3557c4);
    font-size: 1.6rem;
    line-height: 1;
    cursor: pointer;
  }
</style>
