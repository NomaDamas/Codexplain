const TAB_WIDTH = 4;

function isCombining(codePoint) {
  return (
    (codePoint >= 0x0300 && codePoint <= 0x036f) ||
    (codePoint >= 0x1ab0 && codePoint <= 0x1aff) ||
    (codePoint >= 0x1dc0 && codePoint <= 0x1dff) ||
    (codePoint >= 0x20d0 && codePoint <= 0x20ff) ||
    (codePoint >= 0xfe20 && codePoint <= 0xfe2f)
  );
}

function isWide(codePoint) {
  return (
    codePoint >= 0x1100 &&
    (codePoint <= 0x115f ||
      codePoint === 0x2329 ||
      codePoint === 0x232a ||
      (codePoint >= 0x2e80 && codePoint <= 0xa4cf && codePoint !== 0x303f) ||
      (codePoint >= 0xac00 && codePoint <= 0xd7a3) ||
      (codePoint >= 0xf900 && codePoint <= 0xfaff) ||
      (codePoint >= 0xfe10 && codePoint <= 0xfe19) ||
      (codePoint >= 0xfe30 && codePoint <= 0xfe6f) ||
      (codePoint >= 0xff00 && codePoint <= 0xff60) ||
      (codePoint >= 0x1f300 && codePoint <= 0x1faff))
  );
}

function charWidth(char) {
  const codePoint = char.codePointAt(0);
  if (codePoint === undefined) return 0;
  if (codePoint === 0x09) return TAB_WIDTH;
  if (codePoint === 0 || codePoint < 0x20 || isCombining(codePoint)) return 0;
  return isWide(codePoint) ? 2 : 1;
}

export function measureWidth(value) {
  return [...String(value ?? "")].reduce((sum, char) => sum + charWidth(char), 0);
}

export function padEndWidth(value, width) {
  const text = String(value ?? "");
  return text + " ".repeat(Math.max(0, width - measureWidth(text)));
}

export function wrapText(value, width) {
  const target = Math.max(1, Math.floor(width));
  const output = [];

  for (const rawLine of String(value ?? "").replace(/\r\n?/g, "\n").split("\n")) {
    let line = "";
    let lineWidth = 0;
    for (const char of rawLine) {
      const width = charWidth(char);
      if (line && lineWidth + width > target) {
        output.push(line);
        line = "";
        lineWidth = 0;
      }
      line += char;
      lineWidth += width;
    }
    output.push(line);
  }

  return output.length ? output : [""];
}
