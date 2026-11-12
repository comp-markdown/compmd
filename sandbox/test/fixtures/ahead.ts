// A pack's source components, which the tests' setup compiles ahead as `@compmd/vite` does, and the tests give the
// processor's way: by their place, with their code.

/** A JSX component that sets `result` once it has rendered. */
export const AHEAD_JSX = 'import { useEffect } from "react";\nexport default function Ahead({ compmd }) {\n  useEffect(() => { compmd.set("result", JSON.stringify("compiled ahead")); }, []);\n  return <b>ahead</b>;\n}\n';

/** A Svelte component that sets `result` to its paragraph's colour, which its own styles give. */
export const AHEAD_SVELTE = '<script>\n  let { compmd } = $props();\n  let p;\n  $effect(() => { compmd.set("result", JSON.stringify(getComputedStyle(p).color)); });\n</script>\n<p bind:this={p}>dial</p>\n<style>p { color: rgb(0, 128, 128); }</style>\n';
