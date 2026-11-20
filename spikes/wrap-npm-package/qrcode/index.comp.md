---
md:
  import:
    - from: ./dist/qr-svg.js
      names: [qr-svg]
  export:
    pack: "@spike/qr-svg"
    version: "0.1.0"
    names: [qr-svg]
---

# @spike/qr-svg

`qr-svg` draws a QR code of `value` as SVG with the library [qrcode](https://www.npmjs.com/package/qrcode) 1.5.4. A reader without it sees the label.

<!-- Not between `compmd-pack:generated` comments: a library has no props to read, so the author writes this signature, and the build leaves it. -->

```compmd component=qr-svg signature
params:
  value: string
  level: L | M | Q | H = M
  margin: number = 4
```
