<!-- A book of the guides as one page, for its PDF: rendered on the server only, printed, then removed. Its contents page
     first, its title the book's, then each chapter, which starts a page of its own. -->
<script lang="ts">
  import type { BookProps } from "../build/book.ts";

  let { title, contents, chapters, online }: BookProps = $props();
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<article class="spec guide book-print" id="book">
  <p class="edition">The online edition, which follows the packages as they change: <a href={online}>{online}</a></p>
  {@html contents}
  {#each chapters as chapter, index (index)}
    <section class="chapter">
      {@html chapter}
    </section>
  {/each}
</article>
