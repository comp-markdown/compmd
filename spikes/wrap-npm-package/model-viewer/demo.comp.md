---
md:
  import:
    - from: "@spike/model-viewer@0"
      names: [model-viewer]
---

# @google/model-viewer, wrapped

A model served beside the test page (the glTF sample triangle), which a strict frame can't fetch:

:: model-viewer[A triangle] {src="./triangle.gltf" cameraControls=true autoRotate=true}
