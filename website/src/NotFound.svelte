<!-- The page a wrong address gets (`404.html`, which GitHub Pages serves for any): the site's header and search, and
     the places to go from. Served at the wrong address itself, at any depth, so its links start at the site's root. -->
<script lang="ts">
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";

  /** The site's root, from any address on it. */
  const ROOT = "/";
</script>

<svelte:head>
  <title>Not found · CompMD</title>
  <meta name="robots" content="noindex" />
</svelte:head>

<Header root={ROOT} />

<main class="wrap not-found">
  <h1>Nothing here.</h1>
  <p>There's no page at this address: it may have moved. Search the site from the header, or start from one of these.</p>
  <ul>
    <li><a href={ROOT}>CompMD</a>, what it is, with examples to try</li>
    <li><a href="{ROOT}spec/">The spec</a></li>
    <li><a href="{ROOT}guides/">The guides</a>, for writing documents and for building with CompMD</li>
    <li><a href="{ROOT}playground/">The playground</a></li>
  </ul>
</main>

<Footer />
