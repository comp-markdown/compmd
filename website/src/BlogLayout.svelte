<!-- The blog's layout, as `@compmd/blog` takes one (`build/blog.ts`): the site's header and footer around the place each
     page of the blog goes, its text in the guides' column and type. One layout for every page of the blog, at any depth,
     so its links start at the site's root. -->
<script lang="ts">
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";

  interface Props {
    /** What goes where each page's content is: the blog tool's marker for it. */
    content: string;
  }

  let { content }: Props = $props();

  /** The site's root, from any page of the blog. */
  const ROOT = "/";
</script>

<Header root={ROOT} current="blog" />

<main class="wrap reading blog">
  <div class="text">
    <div class="spec">
      {@html content}
    </div>
  </div>
</main>

<Footer />
