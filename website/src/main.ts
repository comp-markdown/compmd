import { hydrate, mount } from "svelte";
import App from "./App.svelte";
import "./base.css";
import { countAfterLoad } from "./count.ts";
import "./site.css";

// The built page holds the prerendered App, which hydrates; the dev server's holds none.
const target = document.getElementById("app");
if (target) (target.firstElementChild ? hydrate : mount)(App, { target });
countAfterLoad(window);
