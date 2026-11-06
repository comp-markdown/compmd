// The site's card, the image a link to it shows on social sites and in chats (`og:image`): the logo and the hero's
// line on the site's sheet, 1200 by 630, made from this page by the build (`scripts/card.ts`), so it says what the
// site says.

/** The card's size, Open Graph's and Twitter's large card. */
export const CARD = { width: 1200, height: 630 } as const;

/** The card's file at the site's root. */
export const CARD_FILE = "og.png";

/** The card as a page: the logo `logo` (its SVG), the hero's line, the lede and the site's address. */
export function cardHtml(logo: string): string {
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<style>
  * { box-sizing: border-box; }
  html, body { margin: 0; width: ${String(CARD.width)}px; height: ${String(CARD.height)}px; }
  body {
    display: grid;
    place-items: center;
    background: #e9edf1;
    color: #1b2430;
    font-family: "Avenir Next", "Segoe UI", system-ui, sans-serif;
  }
  .sheet {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    width: 1080px;
    height: 510px;
    padding: 64px 72px;
    border-radius: 14px;
    background: #fbfcfd;
    box-shadow: 0 2px 4px rgb(27 36 48 / 6%), 0 18px 48px rgb(27 36 48 / 10%);
  }
  .sheet::after {
    content: "";
    position: absolute;
    top: -14px;
    right: -14px;
    width: 44px;
    height: 44px;
    border-radius: 11px;
    background: #3557c4;
  }
  .logo svg { display: block; height: 56px; width: auto; }
  h1 {
    margin: 0;
    font: 600 84px/1.02 "Iowan Old Style", Charter, Georgia, serif;
    letter-spacing: -0.015em;
  }
  p { margin: 18px 0 0; color: #5b6776; font-size: 34px; }
  .address { color: #3557c4; font-size: 26px; font-weight: 600; }
</style>
</head>
<body>
  <div class="sheet">
    <div class="logo">${logo}</div>
    <div>
      <h1>Markdown with components.</h1>
      <p>Readable as text, rendered as a page.</p>
    </div>
    <div class="address">compmd.hakanserce.com</div>
  </div>
</body>
</html>`;
}
