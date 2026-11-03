# 8. Components and trust

A document can bring code: a component it defines in a fence, or one from a pack it imports. This chapter is what that code may do in your app, at each of the three trust levels, who decides a level, and what the sandbox guards against. The chapters after it put it to work: packs from npm, your app's own components, and the sandbox in depth.

## Three levels

Every component written in code (JavaScript, TypeScript, JSX or Svelte) runs at one of three levels. Components written in CompMD run no code, so they need none.

| Level | Where it runs | What it can reach | For |
| --- | --- | --- | --- |
| **strict** (the default) | a frame with an opaque origin and no network | its params, its slots, its box, and `set` for the inputs bound to it | code from documents, and every pack your configuration doesn't list for more |
| **permissive** | a frame of its own, one per use | the same, plus the network to the origins you list for its pack, and `host` if you give it | packs you list, such as a map that loads its tiles |
| **trusted** | your page | what your page's own code can, and `host` if you give one | your app's own components, in a folder of your app, built with it |

A level belongs to a pack, never to a component or a use: every component of a pack runs at the pack's level.

## Strict: the default

Strict code gets the host contract ([spec §7.4](https://compmd.hakanserce.com/spec/#74-the-host-contract)) and nothing else. In plain words, a strict component:

1. **sees only what it is given**: its params, its slots and its box, not the rest of the document, its data or your page;
2. **has no network**: no requests, images, fonts, stylesheets or sockets, to anywhere; its code comes from its pack, and its libraries from the frames;
3. **can't reach anyone's session**: no cookies, storage or credentials, yours or the reader's;
4. **can't navigate or open windows**: a link it shows opens only when the reader clicks it, and your page hears of the click first (`view.on("link", …)`);
5. **can be stopped**: a frame that stops answering is removed, and its use shows its fallback (W118).

That is why any document can import any pack: a pack nobody reviewed still can't send what it is given anywhere, because it has nowhere to send it. Private data you pass a strict component as a param stays in its frame.

## Permissive: what you list, and nothing more

A permissive pack keeps rules 3 to 5, and gets only what your configuration lists for it:

- **the network to its origins**, each `https://host[:port]`, added to its frame's policy before any of its code loads;
- **`host`**, the calls your app offers it, when you give it `"host": true` ([chapter 10](10-your-apps-own-components.md)).

Each permissive use gets a frame of its own, so the network one pack is given reaches no other pack's code. A permissive frame can hold strict frames, for strict components in its slots, and nothing else.

## Trusted: your own code

A trusted component is your app's: built, reviewed and shipped with it, so a sandbox would only protect your app from itself. It mounts in your page, as any of your code does, and gets `host` as a permissive one does, when your app gives one: the same calls and events, so a component works at either level ([chapter 10](10-your-apps-own-components.md)). Only a folder of your app can be trusted, never a pack from a registry: `@compmd/vite` stops the build if the configuration says otherwise.

Its params still come from documents, which may be anyone's. A trusted component treats them as untrusted input: no HTML from a param, URLs checked, and nothing done outside the document unless the reader asks for it ([chapter 10](10-your-apps-own-components.md#params-are-untrusted-input)).

## Who decides

**Your configuration, and only it.** `compmd.config.json` lists the packs that get more than strict:

```json
{
  "packs": {
    "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com"] },
    "@acme/reviews": { "trust": "permissive", "host": true },
    "./components": { "trust": "trusted", "ambient": true }
  }
}
```

The plugin turns it into the processor's policy, the processor marks each component the document uses with its pack's level (`trust` in `components`), and the sandbox runs each where its level says. The sandbox never reads the configuration itself, so a level is decided in one place.

**Never a document.** Nothing in a document raises a level: not its frontmatter, not an import, not a param. A component a document defines itself is always strict, in any app. This one runs in the strict sandbox wherever it is opened:

````md
```compmd component=shout signature
params:
  text: string
```

```jsx component=shout runtime=react@19
export default function Shout({ text }) {
  return <strong>{text.toUpperCase()}</strong>;
}
```

:: shout[Hello] {text="Hello"}
````

**Never a pack.** A pack can't name its own level, its own origins or a registry. Listing a pack gives it more wherever your app gets it, installed or fetched.

## Where the page renders

The sandbox renders a document in one of two modes, and your app chooses, with `createSandbox`'s `page` option:

| | `page: "frame"` (the default) | `page: "host"` |
| --- | --- | --- |
| The page | one strict frame per document | your page |
| Strict components | in the page's frame, sharing it | a frame per use |
| Permissive components | their fallbacks | a frame per use |
| Trusted components | their fallbacks | inline, in your page |
| For | documents from anyone, with strict packs | an app with components of its own, or permissive packs |

A permissive or trusted component that can't run where it is shows its fallback, and the view's `error` event says why (`not-here`). [Chapter 11](11-the-sandbox-closer.md) has both modes in depth.

## The threat model, in plain words

Who might misbehave, and what they must not reach:

| Code | Comes from | Must not reach |
| --- | --- | --- |
| Your page, the sandbox's host side, trusted components | your app | (trusted) |
| Permissive code | a pack you list | origins other than its pack's; your page and session; other frames; `host`, unless you give it |
| Strict code | documents, unlisted packs | any network; your page and session; other documents' frames; code of a higher level |
| The document | anyone | a higher level for any of its code |

The sandbox stops each threat with something the browser enforces: the frames' opaque origin, their content security policies, which a frame can only narrow, frames that only nest lower levels, a hash check for every built module, and a ping for every frame that runs components. Each one is a browser test in Chrome and WebKit. The design has the full list, T1 to T10 ([running components in the sandbox](../../docs/running-components-in-the-sandbox.md#threat-model)).

What a browser can't stop, and so the sandbox can't either:

- **WebRTC**, which no content security policy covers.
- **Requests to the files the frames are served from.** A frame loads its scripts and styles from where you serve `compmd/`, so a request there can carry data in its query string to your own server. Strict code can reach nobody else this way. If you mind, serve `compmd/` from an origin of its own, with no logs.
- **A frame that spins in WebKit.** Chrome stops it and removes it; WebKit runs these frames in the page's process, so a frame that never lets go freezes the page with it.
- **What a component draws in its own box**, such as a fake password prompt. What is typed into it has no way out, and the frames are placed as the document's content, never over your interface.

Code that acts with someone's access, a trusted component or one given `host`, has one more rule ([spec §15](https://compmd.hakanserce.com/spec/#15-security)): the document that uses it may be someone else's, so it acts outside the document (sends, writes, shares, pays) only when the reader asks, after showing what it will do.

Next: [packs from npm](09-packs-from-npm.md), what your app installs or fetches, and what your configuration gives them.
