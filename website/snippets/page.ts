import { init, process } from "@compmd/processor";
import { createPage } from "@compmd/render";

await init();

const text = `# Hello

::: box {padding=m border=subtle border-radius=m}
A box, written in **CompMD**.
::: /box
`;
const { expanded } = process({ path: "hello.comp.md", files: { "hello.comp.md": text }, now: new Date().toISOString() });
createPage(document.body).render(expanded);
