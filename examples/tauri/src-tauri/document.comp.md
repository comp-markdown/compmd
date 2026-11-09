---
md:
  import:
    - from: "@comp-markdown/mermaid@0.1"
      names: [mermaid]
  inputs:
    origin: {type: string, default: ""}
---

# CompMD in a Tauri app

This page is a CompMD document, processed by the app in Rust and rendered by `@compmd/sandbox`. Its components run in frames that `tauri-plugin-compmd` serves from a scheme of their own.

A diagram, drawn by the Mermaid pack in a strict frame:

```mermaid {alt="The document goes to the processor, then the sandbox, then the page"}
graph LR
  Document --> Processor --> Sandbox --> Page
```

Code, highlighted by the frames' Shiki:

```rust
fn main() {
    println!("Hello from a CompMD document");
}
```

A component the document defines itself, in a strict frame of its own, which says where it runs, and whether it can reach the app:

:: probe[The probe couldn't run] {result=origin}

```jsx component=probe runtime=react@19
import { useEffect } from "react";

/** What the frame's code can see: its origin, and whether Tauri's IPC is there for it to call the app. */
const seen = () => ({ origin: self.origin, tauri: "__TAURI_INTERNALS__" in self || "__TAURI__" in self });

export default function Probe({ compmd }) {
  useEffect(() => compmd.set("result", JSON.stringify(seen())), []);
  const { origin, tauri } = seen();
  return <strong>Drawn in a strict frame of the origin {String(origin)}, {tauri ? "with" : "without"} Tauri</strong>;
}
```

```compmd component=probe signature
params:
  result: input<string>
```
