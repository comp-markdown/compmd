// Time zone offsets for the WebAssembly module, from the page's `Intl`, so the module carries no zone data.

/**
 * The offset of an IANA zone from UTC at a time, in seconds east of UTC; `undefined` for a zone `Intl` doesn't know.
 * @param {string} zone
 * @param {number} seconds since 1970-01-01T00:00:00Z
 * @returns {number | undefined}
 */
export function zoneOffset(zone, seconds) {
  let format;
  try {
    format = new Intl.DateTimeFormat("en-US", { timeZone: zone, timeZoneName: "longOffset" });
  } catch {
    return undefined;
  }
  const name = format.formatToParts(new Date(seconds * 1000)).find((part) => part.type === "timeZoneName")?.value;
  // `GMT` for UTC itself, else `GMT+05:30` or `GMT-03:00`.
  const match = /^GMT(?:([+-])(\d{2}):(\d{2}))?$/.exec(name ?? "");
  if (!match) return undefined;
  const [, sign, hours, minutes] = match;
  return sign ? (sign === "-" ? -1 : 1) * (Number(hours) * 3600 + Number(minutes) * 60) : 0;
}
