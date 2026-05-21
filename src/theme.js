const RESET = "\u001b[0m";

const THEMES = {
  none: {},
  ocean: {
    border: "\u001b[36m",
    heading: "\u001b[1;34m",
    accent: "\u001b[96m",
    muted: "\u001b[2;36m",
    success: "\u001b[1;32m",
    warning: "\u001b[1;33m",
    danger: "\u001b[1;31m",
  },
  forest: {
    border: "\u001b[32m",
    heading: "\u001b[1;32m",
    accent: "\u001b[92m",
    muted: "\u001b[2;32m",
    success: "\u001b[1;32m",
    warning: "\u001b[1;33m",
    danger: "\u001b[1;31m",
  },
  warm: {
    border: "\u001b[33m",
    heading: "\u001b[1;31m",
    accent: "\u001b[93m",
    muted: "\u001b[2;33m",
    success: "\u001b[1;32m",
    warning: "\u001b[1;33m",
    danger: "\u001b[1;31m",
  },
  sunset: {
    border: "\u001b[38;5;208m",
    heading: "\u001b[1;38;5;196m",
    accent: "\u001b[38;5;214m",
    muted: "\u001b[2;38;5;208m",
    success: "\u001b[1;38;5;118m",
    warning: "\u001b[1;38;5;220m",
    danger: "\u001b[1;38;5;196m",
  },
  grape: {
    border: "\u001b[38;5;141m",
    heading: "\u001b[1;38;5;135m",
    accent: "\u001b[38;5;183m",
    muted: "\u001b[2;38;5;141m",
    success: "\u001b[1;38;5;120m",
    warning: "\u001b[1;38;5;222m",
    danger: "\u001b[1;38;5;204m",
  },
  slate: {
    border: "\u001b[38;5;67m",
    heading: "\u001b[1;38;5;110m",
    accent: "\u001b[38;5;153m",
    muted: "\u001b[2;38;5;67m",
    success: "\u001b[1;38;5;114m",
    warning: "\u001b[1;38;5;179m",
    danger: "\u001b[1;38;5;167m",
  },
  rose: {
    border: "\u001b[38;5;211m",
    heading: "\u001b[1;38;5;199m",
    accent: "\u001b[38;5;218m",
    muted: "\u001b[2;38;5;211m",
    success: "\u001b[1;38;5;120m",
    warning: "\u001b[1;38;5;222m",
    danger: "\u001b[1;38;5;197m",
  },
  mono: {
    border: "\u001b[90m",
    heading: "\u001b[1m",
    accent: "\u001b[37m",
    muted: "\u001b[2m",
    success: "\u001b[1m",
    warning: "\u001b[1m",
    danger: "\u001b[1m",
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
