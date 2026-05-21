const RESET = "\u001b[0m";

const THEMES = {
  none: {},
  ocean: {
    border: "\u001b[36m",
    heading: "\u001b[1;34m",
    accent: "\u001b[96m",
  },
  forest: {
    border: "\u001b[32m",
    heading: "\u001b[1;32m",
    accent: "\u001b[92m",
  },
  warm: {
    border: "\u001b[33m",
    heading: "\u001b[1;31m",
    accent: "\u001b[93m",
  },
  mono: {
    border: "\u001b[90m",
    heading: "\u001b[1m",
    accent: "\u001b[37m",
  },
};

export function normalizeTheme(value, fallback = "none") {
  const theme = String(value ?? "").trim().toLowerCase();
  if (theme === "off" || theme === "false" || theme === "0") return "none";
  return Object.hasOwn(THEMES, theme) ? theme : fallback;
}

export function themeNames() {
  return Object.keys(THEMES);
}

export function colorize(value, role = "accent", theme = "none") {
  const selected = THEMES[normalizeTheme(theme)];
  const code = selected?.[role];
  return code ? `${code}${value}${RESET}` : String(value ?? "");
}

export function stripAnsi(value) {
  return String(value ?? "").replace(/\u001b\[[0-9;]*m/g, "");
}
