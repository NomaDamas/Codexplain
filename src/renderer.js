import { measureWidth, padEndWidth, wrapText } from "./text-width.js";
import { colorize, normalizeTheme, stripAnsi } from "./theme.js";

function normalizeWidth(width) {
  return Math.max(20, Math.floor(Number(width) || 80));
}

function frameChars(frame = "unicode") {
  return frame === "ascii"
    ? {
        horizontal: "-",
        vertical: "|",
        topLeft: "+",
        topJoin: "+",
        topRight: "+",
        middleLeft: "+",
        middleJoin: "+",
        middleRight: "+",
        bottomLeft: "+",
        bottomJoin: "+",
        bottomRight: "+",
        downJoin: "+",
        upJoin: "+",
        arrow: "v",
      }
    : {
        horizontal: "─",
        vertical: "│",
        topLeft: "┌",
        topJoin: "┬",
        topRight: "┐",
        middleLeft: "├",
        middleJoin: "┼",
        middleRight: "┤",
        bottomLeft: "└",
        bottomJoin: "┴",
        bottomRight: "┘",
        downJoin: "┬",
        upJoin: "┴",
        arrow: "▼",
      };
}

function border(left, join, right, widths, theme = "none", frame = "unicode") {
  const chars = frameChars(frame);
  return colorize(
    `${left}${widths.map((width) => chars.horizontal.repeat(width + 2)).join(join)}${right}`,
    "border",
    theme,
  );
}

function row(cells, widths, theme = "none", role = "accent", frame = "unicode") {
  const selectedTheme = normalizeTheme(theme);
  const chars = frameChars(frame);
  const border = colorize(chars.vertical, "border", selectedTheme);
  return `${border}${cells
    .map((cell, index) => ` ${colorize(padEndWidth(cell, widths[index]), role, selectedTheme)} `)
    .join(border)}${border}`;
}

function fitColumnWidths({ headers, rows, terminalWidth }) {
  const columnCount = headers.length;
  const desired = headers.map((header, index) =>
    Math.max(measureWidth(header), ...rows.map((item) => measureWidth(item[index] ?? "")), 2),
  );
  const totalWidth = desired.reduce((sum, item) => sum + item + 3, 1);
  if (totalWidth <= terminalWidth) return desired;

  const contentBudget = terminalWidth - (columnCount * 3 + 1);
  const minimum = headers.map((header) => Math.max(4, measureWidth(header)));
  const minimumTotal = minimum.reduce((sum, item) => sum + item, 0);
  if (contentBudget < minimumTotal) return desired;

  const widths = [...minimum];
  let remaining = contentBudget - minimumTotal;
  const extraNeeds = desired.map((item, index) => Math.max(0, item - minimum[index]));
  let extraTotal = extraNeeds.reduce((sum, item) => sum + item, 0);

  while (remaining > 0 && extraTotal > 0) {
    let changed = false;
    for (let index = 0; index < columnCount && remaining > 0; index += 1) {
      if (widths[index] < desired[index]) {
        widths[index] += 1;
        remaining -= 1;
        changed = true;
      }
    }
    if (!changed) break;
    extraTotal = desired.reduce((sum, item, index) => sum + Math.max(0, item - widths[index]), 0);
  }

  return widths;
}

function wrappedRows(cells, widths) {
  const wrapped = cells.map((cell, index) => wrapText(cell, widths[index]));
  const lineCount = Math.max(...wrapped.map((lines) => lines.length));
  return Array.from({ length: lineCount }, (_, rowIndex) =>
    wrapped.map((lines) => lines[rowIndex] ?? ""),
  );
}

export function renderBoxTable({ headers, rows, width = 80, theme = "none", frame = "unicode" }) {
  const terminalWidth = normalizeWidth(width);
  const selectedTheme = normalizeTheme(theme);
  const chars = frameChars(frame);
  const columnCount = headers.length;
  const widths = fitColumnWidths({ headers, rows, terminalWidth });

  const lines = [
    border(chars.topLeft, chars.topJoin, chars.topRight, widths, selectedTheme, frame),
    ...wrappedRows(headers, widths).map((item) => row(item, widths, selectedTheme, "heading", frame)),
    border(chars.middleLeft, chars.middleJoin, chars.middleRight, widths, selectedTheme, frame),
  ];
  for (const item of rows) {
    if (item.length !== columnCount) throw new RangeError("table row length must match headers");
    lines.push(...wrappedRows(item, widths).map((line) => row(line, widths, selectedTheme, "accent", frame)));
  }
  lines.push(border(chars.bottomLeft, chars.bottomJoin, chars.bottomRight, widths, selectedTheme, frame));
  return lines.join("\n");
}

