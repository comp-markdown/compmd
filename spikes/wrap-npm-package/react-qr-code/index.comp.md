---
md:
  import:
    - from: ./dist/qr-code.js
      names: [qr-code]
  export:
    pack: "@spike/qr-code"
    version: "0.1.0"
    names: [qr-code]
---

# @spike/qr-code

`qr-code` draws a QR code of `value` with [react-qr-code](https://www.npmjs.com/package/react-qr-code) 2.2.0. It fills its box: `width=10rem` on the use sizes it. A reader without it sees the label, which is also the drawing's title.

<!-- compmd-pack:generated from react-qr-code 2.2.0's types and compmd.wrap.json; edits between these comments are overwritten at each build -->
```compmd component=qr-code signature
params:
  value: string
  bgColor: color?
  fgColor: color?
  level: L | M | H | Q?
  title: string?
```
<!-- /compmd-pack:generated -->
