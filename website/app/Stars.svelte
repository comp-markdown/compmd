<!-- A rating of `of` stars, `value` of them filled. As `stars`, a click sets the input `value` is bound to; as `rating`,
     `readonly`, it only shows a value, from data. -->
<script lang="ts">
  interface Props {
    value: number;
    of: number;
    readonly?: boolean;
    compmd: { set(param: string, value: unknown): void; fallback: { text: string } };
  }

  let { value, of, readonly = false, compmd }: Props = $props();
</script>

{#if readonly}
  <span class="stars" role="img" aria-label="Rated {value} of {of}">
    {#each { length: of } as _, i (i)}<span aria-hidden="true">{i < value ? "★" : "☆"}</span>{/each}
  </span>
{:else}
  <span class="stars" role="radiogroup" aria-label={compmd.fallback.text}>
    {#each { length: of } as _, i (i)}
      <button type="button" role="radio" aria-checked={i + 1 === value} aria-label="{i + 1} of {of}" onclick={() => compmd.set("value", i + 1)}>
        {i < value ? "★" : "☆"}
      </button>
    {/each}
  </span>
{/if}

<style>
  .stars button,
  .stars > span {
    padding: 0 2px;
    border: 0;
    background: none;
    color: var(--accent, #3557c4);
    font-size: 1.6rem;
    line-height: 1;
  }

  .stars button {
    cursor: pointer;
  }
</style>
