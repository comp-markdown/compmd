---
md:
  import:
    - from: ./dist/model-viewer.js
      names: [model-viewer]
  export:
    pack: "@spike/model-viewer"
    version: "0.1.0"
    names: [model-viewer]
---

# @spike/model-viewer

`model-viewer` shows the 3D model at `src` with [@google/model-viewer](https://www.npmjs.com/package/@google/model-viewer) 4.4.0. A reader without it sees the label, which is also the model's `alt`.

<!-- compmd-pack:generated from @google/model-viewer 4.4.0's types and compmd.wrap.json; edits between these comments are overwritten at each build -->
```compmd component=model-viewer signature
params:
  src: string
  alt: string?
  poster: string?
  cameraControls: boolean = false
  autoRotate: boolean = false
```
<!-- /compmd-pack:generated -->
