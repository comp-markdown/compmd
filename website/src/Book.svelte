<!-- A book of the guides as one page, for its PDF: rendered on the server only, typeset as a book (`build/book.css`),
     printed, then removed. Its title page, its front matter (the contents), then each chapter. -->
<script lang="ts">
  import type { BookProps } from "../build/book.ts";

  let { title, purpose, version, contents, chapters, online }: BookProps = $props();
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<article class="spec guide book-print" id="book">
  <header class="cover">
    <div>
      <h1 class="title">{title}</h1>
      <p class="purpose">{purpose}</p>
    </div>
    <p class="imprint">
      For the <code>@compmd</code> packages, version {version}.<br />
      The online edition, which follows the packages as they change: <a href={online}>{online}</a>
    </p>
  </header>
  <section class="front">
    {@html contents}
  </section>
  {#each chapters as chapter, index (index)}
    <section class={["chapter", index === 0 && "first"]}>
      {@html chapter}
    </section>
  {/each}
</article>
