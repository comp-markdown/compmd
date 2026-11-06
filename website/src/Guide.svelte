<!-- A page of the CompMD guides: the shelf, a book's contents or a chapter, rendered on the server only, as static
     HTML; `spec.ts` adds the scroll-spy and the phone menu, as on the spec's page. Beside the text, the books, the one
     being read open to its parts and chapters, and its chapter to its sections; under the text, the chapters before
     and after it. -->
<script lang="ts" module>
  /** A chapter to go to. */
  export interface ChapterLink {
    title: string;
    href: string;
  }

  /** A book of the guides: its folder in `guides/` and its title. */
  export interface Shelved {
    name: string;
    title: string;
  }
</script>

<script lang="ts">
  import type { Part } from "../build/guide.ts";
  import type { Heading } from "../build/spec.ts";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import Meta from "./Meta.svelte";
  import { guidePdf } from "./site.ts";

  interface Props {
    /** The page as HTML, from `renderMarkdown`. */
    html: string;
    /** Its sections. */
    toc: Heading[];
    /** The path from this page to the site's root. */
    root: string;
    /** All the books, in order. */
    books: Shelved[];
    /** The book this page is in; absent for the shelf. */
    book?: string | undefined;
    /** The book's parts; none for the shelf. */
    parts: Part[];
    /** The chapter this page is; absent for a book's contents and the shelf. */
    current?: string | undefined;
    previous?: ChapterLink | undefined;
    next?: ChapterLink | undefined;
    /** The title of the page: the chapter's, the book's or the shelf's. */
    title: string;
    /** The version of the `@compmd` packages the guides are written for: the packages' own, as built. */
    version: string;
    /** What the page is about, for its head: its first paragraph. */
    description: string;
  }

  let { html, toc, root, books, book, parts, current, previous, next, title, version, description }: Props = $props();
  const guides = $derived(`${root}guides/`);
  const open = $derived(books.find((shelved) => shelved.name === book));
  /** The page's sections, for "On this page"; the shelf has none, its books are its page. */
  const sections = $derived(book === undefined ? [] : toc.filter((heading) => heading.level === 2));
</script>

<svelte:head>
  <title>{current === undefined || open === undefined ? title : `${title} · ${open.title}`}</title>
</svelte:head>

<Meta title={current === undefined || open === undefined ? title : `${title} · ${open.title}`} {description} />

<Header {root} current="guides" />

<div class="status">
  <div class="wrap">
    <p>
      <strong>{open?.title ?? "The CompMD guides"}.</strong> Written for the <code>@compmd</code> packages at {version}, and following them as they change.
      {#if open}<a href="{guides}{open.name}/{guidePdf(open.name)}" download>Download the book as PDF</a>{/if}
    </p>
  </div>
</div>

<div class={["wrap", "reading", book === undefined && "shelf"]}>
  {#if book !== undefined}
    <!-- The shelf has no list beside it: its books are the page. -->
    <nav class="toc book" aria-label="The guides">
    <button type="button" class="toc-toggle" aria-expanded="false" aria-controls="toc-list" hidden>Contents</button>
    <ol id="toc-list">
      <li class="level-2"><a href={guides} aria-current={book === undefined ? "page" : undefined}>The guides</a></li>
      {#each books as shelved (shelved.name)}
        <li class="level-2">
          <a href="{guides}{shelved.name}/" aria-current={shelved.name === book && current === undefined ? "page" : undefined}>{shelved.title}</a>
        </li>
        {#if shelved.name === book}
          {#each parts as part (part.title)}
            <li class="part">{part.title}</li>
            {#each part.chapters as chapter (chapter.number)}
              {#if chapter.slug}
                <li class="chapter">
                  <a href="{guides}{shelved.name}/{chapter.slug}/" aria-current={chapter.slug === current ? "page" : undefined}>{chapter.number}. {chapter.title}</a>
                  {#if chapter.slug === current}
                    <ol>
                      {#each toc.filter((heading) => heading.level === 2) as heading (heading.id)}
                        <li class="level-3"><a href="#{heading.id}">{heading.text}</a></li>
                      {/each}
                    </ol>
                  {/if}
                </li>
              {:else}
                <li class="chapter later"><span>{chapter.number}. {chapter.title}</span></li>
              {/if}
            {/each}
          {/each}
        {/if}
      {/each}
    </ol>
  </nav>
  {/if}
  <div class="text">
    <article class="spec guide">
      {@html html}
    </article>
    {#if previous || next}
      <nav class="turn" aria-label="Chapters">
        {#if previous}<a class="previous" href={previous.href}><span>Previous</span>{previous.title}</a>{/if}
        {#if next}<a class="next" href={next.href}><span>Next</span>{next.title}</a>{/if}
      </nav>
    {/if}
  </div>
  {#if sections.length > 0}
    <nav class="onpage" aria-label="On this page">
      <p>On this page</p>
      <ol>
        {#each sections as heading (heading.id)}
          <li><a href="#{heading.id}">{heading.text}</a></li>
        {/each}
      </ol>
    </nav>
  {/if}
</div>

<Footer />
