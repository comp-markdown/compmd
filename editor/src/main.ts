import { mount } from "svelte";
import { attachSearch } from "../../website/src/search.ts";
import App from "./App.svelte";

const target = document.getElementById("app");
if (target) mount(App, { target });
// The header's search, as on every page of the site.
attachSearch(document);
