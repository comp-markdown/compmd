<!-- A version of the spec as a page: rendered on the server only, as static HTML; `spec.ts` adds the scroll-spy and the
     phone menu. The contents are beside the text, every section and subsection. -->
<script lang="ts" module>
  /** A version of the spec to link to. */
  export interface VersionLink {
    label: string;
    href: string;
    current: boolean;
  }
</script>

<script lang="ts">
  import type { Heading } from "../build/spec.ts";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";

  interface Props {
    /** The spec as HTML, from `renderSpec`. */
    html: string;
    toc: Heading[];
    /** The path from this page to the site's root. */
    root: string;
    /** Its version, `1.0`; absent for the draft. */
    version?: string | undefined;
    /** Every version of the spec, this one among them. */
    versions: VersionLink[];
    /** The spec's file on GitHub, at this version. */
    source: string;
  }

  let { html, toc, root, version, versions, source }: Props = $props();
  const title = $derived(version === undefined ? "CompMD specification, version 1 (draft)" : `CompMD specification, version ${version}`);
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<Header {root} current="spec" />

<div class={["status", version === undefined && "draft"]}>
  <div class="wrap">
    <p>
      {#if version === undefined}
        <strong>Draft.</strong> This is the editor's draft from <code>main</code>: it changes as the spec does, and a change can still break documents.
      {:else}
        <strong>Version {version}.</strong> Released, and never changed after.
      {/if}
      <a href={source}>The text on GitHub</a>
    </p>
    {#if versions.length > 1}
      <nav aria-label="Versions of the spec">
        {#each versions as link (link.href)}
          <a href={link.href} aria-current={link.current ? "page" : undefined}>{link.label}</a>
        {/each}
      </nav>
    {/if}
  </div>
</div>

<div class="wrap reading">
  <nav class="toc" aria-label="Contents">
    <button type="button" class="toc-toggle" aria-expanded="false" aria-controls="toc-list" hidden>Contents</button>
    <ol id="toc-list">
      {#each toc as heading (heading.id)}
        <li class="level-{heading.level}"><a href="#{heading.id}">{heading.text}</a></li>
      {/each}
    </ol>
  </nav>
  <article class="spec">
    {@html html}
  </article>
</div>

<Footer />