function flowBorder(left, center, right, contentWidth, frame = "unicode") {
  const chars = frameChars(frame);
  const dashCount = contentWidth + 2;
  const centerIndex = Math.floor(dashCount / 2);
  return `${left}${chars.horizontal.repeat(centerIndex)}${center}${chars.horizontal.repeat(dashCount - centerIndex - 1)}${right}`;
}

function flowBox(text, contentWidth, position, theme = "none", frame = "unicode") {
  const chars = frameChars(frame);
  const topCenter = position === "first" ? chars.horizontal : chars.upJoin;
  const bottomCenter = position === "last" ? chars.horizontal : chars.downJoin;
  return [
    colorize(flowBorder(chars.topLeft, topCenter, chars.topRight, contentWidth, frame), "border", theme),
    ...wrapText(text, contentWidth).map((line) =>
      `${colorize(chars.vertical, "border", theme)} ${colorize(padEndWidth(line, contentWidth), "accent", theme)} ${colorize(chars.vertical, "border", theme)}`,
    ),
    colorize(flowBorder(chars.bottomLeft, bottomCenter, chars.bottomRight, contentWidth, frame), "border", theme),
  ];
}

export function renderFlow({ steps, width = 80, theme = "none", frame = "unicode" }) {
  if (!steps.length) return "";
  const terminalWidth = normalizeWidth(width);
  const selectedTheme = normalizeTheme(theme);
  const chars = frameChars(frame);
  const contentWidth = Math.min(
    Math.max(...steps.map((step) => measureWidth(step)), 4),
    Math.max(4, terminalWidth - 4),
  );
  const spine = " ".repeat(Math.floor((contentWidth + 2) / 2) + 1);
  const lines = [];

  steps.forEach((step, index) => {
    if (index > 0) {
      lines.push(
        `${spine}${colorize(chars.vertical, "border", selectedTheme)}`,
        `${spine}${colorize(chars.arrow, "heading", selectedTheme)}`,
      );
    }
    lines.push(
      ...flowBox(
        step,
        contentWidth,
        index === 0 ? "first" : index === steps.length - 1 ? "last" : "middle",
        selectedTheme,
        frame,
      ),
    );
  });

  return lines.join("\n");
}

function lineWidth(line) {
  return measureWidth(stripAnsi(line));
}

function panelWidth(panel) {
  return Math.max(...String(panel ?? "").split("\n").map(lineWidth), 0);
}

function padVisible(line, width) {
  return String(line ?? "") + " ".repeat(Math.max(0, width - lineWidth(line)));
}

export function renderResponsivePanels({ panels, width = 100, gap = 3 }) {
  const items = panels.map((panel) => String(panel ?? "").trimEnd()).filter(Boolean);
  if (!items.length) return "";
  if (items.length === 1) return items[0];

  const terminalWidth = normalizeWidth(width);
  const widths = items.map(panelWidth);
  const requiredWidth = widths.reduce((sum, item) => sum + item, 0) + gap * (items.length - 1);
  if (requiredWidth > terminalWidth) return items.join("\n\n");

  const linesByPanel = items.map((panel) => panel.split("\n"));
  const maxLines = Math.max(...linesByPanel.map((lines) => lines.length));
  const spacer = " ".repeat(gap);
  return Array.from({ length: maxLines }, (_, lineIndex) =>
    linesByPanel
      .map((lines, panelIndex) => padVisible(lines[lineIndex] ?? "", widths[panelIndex]))
      .join(spacer),
  ).join("\n");
}

export function renderDemo() {
  return [
    renderFlow({ steps: ["요청 이해", "핵심 정리", "검증 보존"], width: 40 }),
    "",
    renderBoxTable({
      headers: ["영역", "답변 기준"],
      rows: [
        ["내용", "기술 사실 보존"],
        ["표현", "짧고 스캔 가능"],
      ],
      width: 60,
    }),
  ].join("\n");
}
