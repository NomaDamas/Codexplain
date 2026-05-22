use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Frame {
    Unicode,
    Ascii,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FramePreset {
    Table,
    Flow,
    Indexed,
    Formula,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme {
    None,
    Ocean,
    Forest,
    Warm,
    Sunset,
    Grape,
    Slate,
    Rose,
    Mono,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnsiRole {
    Border,
    Heading,
    Accent,
    Muted,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AnsiStyle {
    prefix: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThemeSpec {
    border: AnsiStyle,
    heading: AnsiStyle,
    accent: AnsiStyle,
    muted: AnsiStyle,
    success: AnsiStyle,
    warning: AnsiStyle,
    danger: AnsiStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameBorder {
    horizontal: char,
    vertical: char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameCorners {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameSeparators {
    top_join: char,
    middle_left: char,
    middle_join: char,
    middle_right: char,
    bottom_join: char,
    up_join: char,
    down_join: char,
    arrow_down: char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FramePadding {
    left: usize,
    right: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FrameSpec {
    border: FrameBorder,
    corners: FrameCorners,
    separators: FrameSeparators,
    padding: FramePadding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameRule {
    Top,
    Middle,
    RowDivider,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FrameSegment {
    Glyph(char),
    Repeat(char, usize),
    Text(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FrameLine {
    segments: Vec<FrameSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TableCell {
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TableRow {
    cells: Vec<TableCell>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Table {
    headers: TableRow,
    rows: Vec<TableRow>,
    row_dividers: bool,
    max_width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TableLayout {
    spec: FrameSpec,
    widths: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FlowStep {
    label: String,
    branches: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FlowDiagram {
    steps: Vec<FlowStep>,
    max_width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FlowLayout {
    spec: FrameSpec,
    content_width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProsConsOption {
    choice: &'static str,
    pros: &'static [&'static str],
    cons: &'static [&'static str],
    best_for: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FormulaField {
    label: String,
    value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FormulaBox {
    title: String,
    fields: Vec<FormulaField>,
    max_width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FormulaBoxLayout {
    spec: FrameSpec,
    content_width: usize,
    label_width: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndexStyle {
    Decimal,
    ZeroPadded,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RendererKind {
    Table,
    ProsCons,
    Formula,
    IndexedList,
    Flow,
    Progress,
    TldrProse,
    Prose,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExplanationIntent {
    Comparison,
    DecisionRule,
    OrderedSteps,
    ProcessFlow,
    ProgressReport,
    StructuredSummary,
    StatusSummary,
    GeneralAnswer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptSignalKind {
    Keyword,
    ProfilePreference,
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PromptSignal {
    renderer: RendererKind,
    intent: ExplanationIntent,
    kind: PromptSignalKind,
    pattern: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RendererSelection {
    renderer: RendererKind,
    intent: ExplanationIntent,
    signal: PromptSignal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UxComponent {
    StatusBadge,
    Checklist,
    RiskPanel,
    ConfidenceMeter,
    DiffSummary,
    DecisionMatrix,
    NextAction,
    EtaStrip,
    AttentionCallout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndexedList {
    items: Vec<String>,
    style: IndexStyle,
    max_width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndexedListLayout {
    spec: FrameSpec,
    marker_width: usize,
    content_width: usize,
}

#[derive(Clone)]
struct Profile {
    theme: Theme,
    frame: Frame,
    index_style: IndexStyle,
    detail: String,
    style: String,
    audience: String,
    preferred_structure: String,
    abstraction_min: String,
    abstraction_max: String,
    layers: Vec<String>,
    explanation_depth: String,
    architecture_depth: String,
    abstraction_level: String,
    detail_scale: u8,
    ux_density: u8,
    risk_sensitivity: u8,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            theme: Theme::Ocean,
            frame: Frame::Unicode,
            index_style: IndexStyle::Decimal,
            detail: "deep".to_string(),
            style: "plain".to_string(),
            audience: "general".to_string(),
            preferred_structure: "auto".to_string(),
            abstraction_min: "concrete".to_string(),
            abstraction_max: "architecture".to_string(),
            layers: vec![
                "tldr".to_string(),
                "summary".to_string(),
                "architecture".to_string(),
                "implementation".to_string(),
                "evidence".to_string(),
                "next-step".to_string(),
            ],
            explanation_depth: "deep".to_string(),
            architecture_depth: "system".to_string(),
            abstraction_level: "architecture".to_string(),
            detail_scale: 80,
            ux_density: 65,
            risk_sensitivity: 60,
        }
    }
}

impl Frame {
    fn parse(value: Option<&str>) -> Self {
        Self::select(value, |key| env::var(key).ok())
    }

    fn select<F>(value: Option<&str>, env_value: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        match value
            .unwrap_or("unicode")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "ascii" | "plain-ascii" | "fallback" | "non-unicode" | "no-unicode" => Self::Ascii,
            "auto" | "terminal" => {
                if terminal_supports_unicode(env_value) {
                    Self::Unicode
                } else {
                    Self::Ascii
                }
            }
            "box" | "unicode" | "utf8" | "utf-8" => Self::Unicode,
            _ => Self::Unicode,
        }
    }

    fn preset(self, preset: FramePreset) -> FrameSpec {
        match (self, preset) {
            (Self::Ascii, FramePreset::Table) => FrameSpec::ascii_table(),
            (Self::Ascii, FramePreset::Flow) => FrameSpec::ascii_flow(),
            (Self::Ascii, FramePreset::Indexed) => FrameSpec::ascii_indexed(),
            (Self::Ascii, FramePreset::Formula) => FrameSpec::ascii_formula(),
            (Self::Unicode, FramePreset::Table) => FrameSpec::unicode_table(),
            (Self::Unicode, FramePreset::Flow) => FrameSpec::unicode_flow(),
            (Self::Unicode, FramePreset::Indexed) => FrameSpec::unicode_indexed(),
            (Self::Unicode, FramePreset::Formula) => FrameSpec::unicode_formula(),
        }
    }
}

fn env_flag_enabled(value: Option<String>) -> bool {
    matches!(
        value
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn terminal_supports_unicode<F>(env_value: F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    if env_flag_enabled(env_value("CODEXPLAIN_NO_UNICODE"))
        || env_flag_enabled(env_value("NO_UNICODE"))
    {
        return false;
    }
    if matches!(env_value("TERM").as_deref(), Some("dumb")) {
        return false;
    }
    let locale = env_value("LC_ALL")
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env_value("LC_CTYPE").filter(|value| !value.trim().is_empty()))
        .or_else(|| env_value("LANG").filter(|value| !value.trim().is_empty()));
    match locale {
        Some(value) => {
            let normalized = value.trim().to_ascii_uppercase();
            normalized.contains("UTF-8") || normalized.contains("UTF8")
        }
        None => true,
    }
}

fn env_nonempty(value: Option<String>) -> bool {
    value.map(|item| !item.trim().is_empty()).unwrap_or(false)
}

fn terminal_supports_ansi<F>(env_value: F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    if env_nonempty(env_value("NO_COLOR"))
        || env_flag_enabled(env_value("CODEXPLAIN_NO_COLOR"))
        || env_flag_enabled(env_value("CLAUDEX_NO_COLOR"))
    {
        return false;
    }
    if matches!(env_value("TERM").as_deref(), Some("dumb")) {
        return false;
    }
    if matches!(env_value("CLICOLOR").as_deref(), Some("0")) {
        return false;
    }
    if let Some(value) = env_value("CODEXPLAIN_COLOR").or_else(|| env_value("CLAUDEX_COLOR")) {
        if matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "none" | "never" | "no-color" | "plain"
        ) {
            return false;
        }
    }
    true
}

impl FramePadding {
    fn total(self) -> usize {
        self.left + self.right
    }

    fn apply(self, text: &str, width: usize) -> String {
        format!(
            "{}{}{}",
            " ".repeat(self.left),
            pad(text, width),
            " ".repeat(self.right)
        )
    }
}

impl FrameLine {
    fn new() -> Self {
        Self::default()
    }

    fn glyph(mut self, value: char) -> Self {
        self.segments.push(FrameSegment::Glyph(value));
        self
    }

    fn repeat(mut self, value: char, count: usize) -> Self {
        self.segments.push(FrameSegment::Repeat(value, count));
        self
    }

    fn text(mut self, value: impl Into<String>) -> Self {
        self.segments.push(FrameSegment::Text(value.into()));
        self
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for segment in &self.segments {
            match segment {
                FrameSegment::Glyph(value) => out.push(*value),
                FrameSegment::Repeat(value, count) => {
                    out.push_str(&value.to_string().repeat(*count));
                }
                FrameSegment::Text(value) => out.push_str(value),
            }
        }
        out
    }
}

impl TableCell {
    fn new(value: impl Into<String>) -> Self {
        Self { text: value.into() }
    }

    fn width(&self) -> usize {
        visible_width(&self.text)
    }

    fn wrapped(&self, width: usize) -> Vec<String> {
        wrap_text(&self.text, width)
    }
}

impl TableRow {
    fn new(values: impl IntoIterator<Item = String>) -> Self {
        Self {
            cells: values.into_iter().map(TableCell::new).collect(),
        }
    }

    fn from_strs(values: &[&str]) -> Self {
        Self::new(values.iter().map(|value| (*value).to_string()))
    }
}

impl Table {
    fn new(headers: &[&str], rows: &[Vec<String>], row_dividers: bool, max_width: usize) -> Self {
        Self {
            headers: TableRow::from_strs(headers),
            rows: rows.iter().cloned().map(TableRow::new).collect(),
            row_dividers,
            max_width,
        }
    }

    fn column_count(&self) -> usize {
        self.headers.cells.len()
    }

    fn desired_widths(&self) -> Vec<usize> {
        (0..self.column_count())
            .map(|index| {
                self.rows
                    .iter()
                    .filter_map(|row| row.cells.get(index))
                    .map(TableCell::width)
                    .chain(self.headers.cells.get(index).map(TableCell::width))
                    .max()
                    .unwrap_or(4)
                    .max(4)
            })
            .collect()
    }

    fn layout(&self, frame: Frame) -> TableLayout {
        TableLayout::fit(self, frame.preset(FramePreset::Table))
    }
}

impl TableLayout {
    fn fit(table: &Table, spec: FrameSpec) -> Self {
        let desired = table.desired_widths();
        let column_count = desired.len();
        let frame_width = column_count * spec.padding.total() + column_count + 1;
        let min_column_width = if table.max_width.saturating_sub(frame_width) >= column_count * 8 {
            8
        } else {
            4
        };
        let budget = table
            .max_width
            .saturating_sub(frame_width)
            .max(column_count * min_column_width);
        let desired_sum: usize = desired.iter().sum();
        let widths = if desired_sum <= budget {
            desired
        } else {
            fit_column_widths(&desired, budget, min_column_width)
        };

        Self { spec, widths }
    }

    fn border(&self, rule: FrameRule, theme: Theme) -> String {
        color(theme, "border", &self.spec.rule(rule, &self.widths))
    }

    fn row_divider(&self, theme: Theme) -> String {
        self.border(FrameRule::RowDivider, theme)
    }

    fn padded_cell(&self, text: &str, width: usize) -> String {
        self.spec.padded_cell(text, width)
    }
}

impl FlowStep {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            branches: Vec::new(),
        }
    }

    fn with_branches(label: impl Into<String>, branches: impl IntoIterator<Item = String>) -> Self {
        Self {
            label: label.into(),
            branches: branches.into_iter().collect(),
        }
    }
}

impl FlowDiagram {
    fn new(steps: impl IntoIterator<Item = FlowStep>, max_width: usize) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            max_width,
        }
    }

    fn layout(&self, frame: Frame) -> FlowLayout {
        FlowLayout::fit(self, frame.preset(FramePreset::Flow))
    }
}

impl FlowLayout {
    fn fit(diagram: &FlowDiagram, spec: FrameSpec) -> Self {
        let widest_step = diagram
            .steps
            .iter()
            .flat_map(|step| std::iter::once(&step.label).chain(step.branches.iter()))
            .map(|text| visible_width(text))
            .max()
            .unwrap_or(4)
            .max(4);
        let frame_overhead = spec.padding.total() + 2;
        let max_content_width = diagram.max_width.saturating_sub(frame_overhead).max(4);
        let content_width = widest_step.min(max_content_width).min(48);

        Self {
            spec,
            content_width,
        }
    }

    fn spine_indent(&self) -> String {
        " ".repeat((self.content_width + self.spec.padding.total()) / 2 + 1)
    }

    fn box_line(&self, text: &str, theme: Theme, role: &str) -> String {
        FrameLine::new()
            .text(color(
                theme,
                "border",
                &self.spec.border.vertical.to_string(),
            ))
            .text(color(
                theme,
                role,
                &self.spec.padding.apply(text, self.content_width),
            ))
            .text(color(
                theme,
                "border",
                &self.spec.border.vertical.to_string(),
            ))
            .render()
    }
}

impl FormulaField {
    fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

impl FormulaBox {
    fn new(
        title: impl Into<String>,
        fields: impl IntoIterator<Item = FormulaField>,
        max_width: usize,
    ) -> Self {
        Self {
            title: title.into(),
            fields: fields.into_iter().collect(),
            max_width,
        }
    }

    fn layout(&self, frame: Frame) -> FormulaBoxLayout {
        FormulaBoxLayout::fit(self, frame.preset(FramePreset::Formula))
    }
}

impl FormulaBoxLayout {
    fn fit(box_model: &FormulaBox, spec: FrameSpec) -> Self {
        let frame_overhead = spec.padding.total() + 2;
        let max_content_width = box_model.max_width.saturating_sub(frame_overhead).max(1);
        let field_width = box_model
            .fields
            .iter()
            .map(|field| visible_width(&field.label) + 3 + visible_width(&field.value))
            .max()
            .unwrap_or(0);
        let desired_content_width = visible_width(&box_model.title).max(field_width).max(20);
        let content_width = desired_content_width.min(max_content_width);
        let max_label_width = content_width.saturating_sub(5).max(1);
        let label_width = box_model
            .fields
            .iter()
            .map(|field| visible_width(&field.label))
            .max()
            .unwrap_or(4)
            .max(4)
            .min(max_label_width);

        Self {
            spec,
            content_width,
            label_width,
        }
    }

    fn border(&self, rule: FrameRule, theme: Theme) -> String {
        color(
            theme,
            "border",
            &self.spec.rule(rule, &[self.content_width]),
        )
    }

    fn line(&self, text: &str, theme: Theme, role: &str) -> String {
        FrameLine::new()
            .text(color(
                theme,
                "border",
                &self.spec.border.vertical.to_string(),
            ))
            .text(color(
                theme,
                role,
                &self.spec.padded_cell(text, self.content_width),
            ))
            .text(color(
                theme,
                "border",
                &self.spec.border.vertical.to_string(),
            ))
            .render()
    }
}

impl IndexStyle {
    fn parse(value: Option<&str>) -> Self {
        let raw = value.unwrap_or("decimal").trim();
        if raw == "A" {
            return Self::AlphaUpper;
        }
        if raw == "I" {
            return Self::RomanUpper;
        }
        match raw.to_ascii_lowercase().as_str() {
            "zero" | "zero-padded" | "padded" | "pad" | "01" => Self::ZeroPadded,
            "alpha" | "alpha-lower" | "lower-alpha" | "a" => Self::AlphaLower,
            "alpha-upper" | "upper-alpha" => Self::AlphaUpper,
            "roman" | "roman-lower" | "lower-roman" | "i" => Self::RomanLower,
            "roman-upper" | "upper-roman" => Self::RomanUpper,
            _ => Self::Decimal,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Decimal => "decimal",
            Self::ZeroPadded => "zero-padded",
            Self::AlphaLower => "alpha-lower",
            Self::AlphaUpper => "alpha-upper",
            Self::RomanLower => "roman-lower",
            Self::RomanUpper => "roman-upper",
        }
    }

    fn marker(self, index: usize, item_count: usize) -> String {
        let number = index + 1;
        match self {
            Self::Decimal => format!("{number}."),
            Self::ZeroPadded => {
                let width = item_count.to_string().len().max(2);
                format!("{number:0width$}.")
            }
            Self::AlphaLower => format!("{}.", alpha_index(number, false)),
            Self::AlphaUpper => format!("{}.", alpha_index(number, true)),
            Self::RomanLower => format!("{}.", roman_index(number).to_ascii_lowercase()),
            Self::RomanUpper => format!("{}.", roman_index(number)),
        }
    }
}

impl RendererKind {
    fn from_structure(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "table" | "표" => Some(Self::Table),
            "pros-cons" | "pros_cons" | "tradeoff" | "trade-off" | "comparison" | "장단점" => {
                Some(Self::ProsCons)
            }
            "formula" | "equation" | "math" | "수식" | "공식" => Some(Self::Formula),
            "indexed" | "numbered" | "list" | "ordered-list" | "목록" | "리스트" => {
                Some(Self::IndexedList)
            }
            "flow" | "flowchart" | "diagram" | "흐름" => Some(Self::Flow),
            "progress" | "progress-bar" | "progress_bar" | "진행" | "진행상황" | "상태보고" => {
                Some(Self::Progress)
            }
            "tldr" | "tldr-prose" | "summary" | "요약" => Some(Self::TldrProse),
            "prose" | "paragraph" | "plain" | "문단" => Some(Self::Prose),
            _ => None,
        }
    }

    fn default_intent(self) -> ExplanationIntent {
        match self {
            Self::Table => ExplanationIntent::StructuredSummary,
            Self::ProsCons => ExplanationIntent::Comparison,
            Self::Formula => ExplanationIntent::DecisionRule,
            Self::IndexedList => ExplanationIntent::OrderedSteps,
            Self::Flow => ExplanationIntent::ProcessFlow,
            Self::Progress => ExplanationIntent::ProgressReport,
            Self::TldrProse => ExplanationIntent::StatusSummary,
            Self::Prose => ExplanationIntent::GeneralAnswer,
        }
    }
}

const PROMPT_SIGNAL_MAP: &[PromptSignal] = &[
    PromptSignal {
        renderer: RendererKind::ProsCons,
        intent: ExplanationIntent::Comparison,
        kind: PromptSignalKind::Keyword,
        pattern: "장단점|장점.*단점|pros and cons|tradeoff|trade-off|JS|Rust",
    },
    PromptSignal {
        renderer: RendererKind::Flow,
        intent: ExplanationIntent::ProcessFlow,
        kind: PromptSignalKind::Keyword,
        pattern: "흐름|flow|flowchart|process|pipeline",
    },
    PromptSignal {
        renderer: RendererKind::Table,
        intent: ExplanationIntent::StructuredSummary,
        kind: PromptSignalKind::Keyword,
        pattern: "표|table|matrix|아키텍처|architecture|비교표",
    },
    PromptSignal {
        renderer: RendererKind::Formula,
        intent: ExplanationIntent::DecisionRule,
        kind: PromptSignalKind::Keyword,
        pattern: "수식|공식|formula|equation|math|decision rule",
    },
    PromptSignal {
        renderer: RendererKind::IndexedList,
        intent: ExplanationIntent::OrderedSteps,
        kind: PromptSignalKind::Keyword,
        pattern: "1,2,3|번호|순번|목록|리스트|단계별|numbered|indexed|list",
    },
    PromptSignal {
        renderer: RendererKind::Progress,
        intent: ExplanationIntent::ProgressReport,
        kind: PromptSignalKind::Keyword,
        pattern:
            "progress|progress bar|진행상황|진행 상황|진척|몇 퍼센트|percent|상태 보고|작업 상태",
    },
    PromptSignal {
        renderer: RendererKind::TldrProse,
        intent: ExplanationIntent::StatusSummary,
        kind: PromptSignalKind::Keyword,
        pattern: "tldr|tl;dr|요약|핵심만|현재 상태|status",
    },
    PromptSignal {
        renderer: RendererKind::Prose,
        intent: ExplanationIntent::GeneralAnswer,
        kind: PromptSignalKind::Default,
        pattern: "default-prose",
    },
];

fn prompt_signal_map() -> &'static [PromptSignal] {
    PROMPT_SIGNAL_MAP
}

fn select_renderer(prompt: &str, profile: &Profile) -> RendererSelection {
    if profile.preferred_structure.trim() != "auto" {
        if let Some(renderer) = RendererKind::from_structure(&profile.preferred_structure) {
            return RendererSelection {
                renderer,
                intent: renderer.default_intent(),
                signal: PromptSignal {
                    renderer,
                    intent: renderer.default_intent(),
                    kind: PromptSignalKind::ProfilePreference,
                    pattern: "preferredStructure",
                },
            };
        }
    }

    prompt_signal_map()
        .iter()
        .copied()
        .find(|signal| {
            signal.kind != PromptSignalKind::Default && prompt_matches_signal(prompt, *signal)
        })
        .map(|signal| RendererSelection {
            renderer: signal.renderer,
            intent: signal.intent,
            signal,
        })
        .unwrap_or(RendererSelection {
            renderer: RendererKind::Prose,
            intent: ExplanationIntent::GeneralAnswer,
            signal: PromptSignal {
                renderer: RendererKind::Prose,
                intent: ExplanationIntent::GeneralAnswer,
                kind: PromptSignalKind::Default,
                pattern: "default-prose",
            },
        })
}

fn requested_renderers(prompt: &str) -> Vec<RendererKind> {
    let mut renderers = Vec::new();
    for signal in prompt_signal_map()
        .iter()
        .copied()
        .filter(|signal| signal.kind != PromptSignalKind::Default)
    {
        if prompt_matches_signal(prompt, signal) && !renderers.contains(&signal.renderer) {
            renderers.push(signal.renderer);
        }
    }
    renderers
}

fn push_ux_component(items: &mut Vec<UxComponent>, item: UxComponent) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn clamp_control(value: i32) -> u8 {
    value.clamp(0, 100) as u8
}

fn parse_control_value(value: &str) -> Option<u8> {
    value.trim().parse::<i32>().ok().map(clamp_control)
}

fn normalize_explanation_depth(value: &str, fallback: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "light" | "brief" | "simple" | "shallow" | "low" | "얕게" | "간단" => {
            "light".to_string()
        }
        "standard" | "balanced" | "normal" | "medium" | "mid" | "보통" => "standard".to_string(),
        "deep" | "detailed" | "high" | "깊게" | "자세" | "상세" => "deep".to_string(),
        _ => fallback.to_string(),
    }
}

fn normalize_architecture_depth(value: &str, fallback: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "overview" | "surface" | "light" | "low" | "요약" | "개요" => "overview".to_string(),
        "system" | "standard" | "balanced" | "architecture" | "medium" | "보통" => {
            "system".to_string()
        }
        "internals" | "internal" | "implementation" | "deep" | "high" | "내부" | "구현" => {
            "internals".to_string()
        }
        _ => fallback.to_string(),
    }
}

fn normalize_abstraction_level(value: &str, fallback: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "concrete" | "implementation" | "low" | "code" | "구체" | "코드" => {
            "concrete".to_string()
        }
        "architecture" | "system" | "medium" | "mid" | "구조" | "아키텍처" => {
            "architecture".to_string()
        }
        "strategy" | "strategic" | "high" | "concept" | "전략" | "상위" => {
            "strategy".to_string()
        }
        _ => fallback.to_string(),
    }
}

fn parse_ux_component(value: &str) -> Option<UxComponent> {
    let text = value
        .trim()
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
        .to_ascii_lowercase()
        .replace('_', "-");
    match text.as_str() {
        "badge" | "status" | "status-badge" | "state-badge" => Some(UxComponent::StatusBadge),
        "check" | "checks" | "checklist" | "todo" => Some(UxComponent::Checklist),
        "risk" | "risks" | "risk-panel" | "warning" => Some(UxComponent::RiskPanel),
        "confidence" | "confidence-meter" | "certainty" => Some(UxComponent::ConfidenceMeter),
        "diff" | "diff-summary" | "change" | "changes" => Some(UxComponent::DiffSummary),
        "decision" | "decision-matrix" | "matrix" => Some(UxComponent::DecisionMatrix),
        "next" | "next-action" | "action" | "footer" => Some(UxComponent::NextAction),
        "eta" | "eta-strip" | "elapsed" | "time" => Some(UxComponent::EtaStrip),
        "callout" | "attention" | "attention-callout" | "important" => {
            Some(UxComponent::AttentionCallout)
        }
        _ => None,
    }
}

fn parse_ux_component_plan(plan: &str) -> Vec<UxComponent> {
    let mut items = Vec::new();
    let mut token = String::new();
    for ch in plan.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            token.push(ch);
            continue;
        }
        if let Some(component) = parse_ux_component(&token) {
            push_ux_component(&mut items, component);
        }
        token.clear();
    }
    if let Some(component) = parse_ux_component(&token) {
        push_ux_component(&mut items, component);
    }
    items
}

fn planner_ux_components(prompt: &str, response: &str) -> Vec<UxComponent> {
    if let Ok(plan) = env::var("CODEXPLAIN_UX_PLAN").or_else(|_| env::var("CLAUDEX_UX_PLAN")) {
        let components = parse_ux_component_plan(&plan);
        if !components.is_empty() {
            return components;
        }
    }

    let Ok(command) = env::var("CODEXPLAIN_UX_PLANNER_COMMAND") else {
        return Vec::new();
    };
    let parts = command.split_whitespace().collect::<Vec<_>>();
    let Some(program) = parts.first() else {
        return Vec::new();
    };
    let output = Command::new(program)
        .args(parts.iter().skip(1))
        .env("CODEXPLAIN_PROMPT", prompt)
        .env("CODEXPLAIN_RESPONSE", response)
        .output();
    match output {
        Ok(result) if result.status.success() => {
            parse_ux_component_plan(&String::from_utf8_lossy(&result.stdout))
        }
        _ => Vec::new(),
    }
}

fn add_component_score(scores: &mut Vec<(UxComponent, i32)>, component: UxComponent, score: i32) {
    if let Some((_, current)) = scores.iter_mut().find(|(item, _)| *item == component) {
        *current = (*current).max(score);
    } else {
        scores.push((component, score));
    }
}

fn ux_component_threshold(profile: &Profile) -> i32 {
    (100 - profile.ux_density as i32).clamp(10, 95)
}

fn requested_ux_components(prompt: &str, response: &str, profile: &Profile) -> Vec<UxComponent> {
    let text = format!("{} {}", prompt, response).to_ascii_lowercase();
    let prompt_lower = prompt.to_ascii_lowercase();
    let full_kit = prompt_lower.contains("ux")
        || prompt_lower.contains("ui")
        || prompt.contains("풍부")
        || prompt.contains("모두")
        || prompt.contains("시각적")
        || prompt_lower.contains("dashboard");
    let mut scores = Vec::new();

    for component in planner_ux_components(prompt, response) {
        add_component_score(&mut scores, component, 130);
    }

    let explicit_score = 120;
    if full_kit
        || text.contains("badge")
        || prompt.contains("상태 라벨")
        || prompt.contains("상태 배지")
    {
        add_component_score(&mut scores, UxComponent::StatusBadge, explicit_score);
    }
    if full_kit
        || text.contains("checklist")
        || prompt.contains("체크리스트")
        || prompt.contains("완료 항목")
    {
        add_component_score(&mut scores, UxComponent::Checklist, explicit_score);
    }
    if full_kit
        || text.contains("risk")
        || prompt.contains("위험")
        || prompt.contains("리스크")
        || prompt.contains("막힌")
    {
        add_component_score(&mut scores, UxComponent::RiskPanel, explicit_score);
    }
    if full_kit
        || text.contains("confidence")
        || prompt.contains("확신")
        || prompt.contains("신뢰도")
    {
        add_component_score(&mut scores, UxComponent::ConfidenceMeter, explicit_score);
    }
    if full_kit || text.contains("diff") || prompt.contains("변경 요약") || prompt.contains("바뀐")
    {
        add_component_score(&mut scores, UxComponent::DiffSummary, explicit_score);
    }
    if full_kit
        || text.contains("decision")
        || prompt.contains("결정")
        || prompt.contains("의사결정")
        || prompt.contains("matrix")
    {
        add_component_score(&mut scores, UxComponent::DecisionMatrix, explicit_score);
    }
    if full_kit
        || text.contains("next action")
        || prompt.contains("다음 행동")
        || prompt.contains("다음 액션")
    {
        add_component_score(&mut scores, UxComponent::NextAction, explicit_score);
    }
    if full_kit
        || text.contains("eta")
        || prompt.contains("예상")
        || prompt.contains("남은 시간")
        || prompt.contains("경과")
    {
        add_component_score(&mut scores, UxComponent::EtaStrip, explicit_score);
    }
    if full_kit
        || text.contains("callout")
        || prompt.contains("주의")
        || prompt.contains("강조")
        || prompt.contains("중요")
    {
        add_component_score(&mut scores, UxComponent::AttentionCallout, explicit_score);
    }

    let lower = response.to_ascii_lowercase();
    if lower.contains("fail") || lower.contains("error") || response.contains("실패") {
        let safety_score = 55 + (profile.risk_sensitivity as i32 / 2);
        add_component_score(&mut scores, UxComponent::StatusBadge, safety_score);
        add_component_score(&mut scores, UxComponent::RiskPanel, safety_score + 10);
        add_component_score(&mut scores, UxComponent::AttentionCallout, safety_score + 5);
        add_component_score(&mut scores, UxComponent::NextAction, safety_score);
    }
    if renderer_signal_present(prompt, RendererKind::Progress) {
        let progress_score = 45 + (profile.ux_density as i32 / 2);
        add_component_score(&mut scores, UxComponent::StatusBadge, progress_score);
        add_component_score(&mut scores, UxComponent::Checklist, progress_score);
        add_component_score(&mut scores, UxComponent::NextAction, progress_score);
    }

    let threshold = ux_component_threshold(profile);
    let mut items = Vec::new();
    for (component, score) in scores {
        if score >= threshold {
            push_ux_component(&mut items, component);
        }
    }
    items
}

fn prompt_matches_signal(prompt: &str, signal: PromptSignal) -> bool {
    signal
        .pattern
        .split('|')
        .any(|pattern| prompt_matches_pattern(prompt, pattern))
}

fn prompt_matches_pattern(prompt: &str, pattern: &str) -> bool {
    let text = prompt.to_ascii_lowercase();
    let pattern = pattern.trim();
    let lower_pattern = pattern.to_ascii_lowercase();

    if lower_pattern.contains(".*") {
        let mut cursor = 0;
        for part in lower_pattern.split(".*").filter(|part| !part.is_empty()) {
            let Some(offset) = text[cursor..].find(part) else {
                return false;
            };
            cursor += offset + part.len();
        }
        return true;
    }

    text.contains(&lower_pattern)
}

impl IndexedList {
    fn new(items: impl IntoIterator<Item = String>, style: IndexStyle, max_width: usize) -> Self {
        Self {
            items: items.into_iter().collect(),
            style,
            max_width,
        }
    }

    fn layout(&self, frame: Frame) -> IndexedListLayout {
        IndexedListLayout::fit(self, frame.preset(FramePreset::Indexed))
    }
}

impl IndexedListLayout {
    fn fit(list: &IndexedList, spec: FrameSpec) -> Self {
        let marker_width = list
            .items
            .iter()
            .enumerate()
            .map(|(index, _)| visible_width(&list.style.marker(index, list.items.len())))
            .max()
            .unwrap_or(2)
            .max(2);
        let separator_width = 3;
        let content_width = list
            .max_width
            .saturating_sub(marker_width + separator_width)
            .max(10);

        Self {
            spec,
            marker_width,
            content_width,
        }
    }

    fn gutter(&self, theme: Theme) -> String {
        color(theme, "border", &self.spec.border.vertical.to_string())
    }

    fn marker(&self, marker: &str, theme: Theme) -> String {
        color(theme, "heading", &pad_left(marker, self.marker_width))
    }

    fn continuation(&self) -> String {
        " ".repeat(self.marker_width)
    }
}

fn fit_column_widths(desired: &[usize], budget: usize, min_column_width: usize) -> Vec<usize> {
    let mut widths = desired
        .iter()
        .map(|width| (*width).min(min_column_width).max(1))
        .collect::<Vec<_>>();
    let mut remaining = budget.saturating_sub(widths.iter().sum::<usize>());

    for (index, desired_width) in desired.iter().enumerate() {
        if remaining == 0 {
            break;
        }
        if *desired_width <= min_column_width + 4 && widths[index] < *desired_width {
            let extra = (*desired_width - widths[index]).min(remaining);
            widths[index] += extra;
            remaining -= extra;
        }
    }

    while remaining > 0 {
        let Some((index, needed)) = widths
            .iter()
            .enumerate()
            .map(|(index, width)| (index, desired[index].saturating_sub(*width)))
            .max_by_key(|(_, needed)| *needed)
        else {
            break;
        };
        if needed == 0 {
            break;
        }
        widths[index] += 1;
        remaining -= 1;
    }

    widths
}

impl FrameSpec {
    fn ascii_table() -> Self {
        Self {
            border: FrameBorder {
                horizontal: '-',
                vertical: '|',
            },
            corners: FrameCorners {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
            },
            separators: FrameSeparators {
                top_join: '+',
                middle_left: '+',
                middle_join: '+',
                middle_right: '+',
                bottom_join: '+',
                up_join: '+',
                down_join: '+',
                arrow_down: 'v',
            },
            padding: FramePadding { left: 1, right: 1 },
        }
    }

    fn ascii_flow() -> Self {
        Self {
            separators: FrameSeparators {
                arrow_down: 'v',
                ..Self::ascii_table().separators
            },
            ..Self::ascii_table()
        }
    }

    fn ascii_indexed() -> Self {
        Self::ascii_table()
    }

    fn unicode_table() -> Self {
        Self {
            border: FrameBorder {
                horizontal: '─',
                vertical: '│',
            },
            corners: FrameCorners {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
            },
            separators: FrameSeparators {
                top_join: '┬',
                middle_left: '├',
                middle_join: '┼',
                middle_right: '┤',
                bottom_join: '┴',
                up_join: '┴',
                down_join: '┬',
                arrow_down: '▼',
            },
            padding: FramePadding { left: 1, right: 1 },
        }
    }

    fn unicode_flow() -> Self {
        Self::unicode_table()
    }

    fn unicode_indexed() -> Self {
        Self::unicode_table()
    }

    fn ascii_formula() -> Self {
        Self::ascii_table()
    }

    fn unicode_formula() -> Self {
        Self::unicode_table()
    }

    fn rule(self, rule: FrameRule, widths: &[usize]) -> String {
        let (left, join, right) = match rule {
            FrameRule::Top => (
                self.corners.top_left,
                self.separators.top_join,
                self.corners.top_right,
            ),
            FrameRule::Middle | FrameRule::RowDivider => (
                self.separators.middle_left,
                self.separators.middle_join,
                self.separators.middle_right,
            ),
            FrameRule::Bottom => (
                self.corners.bottom_left,
                self.separators.bottom_join,
                self.corners.bottom_right,
            ),
        };
        let mut line = FrameLine::new().glyph(left);
        for (index, width) in widths.iter().enumerate() {
            line = line
                .repeat(self.border.horizontal, width + self.padding.total())
                .glyph(if index + 1 == widths.len() {
                    right
                } else {
                    join
                });
        }
        line.render()
    }

    fn padded_cell(self, text: &str, width: usize) -> String {
        self.padding.apply(text, width)
    }
}

impl AnsiRole {
    fn parse(value: &str) -> Self {
        match value {
            "border" => Self::Border,
            "heading" => Self::Heading,
            "muted" => Self::Muted,
            "success" => Self::Success,
            "warning" => Self::Warning,
            "danger" => Self::Danger,
            _ => Self::Accent,
        }
    }
}

impl AnsiStyle {
    const PLAIN: Self = Self { prefix: "" };

    const fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }

    fn apply(self, value: &str) -> String {
        if self.prefix.is_empty() {
            value.to_string()
        } else {
            format!("{}{value}\x1b[0m", self.prefix)
        }
    }
}

impl ThemeSpec {
    const PLAIN: Self = Self {
        border: AnsiStyle::PLAIN,
        heading: AnsiStyle::PLAIN,
        accent: AnsiStyle::PLAIN,
        muted: AnsiStyle::PLAIN,
        success: AnsiStyle::PLAIN,
        warning: AnsiStyle::PLAIN,
        danger: AnsiStyle::PLAIN,
    };

    fn style(self, role: AnsiRole) -> AnsiStyle {
        match role {
            AnsiRole::Border => self.border,
            AnsiRole::Heading => self.heading,
            AnsiRole::Accent => self.accent,
            AnsiRole::Muted => self.muted,
            AnsiRole::Success => self.success,
            AnsiRole::Warning => self.warning,
            AnsiRole::Danger => self.danger,
        }
    }
}

impl Theme {
    fn parse(value: Option<&str>) -> Self {
        match value
            .unwrap_or("ocean")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "none" | "off" | "0" | "false" | "plain" | "no-color" | "no_color" => Self::None,
            "forest" => Self::Forest,
            "warm" => Self::Warm,
            "sunset" => Self::Sunset,
            "grape" => Self::Grape,
            "slate" => Self::Slate,
            "rose" => Self::Rose,
            "mono" => Self::Mono,
            _ => Self::Ocean,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ocean => "ocean",
            Self::Forest => "forest",
            Self::Warm => "warm",
            Self::Sunset => "sunset",
            Self::Grape => "grape",
            Self::Slate => "slate",
            Self::Rose => "rose",
            Self::Mono => "mono",
        }
    }

    fn spec(self) -> ThemeSpec {
        match self {
            Self::None => ThemeSpec::PLAIN,
            Self::Ocean => ThemeSpec {
                border: AnsiStyle::new("\x1b[36m"),
                heading: AnsiStyle::new("\x1b[1;34m"),
                accent: AnsiStyle::new("\x1b[96m"),
                muted: AnsiStyle::new("\x1b[2;36m"),
                success: AnsiStyle::new("\x1b[1;32m"),
                warning: AnsiStyle::new("\x1b[1;33m"),
                danger: AnsiStyle::new("\x1b[1;31m"),
            },
            Self::Forest => ThemeSpec {
                border: AnsiStyle::new("\x1b[32m"),
                heading: AnsiStyle::new("\x1b[1;32m"),
                accent: AnsiStyle::new("\x1b[92m"),
                muted: AnsiStyle::new("\x1b[2;32m"),
                success: AnsiStyle::new("\x1b[1;32m"),
                warning: AnsiStyle::new("\x1b[1;33m"),
                danger: AnsiStyle::new("\x1b[1;31m"),
            },
            Self::Warm => ThemeSpec {
                border: AnsiStyle::new("\x1b[33m"),
                heading: AnsiStyle::new("\x1b[1;31m"),
                accent: AnsiStyle::new("\x1b[93m"),
                muted: AnsiStyle::new("\x1b[2;33m"),
                success: AnsiStyle::new("\x1b[1;32m"),
                warning: AnsiStyle::new("\x1b[1;33m"),
                danger: AnsiStyle::new("\x1b[1;31m"),
            },
            Self::Sunset => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;208m"),
                heading: AnsiStyle::new("\x1b[1;38;5;196m"),
                accent: AnsiStyle::new("\x1b[38;5;214m"),
                muted: AnsiStyle::new("\x1b[2;38;5;208m"),
                success: AnsiStyle::new("\x1b[1;38;5;118m"),
                warning: AnsiStyle::new("\x1b[1;38;5;220m"),
                danger: AnsiStyle::new("\x1b[1;38;5;196m"),
            },
            Self::Grape => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;141m"),
                heading: AnsiStyle::new("\x1b[1;38;5;135m"),
                accent: AnsiStyle::new("\x1b[38;5;183m"),
                muted: AnsiStyle::new("\x1b[2;38;5;141m"),
                success: AnsiStyle::new("\x1b[1;38;5;120m"),
                warning: AnsiStyle::new("\x1b[1;38;5;222m"),
                danger: AnsiStyle::new("\x1b[1;38;5;204m"),
            },
            Self::Slate => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;67m"),
                heading: AnsiStyle::new("\x1b[1;38;5;110m"),
                accent: AnsiStyle::new("\x1b[38;5;153m"),
                muted: AnsiStyle::new("\x1b[2;38;5;67m"),
                success: AnsiStyle::new("\x1b[1;38;5;114m"),
                warning: AnsiStyle::new("\x1b[1;38;5;179m"),
                danger: AnsiStyle::new("\x1b[1;38;5;167m"),
            },
            Self::Rose => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;211m"),
                heading: AnsiStyle::new("\x1b[1;38;5;199m"),
                accent: AnsiStyle::new("\x1b[38;5;218m"),
                muted: AnsiStyle::new("\x1b[2;38;5;211m"),
                success: AnsiStyle::new("\x1b[1;38;5;120m"),
                warning: AnsiStyle::new("\x1b[1;38;5;222m"),
                danger: AnsiStyle::new("\x1b[1;38;5;197m"),
            },
            Self::Mono => ThemeSpec {
                border: AnsiStyle::new("\x1b[90m"),
                heading: AnsiStyle::new("\x1b[1m"),
                accent: AnsiStyle::new("\x1b[37m"),
                muted: AnsiStyle::new("\x1b[2m"),
                success: AnsiStyle::new("\x1b[1m"),
                warning: AnsiStyle::new("\x1b[1m"),
                danger: AnsiStyle::new("\x1b[1m"),
            },
        }
    }

    fn style(self, role: AnsiRole) -> AnsiStyle {
        self.spec().style(role)
    }

    fn apply_terminal_policy<F>(self, env_value: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        if terminal_supports_ansi(env_value) {
            self
        } else {
            Self::None
        }
    }
}

fn color(theme: Theme, role: &str, value: &str) -> String {
    theme.style(AnsiRole::parse(role)).apply(value)
}

fn role_for(value: &str, fallback: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "tldr" | "핵심" | "결론" | "장점" | "pros" | "success" => "success",
        "단점" | "위험" | "주의" | "cons" | "risk" | "warning" => "warning",
        "오류" | "실패" | "danger" | "error" => "danger",
        _ => match fallback {
            "heading" => "heading",
            "border" => "border",
            _ => "accent",
        },
    }
}

fn visible_width(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            consume_ansi_escape(&mut chars);
            continue;
        }
        width += char_width(ch);
    }
    width
}

fn pad(text: &str, width: usize) -> String {
    format!(
        "{text}{}",
        " ".repeat(width.saturating_sub(visible_width(text)))
    )
}

fn pad_left(text: &str, width: usize) -> String {
    format!(
        "{}{text}",
        " ".repeat(width.saturating_sub(visible_width(text)))
    )
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            current.push(ch);
            append_ansi_escape(&mut chars, &mut current);
            continue;
        }
        let ch_width = char_width(ch);
        if ch == '\n' || current_width + ch_width > width {
            lines.push(current.trim_end().to_string());
            current.clear();
            current_width = 0;
            if ch == '\n' {
                continue;
            }
        }
        current.push(ch);
        current_width += ch_width;
    }
    if !current.is_empty() {
        lines.push(current.trim_end().to_string());
    }
    lines
}

fn consume_ansi_escape<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    if matches!(chars.peek(), Some('[')) {
        chars.next();
        while let Some(ch) = chars.next() {
            if ('@'..='~').contains(&ch) {
                break;
            }
        }
    } else {
        let _ = chars.next();
    }
}

fn append_ansi_escape<I>(chars: &mut std::iter::Peekable<I>, out: &mut String)
where
    I: Iterator<Item = char>,
{
    if matches!(chars.peek(), Some('[')) {
        out.push(chars.next().unwrap_or('['));
        while let Some(ch) = chars.next() {
            out.push(ch);
            if ('@'..='~').contains(&ch) {
                break;
            }
        }
    } else if let Some(ch) = chars.next() {
        out.push(ch);
    }
}

fn char_width(ch: char) -> usize {
    let code = ch as u32;
    if ch == '\t' {
        return 4;
    }
    if code == 0 || code < 0x20 || is_combining(code) {
        return 0;
    }
    if is_wide(code) {
        2
    } else {
        1
    }
}

fn is_combining(code: u32) -> bool {
    (0x0300..=0x036f).contains(&code)
        || (0x1ab0..=0x1aff).contains(&code)
        || (0x1dc0..=0x1dff).contains(&code)
        || (0x20d0..=0x20ff).contains(&code)
        || (0xfe20..=0xfe2f).contains(&code)
}

fn is_wide(code: u32) -> bool {
    code >= 0x1100
        && (code <= 0x115f
            || code == 0x2329
            || code == 0x232a
            || ((0x2e80..=0xa4cf).contains(&code) && code != 0x303f)
            || (0xac00..=0xd7a3).contains(&code)
            || (0xf900..=0xfaff).contains(&code)
            || (0xfe10..=0xfe19).contains(&code)
            || (0xfe30..=0xfe6f).contains(&code)
            || (0xff00..=0xff60).contains(&code)
            || (0x1f300..=0x1faff).contains(&code))
}

fn alpha_index(mut number: usize, uppercase: bool) -> String {
    if number == 0 {
        return String::new();
    }
    let mut chars = Vec::new();
    while number > 0 {
        number -= 1;
        let offset = (number % 26) as u8;
        let base = if uppercase { b'A' } else { b'a' };
        chars.push((base + offset) as char);
        number /= 26;
    }
    chars.iter().rev().collect()
}

fn roman_index(mut number: usize) -> String {
    if number == 0 {
        return String::new();
    }
    const VALUES: &[(usize, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (value, glyph) in VALUES {
        while number >= *value {
            out.push_str(glyph);
            number -= *value;
        }
    }
    out
}

fn compact(text: &str, limit: usize) -> String {
    let mut out = Vec::new();
    let mut sentence = String::new();
    for ch in text.replace('\n', " ").chars() {
        sentence.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            if !sentence.trim().is_empty() {
                out.push(sentence.trim().to_string());
            }
            sentence.clear();
            if out.len() >= limit {
                break;
            }
        }
    }
    if out.is_empty() && !sentence.trim().is_empty() {
        out.push(sentence.trim().to_string());
    }
    if out.is_empty() {
        text.split_whitespace()
            .take(40)
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        out.join(" ")
    }
}

fn table(
    headers: &[&str],
    rows: &[Vec<String>],
    frame: Frame,
    theme: Theme,
    row_dividers: bool,
    max_width: usize,
) -> String {
    let table = Table::new(headers, rows, row_dividers, max_width);
    render_table_model(&table, frame, theme)
}

fn render_table_model(table: &Table, frame: Frame, theme: Theme) -> String {
    let layout = table.layout(frame);

    let mut lines = Vec::new();
    lines.push(layout.border(FrameRule::Top, theme));
    lines.extend(wrapped_row(&table.headers, &layout, theme, "heading"));
    lines.push(layout.border(FrameRule::Middle, theme));
    for (row_index, item) in table.rows.iter().enumerate() {
        lines.extend(wrapped_row(item, &layout, theme, "accent"));
        if table.row_dividers && row_index + 1 < table.rows.len() {
            lines.push(layout.row_divider(theme));
        }
    }
    lines.push(layout.border(FrameRule::Bottom, theme));
    lines.join("\n")
}

fn render_responsive_panels(left: &str, right: &str, width: usize, gap: usize) -> String {
    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines: Vec<&str> = right.lines().collect();
    let left_width = left_lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    let right_width = right_lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);

    if left_width + gap + right_width > width {
        return format!("{left}\n\n{right}");
    }

    let mut lines = Vec::new();
    let line_count = left_lines.len().max(right_lines.len());
    let spacer = " ".repeat(gap);
    for index in 0..line_count {
        let left_line = left_lines.get(index).copied().unwrap_or("");
        let right_line = right_lines.get(index).copied().unwrap_or("");
        lines.push(format!(
            "{}{}{}",
            pad(left_line, left_width),
            spacer,
            right_line
        ));
    }
    lines.join("\n")
}

fn wrapped_row(
    row: &TableRow,
    layout: &TableLayout,
    theme: Theme,
    default_role: &str,
) -> Vec<String> {
    let wrapped: Vec<Vec<String>> = layout
        .widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            row.cells
                .get(index)
                .map(|cell| cell.wrapped(*width))
                .unwrap_or_else(|| vec![String::new()])
        })
        .collect();
    let line_count = wrapped.iter().map(Vec::len).max().unwrap_or(1);
    let mut lines = Vec::new();
    for line_index in 0..line_count {
        let mut line = FrameLine::new().text(color(
            theme,
            "border",
            &layout.spec.border.vertical.to_string(),
        ));
        for (cell_index, width) in layout.widths.iter().enumerate() {
            let cell = wrapped[cell_index]
                .get(line_index)
                .map(String::as_str)
                .unwrap_or("");
            let role = role_for(cell, default_role);
            line = line.text(color(theme, role, &layout.padded_cell(cell, *width)));
            line = line.text(color(
                theme,
                "border",
                &layout.spec.border.vertical.to_string(),
            ));
        }
        lines.push(line.render());
    }
    lines
}

fn codexplain_flow(frame: Frame, theme: Theme, max_width: usize) -> String {
    let diagram = FlowDiagram::new(
        [
            FlowStep::new("입력"),
            FlowStep::with_branches(
                "정책 검사",
                ["strict 출력 보존".to_string(), "설명 UX 렌더링".to_string()],
            ),
            FlowStep::new("UX 프로필"),
            FlowStep::new("출력"),
        ],
        max_width,
    );
    render_flow_diagram(&diagram, frame, theme)
}

fn architecture_panels(profile: &Profile, summary: &str, width: usize) -> String {
    let stacked_width = width.max(50);
    let panel_width = if width >= 112 {
        ((width - 3) / 2).max(40)
    } else {
        stacked_width
    };
    let table_panel = table(
        &["계층", "역할"],
        &layer_rows(summary, profile),
        profile.frame,
        profile.theme,
        true,
        panel_width,
    );
    let flow_panel = codexplain_flow(profile.frame, profile.theme, panel_width);
    render_responsive_panels(&table_panel, &flow_panel, width, 3)
}

fn render_flow_diagram(diagram: &FlowDiagram, frame: Frame, theme: Theme) -> String {
    if diagram.steps.is_empty() {
        return String::new();
    }
    let layout = diagram.layout(frame);
    let mut lines = Vec::new();
    for (index, step) in diagram.steps.iter().enumerate() {
        if index > 0 {
            lines.extend(flow_sequence_connector(&layout, theme));
        }

        lines.extend(flow_step_box(
            &step.label,
            &layout,
            index == 0,
            index + 1 == diagram.steps.len() && step.branches.is_empty(),
            theme,
        ));
        if !step.branches.is_empty() {
            lines.extend(flow_branch_block(&step.branches, &layout, frame, theme));
        }
    }
    lines.join("\n")
}

fn flow_sequence_connector(layout: &FlowLayout, theme: Theme) -> Vec<String> {
    let spine = layout.spine_indent();
    vec![
        format!(
            "{spine}{}",
            color(theme, "border", &layout.spec.border.vertical.to_string())
        ),
        format!(
            "{spine}{}",
            color(
                theme,
                "heading",
                &layout.spec.separators.arrow_down.to_string()
            )
        ),
    ]
}

fn flow_step_box(
    text: &str,
    layout: &FlowLayout,
    is_first: bool,
    is_last: bool,
    theme: Theme,
) -> Vec<String> {
    let top_mid = if is_first {
        layout.spec.border.horizontal
    } else {
        layout.spec.separators.up_join
    };
    let bottom_mid = if is_last {
        layout.spec.border.horizontal
    } else {
        layout.spec.separators.down_join
    };
    let mut lines = vec![flow_border(
        layout.spec.corners.top_left,
        top_mid,
        layout.spec.corners.top_right,
        layout.content_width,
        layout.spec,
        theme,
    )];
    for line in wrap_text(text, layout.content_width) {
        lines.push(layout.box_line(&line, theme, "accent"));
    }
    lines.push(flow_border(
        layout.spec.corners.bottom_left,
        bottom_mid,
        layout.spec.corners.bottom_right,
        layout.content_width,
        layout.spec,
        theme,
    ));
    lines
}

fn flow_branch_block(
    branches: &[String],
    layout: &FlowLayout,
    frame: Frame,
    theme: Theme,
) -> Vec<String> {
    let mut lines = flow_sequence_connector(layout, theme);
    let spine = layout.spine_indent();
    let branch_count = branches.len();

    for (index, branch) in branches.iter().enumerate() {
        let connector = branch_connector(frame, index + 1 == branch_count);
        for (line_index, line) in wrap_text(branch, layout.content_width).iter().enumerate() {
            let prefix = if line_index == 0 {
                connector.clone()
            } else {
                branch_continuation(frame)
            };
            lines.push(format!(
                "{spine}{} {}",
                color(theme, "border", &prefix),
                color(theme, "accent", line)
            ));
        }
    }
    lines
}

fn branch_connector(frame: Frame, is_last: bool) -> String {
    match (frame, is_last) {
        (Frame::Unicode, false) => "├─▶".to_string(),
        (Frame::Unicode, true) => "└─▶".to_string(),
        (Frame::Ascii, false) => "+->".to_string(),
        (Frame::Ascii, true) => "`->".to_string(),
    }
}

fn branch_continuation(frame: Frame) -> String {
    match frame {
        Frame::Unicode => "│  ".to_string(),
        Frame::Ascii => "|  ".to_string(),
    }
}

fn flow_border(
    left: char,
    center: char,
    right: char,
    width: usize,
    spec: FrameSpec,
    theme: Theme,
) -> String {
    let dash_count = width + 2;
    let center_index = dash_count / 2;
    let value = FrameLine::new()
        .glyph(left)
        .repeat(spec.border.horizontal, center_index)
        .glyph(center)
        .repeat(
            spec.border.horizontal,
            dash_count.saturating_sub(center_index + 1),
        )
        .glyph(right)
        .render();
    color(theme, "border", &value)
}

fn indexed(
    items: &[String],
    frame: Frame,
    theme: Theme,
    width: usize,
    style: IndexStyle,
) -> String {
    let list = IndexedList::new(items.iter().cloned(), style, width);
    render_indexed_list(&list, frame, theme)
}

fn render_indexed_list(list: &IndexedList, frame: Frame, theme: Theme) -> String {
    let layout = list.layout(frame);
    let item_count = list.items.len();

    list.items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let marker = list.style.marker(index, item_count);
            let marker = layout.marker(&marker, theme);
            let continuation = layout.continuation();
            let gutter = layout.gutter(theme);

            wrap_text(item, layout.content_width)
                .into_iter()
                .enumerate()
                .map(|(line_index, line)| {
                    let marker = if line_index == 0 {
                        marker.clone()
                    } else {
                        continuation.clone()
                    };
                    format!(
                        "{marker} {gutter} {}",
                        color(theme, role_for(&line, "accent"), &line)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn progress_percent(response: &str) -> usize {
    for token in response.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != '%' && ch != '/');
        if let Some(value) = cleaned.strip_suffix('%') {
            if let Ok(percent) = value.parse::<usize>() {
                return percent.min(100);
            }
        }
        if let Some((done, total)) = cleaned.split_once('/') {
            if let (Ok(done), Ok(total)) = (done.parse::<usize>(), total.parse::<usize>()) {
                if total > 0 {
                    return ((done * 100) / total).min(100);
                }
            }
        }
    }

    let lower = response.to_ascii_lowercase();
    if response.contains("완료") || lower.contains("done") || lower.contains("pass") {
        100
    } else if response.contains("실패") || lower.contains("fail") || lower.contains("error") {
        0
    } else if response.contains("진행")
        || lower.contains("running")
        || lower.contains("in progress")
    {
        60
    } else if response.contains("대기") || lower.contains("pending") {
        20
    } else {
        50
    }
}

fn progress_label(percent: usize) -> &'static str {
    match percent {
        100 => "완료",
        80..=99 => "마무리 중",
        40..=79 => "진행 중",
        1..=39 => "초기 진행",
        _ => "확인 필요",
    }
}

fn render_progress_bar(percent: usize, width: usize, frame: Frame, theme: Theme) -> String {
    let bar_width = width.clamp(12, 36);
    let filled = ((bar_width * percent.min(100)) + 50) / 100;
    let empty = bar_width.saturating_sub(filled);
    let (fill, blank) = match frame {
        Frame::Unicode => ('█', '░'),
        Frame::Ascii => ('#', '-'),
    };
    let bar = format!(
        "{}{}",
        fill.to_string().repeat(filled),
        blank.to_string().repeat(empty)
    );
    format!(
        "[{}] {:>3}%",
        color(theme, "success", &bar),
        percent.min(100)
    )
}

fn progress_report(profile: &Profile, response: &str, summary: &str, width: usize) -> String {
    let percent = progress_percent(response);
    let status = progress_label(percent);
    let bar_width = width.saturating_sub(18).min(36).max(12);
    let headline = format!(
        "{}{}",
        color(profile.theme, "heading", "진행상황: "),
        color(profile.theme, role_for(status, "accent"), status)
    );
    let bar = render_progress_bar(percent, bar_width, profile.frame, profile.theme);
    let rows = vec![
        vec!["현재".to_string(), compact(summary, 1)],
        vec!["진척".to_string(), format!("{status} · {percent}%")],
        vec![
            "다음 행동".to_string(),
            "막힌 지점, 실패 로그, 남은 검증을 한 줄로 확인합니다.".to_string(),
        ],
    ];
    let detail = table(
        &["항목", "보고"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    );
    format!("{headline}\n{bar}\n\n{detail}")
}

fn status_badge(profile: &Profile, response: &str) -> String {
    let percent = progress_percent(response);
    let label = if percent == 100 {
        "PASS"
    } else if percent == 0 {
        "BLOCKED"
    } else {
        "RUNNING"
    };
    let role = match label {
        "PASS" => "success",
        "BLOCKED" => "danger",
        _ => "warning",
    };
    format!(
        "{} {}",
        color(profile.theme, role, &format!("[{label}]")),
        color(profile.theme, "accent", progress_label(percent))
    )
}

fn checklist(profile: &Profile, summary: &str, width: usize) -> String {
    let rows = vec![
        vec![
            "완료".to_string(),
            "검증 가능한 사실과 출력 근거를 먼저 확인".to_string(),
        ],
        vec!["진행".to_string(), compact(summary, 1)],
        vec![
            "남음".to_string(),
            "사용자 확인 또는 다음 명령 실행".to_string(),
        ],
    ];
    table(
        &["상태", "체크포인트"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn risk_panel(profile: &Profile, response: &str, width: usize) -> String {
    let risk = if response.contains("실패") || response.to_ascii_lowercase().contains("fail") {
        "실패 로그와 재시도 조건을 먼저 확인해야 합니다."
    } else if response.contains("드리프트") || response.to_ascii_lowercase().contains("drift") {
        "작업 문맥이 섞였을 수 있으므로 자동화를 중단해야 합니다."
    } else {
        "숨은 전제, 남은 검증, 저장공간 변화를 확인해야 합니다."
    };
    let rows = vec![
        vec!["위험".to_string(), risk.to_string()],
        vec![
            "대응".to_string(),
            "증거를 확인하고 필요한 경우 범위를 좁혀 재실행합니다.".to_string(),
        ],
    ];
    table(
        &["구분", "내용"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn confidence_percent(response: &str) -> usize {
    let lower = response.to_ascii_lowercase();
    if lower.contains("uncertain") || response.contains("불확실") || response.contains("추정")
    {
        55
    } else if lower.contains("fail") || response.contains("실패") {
        40
    } else if lower.contains("pass") || response.contains("통과") || response.contains("완료") {
        90
    } else {
        75
    }
}

fn confidence_meter(profile: &Profile, response: &str, width: usize) -> String {
    let percent = confidence_percent(response);
    format!(
        "{}\n{}",
        color(profile.theme, "heading", "확신도"),
        render_progress_bar(
            percent,
            width.saturating_sub(12).min(24).max(12),
            profile.frame,
            profile.theme
        )
    )
}

fn diff_summary_card(profile: &Profile, summary: &str, width: usize) -> String {
    let rows = vec![vec![
        "변경".to_string(),
        compact(summary, 1),
        "사용자-facing 설명 UX가 더 구조화됩니다.".to_string(),
        "테스트와 storage-check로 확인합니다.".to_string(),
    ]];
    table(
        &["구분", "무엇", "영향", "검증"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn decision_matrix(profile: &Profile, width: usize) -> String {
    let rows = vec![
        vec![
            "계속 진행".to_string(),
            "높음".to_string(),
            "검증 통과 시 즉시 가치가 있습니다.".to_string(),
        ],
        vec![
            "멈춤".to_string(),
            "중간".to_string(),
            "드리프트나 실패가 있으면 안전합니다.".to_string(),
        ],
        vec![
            "축소 재시도".to_string(),
            "높음".to_string(),
            "범위를 줄여 실패 비용을 낮춥니다.".to_string(),
        ],
    ];
    table(
        &["선택", "점수", "근거"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn next_action_footer(profile: &Profile) -> String {
    format!(
        "{} {}",
        color(profile.theme, "heading", "다음 행동:"),
        color(
            profile.theme,
            "accent",
            "검증 결과, 위험, 남은 항목 중 하나만 선택해 바로 실행합니다."
        )
    )
}

fn eta_strip(profile: &Profile, response: &str) -> String {
    let percent = progress_percent(response);
    let eta = if percent >= 90 {
        "마무리 단계"
    } else if percent >= 50 {
        "검증 1-2단계 남음"
    } else {
        "초기 확인 필요"
    };
    format!(
        "{} {} · {} {}%",
        color(profile.theme, "heading", "ETA:"),
        color(profile.theme, "accent", eta),
        color(profile.theme, "heading", "진척"),
        percent
    )
}

fn attention_callout(profile: &Profile, response: &str, width: usize) -> String {
    let message =
        if response.contains("드리프트") || response.to_ascii_lowercase().contains("drift") {
            "드리프트가 감지되면 자동화 작업을 중단하고 새 Seed로 재시작하세요."
        } else if response.contains("실패") || response.to_ascii_lowercase().contains("fail") {
            "실패 원인과 재현 명령을 먼저 고정하세요."
        } else {
            "색상과 막대는 보조 신호입니다. 텍스트 라벨을 기준으로 판단하세요."
        };
    table(
        &["주의", "내용"],
        &[vec!["중요".to_string(), message.to_string()]],
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn ux_component_output(
    component: UxComponent,
    profile: &Profile,
    response: &str,
    summary: &str,
    width: usize,
) -> String {
    match component {
        UxComponent::StatusBadge => status_badge(profile, response),
        UxComponent::Checklist => checklist(profile, summary, width),
        UxComponent::RiskPanel => risk_panel(profile, response, width),
        UxComponent::ConfidenceMeter => confidence_meter(profile, response, width),
        UxComponent::DiffSummary => diff_summary_card(profile, summary, width),
        UxComponent::DecisionMatrix => decision_matrix(profile, width),
        UxComponent::NextAction => next_action_footer(profile),
        UxComponent::EtaStrip => eta_strip(profile, response),
        UxComponent::AttentionCallout => attention_callout(profile, response, width),
    }
}

fn ux_component_sections(
    profile: &Profile,
    response: &str,
    summary: &str,
    width: usize,
    components: &[UxComponent],
) -> Vec<String> {
    components
        .iter()
        .copied()
        .map(|component| ux_component_output(component, profile, response, summary, width))
        .collect()
}

fn formula(profile: &Profile, summary: &str) -> String {
    let box_model = FormulaBox::new(
        "수식 박스",
        [
            FormulaField::new("핵심식", "설명 품질 = f(명확성, 구조, 근거, 다음 행동)"),
            FormulaField::new(
                "의미",
                "색은 보조 신호이고 텍스트 label/value가 의미를 보존합니다.",
            ),
            FormulaField::new("설명", summary),
        ],
        100,
    );
    render_formula_box(&box_model, profile.frame, profile.theme)
}

fn render_formula_box(box_model: &FormulaBox, frame: Frame, theme: Theme) -> String {
    let layout = box_model.layout(frame);
    let mut lines = Vec::new();
    lines.push(layout.border(FrameRule::Top, theme));
    lines.push(layout.line(&box_model.title, theme, "heading"));
    lines.push(layout.border(FrameRule::Middle, theme));
    for (index, field) in box_model.fields.iter().enumerate() {
        lines.extend(formula_field_lines(field, &layout, theme));
        if index == 0 && box_model.fields.len() > 1 {
            lines.push(layout.border(FrameRule::RowDivider, theme));
        }
    }
    lines.push(layout.border(FrameRule::Bottom, theme));
    lines.join("\n")
}

fn formula_field_lines(
    field: &FormulaField,
    layout: &FormulaBoxLayout,
    theme: Theme,
) -> Vec<String> {
    let separator = " : ";
    let value_width = layout
        .content_width
        .saturating_sub(layout.label_width + visible_width(separator))
        .max(1);
    let label_lines = wrap_text(&field.label, layout.label_width);
    let value_lines = wrap_text(&field.value, value_width);
    let line_count = label_lines.len().max(value_lines.len()).max(1);
    let mut lines = Vec::new();

    for line_index in 0..line_count {
        let label = label_lines
            .get(line_index)
            .map(String::as_str)
            .unwrap_or("");
        let value = value_lines
            .get(line_index)
            .map(String::as_str)
            .unwrap_or("");
        let prefix = if line_index == 0 {
            format!("{}{}", pad(label, layout.label_width), separator)
        } else {
            format!(
                "{}{}",
                " ".repeat(layout.label_width),
                " ".repeat(visible_width(separator))
            )
        };
        let role = if line_index == 0 {
            role_for(label, "heading")
        } else {
            "accent"
        };
        lines.push(layout.line(&format!("{prefix}{}", pad(value, value_width)), theme, role));
    }

    lines
}

fn pros_cons(profile: &Profile) -> String {
    let table = pros_cons_table(120);
    render_table_model(&table, profile.frame, profile.theme)
}

fn pros_cons_table(max_width: usize) -> Table {
    const HEADERS: [&str; 4] = ["선택지", "장점", "단점", "적합한 때"];
    let options = [
        ProsConsOption {
            choice: "JS / Node",
            pros: &["빠른 수정", "provider 연동", "JSON 처리"],
            cons: &["런타임 의존성", "단일 바이너리 약함"],
            best_for: "UX 실험과 피드백 루프",
        },
        ProsConsOption {
            choice: "Rust",
            pros: &["단일 바이너리", "빠른 시작", "낮은 메모리"],
            cons: &["초기 구현 비용", "provider 실험 비용"],
            best_for: "안정화된 CLI core",
        },
    ];
    let rows = options
        .iter()
        .map(|option| {
            vec![
                option.choice.to_string(),
                option.pros.join("\n"),
                option.cons.join("\n"),
                option.best_for.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    Table::new(&HEADERS, &rows, true, max_width)
}

fn summary_sentence_limit(profile: &Profile) -> usize {
    match profile.explanation_depth.as_str() {
        "light" => 3,
        "standard" => 6,
        "deep" => 10,
        _ => match profile.detail.as_str() {
            "brief" => 3,
            "balanced" => 6,
            "deep" => 10,
            _ => 6,
        },
    }
}

fn shape(prompt: &str, response: &str, profile: &Profile, width: usize) -> String {
    if should_back_off(prompt, response) {
        return response.to_string();
    }
    let summary = compact(response, summary_sentence_limit(profile));
    let selection = select_renderer(prompt, profile);
    dispatch_explanation(selection, prompt, response, &summary, profile, width)
}

fn dispatch_explanation(
    selection: RendererSelection,
    prompt: &str,
    response: &str,
    summary: &str,
    profile: &Profile,
    width: usize,
) -> String {
    let requested = requested_renderers(prompt);
    let ux_components = requested_ux_components(prompt, response, profile);
    let wants_architecture = requested.contains(&RendererKind::Table)
        && (requested.contains(&RendererKind::Flow)
            || prompt_matches_pattern(prompt, "아키텍처")
            || prompt_matches_pattern(prompt, "architecture"));

    if requested.len() > 1 || wants_architecture {
        let mut sections = Vec::new();
        if wants_architecture {
            sections.push(architecture_panels(profile, summary, width));
        } else if requested.contains(&RendererKind::Table) {
            sections.push(table(
                &["구분", "내용"],
                &layer_rows(summary, profile),
                profile.frame,
                profile.theme,
                true,
                width,
            ));
        }

        if requested.contains(&RendererKind::ProsCons) {
            sections.push(pros_cons(profile));
        }
        if requested.contains(&RendererKind::Formula) {
            sections.push(formula(
                profile,
                "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다.",
            ));
        }
        if requested.contains(&RendererKind::Progress) {
            sections.push(progress_report(profile, response, summary, width));
        }
        if requested.contains(&RendererKind::IndexedList) {
            let items = split_sentences(summary);
            sections.push(indexed(
                &items,
                profile.frame,
                profile.theme,
                width,
                profile.index_style,
            ));
        }
        sections.extend(ux_component_sections(
            profile,
            response,
            summary,
            width,
            &ux_components,
        ));
        if requested.contains(&RendererKind::Flow) && !wants_architecture {
            sections.push(codexplain_flow(profile.frame, profile.theme, width));
        }

        if !sections.is_empty() {
            return sections.join("\n\n");
        }
    }

    if !ux_components.is_empty() {
        let mut sections = ux_component_sections(profile, response, summary, width, &ux_components);
        if requested.contains(&RendererKind::Progress) {
            sections.insert(0, progress_report(profile, response, summary, width));
        }
        return sections.join("\n\n");
    }

    match selection.intent {
        ExplanationIntent::Comparison => {
            let mut output = pros_cons(profile);
            if renderer_signal_present(prompt, RendererKind::Formula) {
                output.push_str("\n\n");
                output.push_str(&formula(
                    profile,
                    "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다.",
                ));
            }
            output
        }
        ExplanationIntent::OrderedSteps => {
            let items = split_sentences(summary);
            indexed(
                &items,
                profile.frame,
                profile.theme,
                width,
                profile.index_style,
            )
        }
        ExplanationIntent::DecisionRule => formula(profile, summary),
        ExplanationIntent::ProcessFlow => codexplain_flow(profile.frame, profile.theme, width),
        ExplanationIntent::ProgressReport => progress_report(profile, response, summary, width),
        ExplanationIntent::StructuredSummary => table(
            &["구분", "내용"],
            &layer_rows(summary, profile),
            profile.frame,
            profile.theme,
            true,
            width,
        ),
        ExplanationIntent::StatusSummary => format!(
            "{}{}\n{}{}",
            color(profile.theme, "heading", "TLDR: "),
            compact(response, 1),
            color(profile.theme, "heading", "요약하면, "),
            summary
        ),
        ExplanationIntent::GeneralAnswer => format!(
            "{}{}",
            color(profile.theme, "heading", "요약하면, "),
            summary
        ),
    }
}

fn renderer_signal_present(prompt: &str, renderer: RendererKind) -> bool {
    prompt_signal_map()
        .iter()
        .copied()
        .any(|signal| signal.renderer == renderer && prompt_matches_signal(prompt, signal))
}

fn should_back_off(prompt: &str, response: &str) -> bool {
    let prompt_lower = prompt.to_ascii_lowercase();
    let strict_prompt = [
        "only json",
        "raw json",
        "valid json",
        "strict format",
        "exact format",
        "verbatim",
        "commit message",
        "test output",
        "logs",
    ]
    .iter()
    .any(|needle| prompt_lower.contains(needle))
        || prompt.contains("JSON만")
        || prompt.contains("정확한 형식")
        || prompt.contains("엄격한 형식")
        || prompt.contains("커밋 메시지")
        || prompt.contains("로그만")
        || prompt.contains("코드만")
        || prompt.contains("테스트 출력만")
        || prompt.contains("diff만")
        || prompt.contains("patch만")
        || prompt.contains("패치만");
    strict_prompt || looks_like_machine_output(response)
}

fn looks_like_machine_output(response: &str) -> bool {
    let value = response.trim();
    if value.is_empty() {
        return false;
    }
    if (value.starts_with('{') && value.ends_with('}'))
        || (value.starts_with('[') && value.ends_with(']'))
        || value.starts_with("```")
        || value.starts_with("diff --git")
        || value.starts_with("*** Begin Patch")
        || value.starts_with("TAP version")
        || value.starts_with("PASS")
        || value.starts_with("FAIL")
        || value.starts_with("ok ")
        || value.starts_with("not ok ")
    {
        return true;
    }
    value.lines().any(|line| {
        line.starts_with("@@ -") || line.starts_with("error:") || line.starts_with("warn:")
    })
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut sentence = String::new();
    for ch in text.chars() {
        sentence.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            items.push(sentence.trim().to_string());
            sentence.clear();
        }
    }
    if !sentence.trim().is_empty() {
        items.push(sentence.trim().to_string());
    }
    if items.is_empty() {
        vec![text.to_string()]
    } else {
        items
    }
}

fn layer_rows(summary: &str, profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec!["TLDR".to_string(), compact(summary, 1)],
        vec!["핵심".to_string(), summary.to_string()],
    ];

    let architecture = match profile.architecture_depth.as_str() {
        "overview" => "CLI → Policy → Renderer 흐름만 빠르게 봅니다.",
        "internals" => {
            "CLI wrapper → Rust selector → table/flow/formula primitives → ANSI/theme 출력까지 봅니다."
        }
        _ => "CLI → Policy → Evolution → Shaper → Renderer 순서로 책임을 나눕니다.",
    };
    rows.push(vec!["아키텍처".to_string(), architecture.to_string()]);

    if profile.architecture_depth != "overview" {
        rows.push(vec![
            "선택기".to_string(),
            "명시적 룰, 점수 기반 UX selector, 선택적 planner hint를 조합합니다.".to_string(),
        ]);
    }

    if profile.architecture_depth == "internals" || profile.explanation_depth == "deep" {
        rows.push(vec![
            "구현".to_string(),
            "프로필, 환경변수, prompt/response 신호를 Rust core에서 안전하게 합칩니다.".to_string(),
        ]);
    }

    let abstraction = match profile.abstraction_level.as_str() {
        "concrete" => "명령, 파일, 테스트처럼 바로 실행 가능한 수준으로 설명합니다.",
        "strategy" => "왜 이 구조가 제품/사용자 경험에 유리한지 상위 의사결정으로 설명합니다.",
        _ => "컴포넌트 책임과 데이터 흐름을 중심으로 설명합니다.",
    };
    rows.push(vec!["추상화".to_string(), abstraction.to_string()]);

    if profile.explanation_depth != "light" {
        rows.push(vec![
            "다음 행동".to_string(),
            "필요하면 explanation-depth, architecture-depth, abstraction-level을 3단계로 조절합니다."
                .to_string(),
        ]);
    }

    rows
}

fn project_path(relative: &str) -> PathBuf {
    env::var("CODEXPLAIN_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(relative)
}

fn profile_path() -> PathBuf {
    project_path(".codexplain/ux-profile.json")
}

fn config_path() -> PathBuf {
    project_path(".codexplain/config.json")
}

fn load_profile() -> Profile {
    let mut profile = Profile::default();
    if let Ok(raw) = fs::read_to_string(profile_path()) {
        profile.theme = Theme::parse(extract_json_string(&raw, "theme").as_deref());
        profile.frame = Frame::parse(extract_json_string(&raw, "frame").as_deref());
        profile.index_style = IndexStyle::parse(extract_json_string(&raw, "indexStyle").as_deref());
        if let Some(detail) = extract_json_string(&raw, "detail") {
            profile.detail = detail;
        }
        if let Some(style) = extract_json_string(&raw, "style") {
            profile.style = style;
        }
        if let Some(audience) = extract_json_string(&raw, "audience") {
            profile.audience = audience;
        }
        if let Some(preferred_structure) = extract_json_string(&raw, "preferredStructure") {
            profile.preferred_structure = preferred_structure;
        }
        if let Some(min) = extract_json_string_after(&raw, "abstractionRange", "min") {
            profile.abstraction_min = min;
        }
        if let Some(max) = extract_json_string_after(&raw, "abstractionRange", "max") {
            profile.abstraction_max = max;
        }
        if let Some(layers) = extract_json_array_strings(&raw, "detailLayers") {
            profile.layers = layers;
        }
        if let Some(value) = extract_json_string(&raw, "explanationDepth") {
            profile.explanation_depth =
                normalize_explanation_depth(&value, &profile.explanation_depth);
        }
        if let Some(value) = extract_json_string(&raw, "architectureDepth") {
            profile.architecture_depth =
                normalize_architecture_depth(&value, &profile.architecture_depth);
        }
        if let Some(value) = extract_json_string(&raw, "abstractionLevel") {
            profile.abstraction_level =
                normalize_abstraction_level(&value, &profile.abstraction_level);
        }
        if let Some(detail_scale) = extract_json_u8(&raw, "detailScale") {
            profile.detail_scale = detail_scale;
        }
        if let Some(ux_density) = extract_json_u8(&raw, "uxDensity") {
            profile.ux_density = ux_density;
        }
        if let Some(risk_sensitivity) = extract_json_u8(&raw, "riskSensitivity") {
            profile.risk_sensitivity = risk_sensitivity;
        }
    }
    if let Ok(theme) = env::var("CODEXPLAIN_THEME") {
        profile.theme = Theme::parse(Some(&theme));
    }
    if let Ok(frame) = env::var("CODEXPLAIN_FRAME").or_else(|_| env::var("CLAUDEX_FRAME")) {
        profile.frame = Frame::parse(Some(&frame));
    }
    if let Ok(index_style) =
        env::var("CODEXPLAIN_INDEX_STYLE").or_else(|_| env::var("CLAUDEX_INDEX_STYLE"))
    {
        profile.index_style = IndexStyle::parse(Some(&index_style));
    }
    if let Ok(value) = env::var("CODEXPLAIN_EXPLANATION_DEPTH") {
        profile.explanation_depth = normalize_explanation_depth(&value, &profile.explanation_depth);
    }
    if let Ok(value) = env::var("CODEXPLAIN_ARCHITECTURE_DEPTH") {
        profile.architecture_depth =
            normalize_architecture_depth(&value, &profile.architecture_depth);
    }
    if let Ok(value) = env::var("CODEXPLAIN_ABSTRACTION_LEVEL") {
        profile.abstraction_level = normalize_abstraction_level(&value, &profile.abstraction_level);
    }
    if let Ok(value) = env::var("CODEXPLAIN_DETAIL_SCALE") {
        if let Some(parsed) = parse_control_value(&value) {
            profile.detail_scale = parsed;
        }
    }
    if let Ok(value) = env::var("CODEXPLAIN_UX_DENSITY") {
        if let Some(parsed) = parse_control_value(&value) {
            profile.ux_density = parsed;
        }
    }
    if let Ok(value) = env::var("CODEXPLAIN_RISK_SENSITIVITY") {
        if let Some(parsed) = parse_control_value(&value) {
            profile.risk_sensitivity = parsed;
        }
    }
    profile
}

fn load_profile_for_args(args: &[String]) -> Profile {
    let mut profile = load_profile();
    if let Some(theme) = arg_value(args, "--theme") {
        profile.theme = Theme::parse(Some(theme));
    }
    if let Some(frame) = arg_value(args, "--frame") {
        profile.frame = Frame::parse(Some(frame));
    }
    if let Some(index_style) = arg_value(args, "--index-style") {
        profile.index_style = IndexStyle::parse(Some(index_style));
    }
    if let Some(detail) = arg_value(args, "--detail") {
        profile.detail = detail.to_string();
    }
    if let Some(layers) = arg_value(args, "--layers") {
        profile.layers = parse_layers(layers);
    }
    if let Some(value) = arg_value(args, "--explanation-depth") {
        profile.explanation_depth = normalize_explanation_depth(value, &profile.explanation_depth);
    }
    if let Some(value) = arg_value(args, "--architecture-depth") {
        profile.architecture_depth =
            normalize_architecture_depth(value, &profile.architecture_depth);
    }
    if let Some(value) = arg_value(args, "--abstraction-level") {
        profile.abstraction_level = normalize_abstraction_level(value, &profile.abstraction_level);
    }
    if let Some(value) = arg_value(args, "--detail-scale").and_then(parse_control_value) {
        profile.detail_scale = value;
    }
    if let Some(value) = arg_value(args, "--ux-density").and_then(parse_control_value) {
        profile.ux_density = value;
    }
    if let Some(value) = arg_value(args, "--risk-sensitivity").and_then(parse_control_value) {
        profile.risk_sensitivity = value;
    }
    profile.theme = profile
        .theme
        .apply_terminal_policy(|key| env::var(key).ok());
    profile
}

fn save_profile(profile: &Profile) -> io::Result<()> {
    fs::create_dir_all(project_path(".codexplain"))?;
    fs::write(
        profile_path(),
        format!(
            concat!(
                "{{\n",
                "  \"schemaVersion\": 1,\n",
                "  \"detail\": \"{}\",\n",
                "  \"style\": \"{}\",\n",
                "  \"theme\": \"{}\",\n",
                "  \"frame\": \"{}\",\n",
                "  \"indexStyle\": \"{}\",\n",
                "  \"audience\": \"{}\",\n",
                "  \"preferredStructure\": \"{}\",\n",
                "  \"abstractionRange\": {{\n",
                "    \"min\": \"{}\",\n",
                "    \"max\": \"{}\"\n",
                "  }},\n",
                "  \"detailLayers\": [\"{}\"],\n",
                "  \"explanationDepth\": \"{}\",\n",
                "  \"architectureDepth\": \"{}\",\n",
                "  \"abstractionLevel\": \"{}\",\n",
                "  \"detailScale\": {},\n",
                "  \"uxDensity\": {},\n",
                "  \"riskSensitivity\": {},\n",
                "  \"explanationMoves\": [\"tldr\", \"answer-first\", \"plain-language\", \"evidence\", \"next-step\"],\n",
                "  \"feedback\": {{\"positive\": 0, \"negative\": 0, \"revisions\": 0, \"rewardScore\": 0, \"signals\": []}}\n",
                "}}\n"
            ),
            profile.detail,
            profile.style,
            profile.theme.name(),
            if profile.frame == Frame::Ascii { "ascii" } else { "unicode" },
            profile.index_style.name(),
            profile.audience,
            profile.preferred_structure,
            profile.abstraction_min,
            profile.abstraction_max,
            profile.layers.join("\", \""),
            profile.explanation_depth,
            profile.architecture_depth,
            profile.abstraction_level,
            profile.detail_scale,
            profile.ux_density,
            profile.risk_sensitivity
        ),
    )
}

fn extract_json_string(raw: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let index = raw.find(&needle)?;
    let rest = &raw[index + needle.len()..];
    let colon = rest.find(':')?;
    let after_colon = &rest[colon + 1..];
    let first_quote = after_colon.find('"')?;
    let after_first = &after_colon[first_quote + 1..];
    let mut escaped = false;
    let mut end = None;
    for (index, ch) in after_first.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end = Some(index);
            break;
        }
    }
    Some(unescape_json_string(&after_first[..end?]))
}

fn extract_json_string_after(raw: &str, object_key: &str, key: &str) -> Option<String> {
    let index = raw.find(&format!("\"{object_key}\""))?;
    extract_json_string(&raw[index..], key)
}

fn extract_json_u8(raw: &str, key: &str) -> Option<u8> {
    let needle = format!("\"{key}\"");
    let index = raw.find(&needle)?;
    let rest = &raw[index + needle.len()..];
    let colon = rest.find(':')?;
    let after_colon = rest[colon + 1..].trim_start();
    let digits = after_colon
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '-')
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i32>().ok().map(clamp_control)
    }
}

fn extract_json_u64_after(raw: &str, object_key: &str, key: &str) -> Option<u64> {
    let index = raw.find(&format!("\"{object_key}\""))?;
    let rest = &raw[index..];
    let needle = format!("\"{key}\"");
    let key_index = rest.find(&needle)?;
    let after_key = &rest[key_index + needle.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim_start();
    let digits = after_colon
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn extract_json_array_strings(raw: &str, key: &str) -> Option<Vec<String>> {
    let needle = format!("\"{key}\"");
    let index = raw.find(&needle)?;
    let rest = &raw[index + needle.len()..];
    let start = rest.find('[')?;
    let after_start = &rest[start + 1..];
    let end = after_start.find(']')?;
    Some(parse_layers(&after_start[..end].replace('"', "")))
}

fn parse_layers(value: &str) -> Vec<String> {
    let layers = value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(unescape_json_string)
        .collect::<Vec<_>>();
    if layers.is_empty() {
        Profile::default().layers
    } else {
        layers
    }
}

fn unescape_json_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000c}'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|item| item == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn read_stdin_if_needed() -> String {
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    input
}

const STORAGE_DIRS: [&str; 3] = ["target", "dist", "node_modules"];
const DEFAULT_STORAGE_MIN_FREE_AMOUNT: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StorageUnit {
    Gb,
}

impl StorageUnit {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "gb" | "gib" => Some(Self::Gb),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Gb => "gb",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FreeSpaceThreshold {
    amount: u64,
    unit: StorageUnit,
}

impl Default for FreeSpaceThreshold {
    fn default() -> Self {
        Self {
            amount: DEFAULT_STORAGE_MIN_FREE_AMOUNT,
            unit: StorageUnit::Gb,
        }
    }
}

impl FreeSpaceThreshold {
    fn gb(self) -> f64 {
        match self.unit {
            StorageUnit::Gb => self.amount as f64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct StorageCheckConfig {
    min_free: FreeSpaceThreshold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StorageCheckOptions {
    min_free: FreeSpaceThreshold,
    clean: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DirectoryMeasurement {
    name: &'static str,
    bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StorageMeasurement {
    free_kib: Option<u64>,
    directories: Vec<DirectoryMeasurement>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetCleanup {
    Removed,
    AlreadyAbsent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StorageReportDetail {
    SuggestedCleanup,
    Cleaned(TargetCleanup),
    CleanError(String),
}

fn dir_size(path: &Path) -> u64 {
    fn walk(path: &Path) -> u64 {
        let Ok(meta) = fs::symlink_metadata(path) else {
            return 0;
        };
        if meta.is_file() {
            return meta.len();
        }
        if meta.file_type().is_symlink() {
            return fs::metadata(path)
                .ok()
                .filter(|target| target.is_file())
                .map(|target| target.len())
                .unwrap_or(0);
        }
        if !meta.is_dir() {
            return 0;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };
        entries.flatten().map(|entry| walk(&entry.path())).sum()
    }
    walk(path)
}

fn measure_storage(root: &Path) -> StorageMeasurement {
    StorageMeasurement {
        free_kib: available_kib(root),
        directories: STORAGE_DIRS
            .iter()
            .map(|name| DirectoryMeasurement {
                name: *name,
                bytes: dir_size(&root.join(*name)),
            })
            .collect(),
    }
}

fn available_kib(root: &Path) -> Option<u64> {
    let output = Command::new("df").arg("-k").arg(root).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().nth(1)?;
    line.split_whitespace().nth(3)?.parse().ok()
}

fn parse_storage_check_config(raw: &str) -> StorageCheckConfig {
    let default = StorageCheckConfig::default();
    if let Some(amount) = extract_json_u64_after(raw, "storageCheck", "minFreeGb") {
        return StorageCheckConfig {
            min_free: FreeSpaceThreshold {
                amount,
                unit: StorageUnit::Gb,
            },
        };
    }

    let amount = extract_json_u64_after(raw, "minFree", "value");
    let unit = extract_json_string_after(raw, "minFree", "unit")
        .and_then(|value| StorageUnit::parse(&value));

    match (amount, unit) {
        (Some(amount), Some(unit)) => StorageCheckConfig {
            min_free: FreeSpaceThreshold { amount, unit },
        },
        _ => default,
    }
}

fn load_storage_check_config() -> StorageCheckConfig {
    fs::read_to_string(config_path())
        .ok()
        .map(|raw| parse_storage_check_config(&raw))
        .unwrap_or_default()
}

fn resolve_storage_threshold(args: &[String], config: StorageCheckConfig) -> FreeSpaceThreshold {
    arg_value(args, "--min-free-gb")
        .and_then(|value| value.parse::<u64>().ok())
        .map(|amount| FreeSpaceThreshold {
            amount,
            unit: StorageUnit::Gb,
        })
        .unwrap_or(config.min_free)
}

fn resolve_storage_check_options(
    args: &[String],
    config: StorageCheckConfig,
) -> StorageCheckOptions {
    StorageCheckOptions {
        min_free: resolve_storage_threshold(args, config),
        clean: args.iter().any(|arg| arg == "--clean"),
    }
}

fn storage_result_message(
    available_gb: f64,
    threshold: FreeSpaceThreshold,
) -> (&'static str, String) {
    let effective_min_gb = threshold.gb();
    if available_gb < effective_min_gb {
        (
            "fail",
            format!(
                "fail: free_gb {available_gb:.2} is below effective_min_free_{} {}",
                threshold.unit.name(),
                threshold.amount
            ),
        )
    } else {
        (
            "pass",
            format!(
                "pass: free_gb {available_gb:.2} meets effective_min_free_{} {}",
                threshold.unit.name(),
                threshold.amount
            ),
        )
    }
}

fn format_storage_report(
    measurement: &StorageMeasurement,
    threshold: FreeSpaceThreshold,
    detail: Option<StorageReportDetail>,
) -> String {
    let available = measurement.free_kib.unwrap_or(0) as f64 / 1024.0 / 1024.0;
    let low_space = available < threshold.gb();
    let (result, message) = storage_result_message(available, threshold);
    let mut lines = vec![
        "contract=codexplain.storage-check.v1".to_string(),
        format!("free_gb={available:.2}"),
        format!("min_free_{}={}", threshold.unit.name(), threshold.amount),
        format!(
            "effective_min_free_{}={}",
            threshold.unit.name(),
            threshold.amount
        ),
    ];
    for directory in &measurement.directories {
        let size_mb = directory.bytes as f64 / 1024.0 / 1024.0;
        lines.push(format!("{}_mb={size_mb:.1}", directory.name));
    }
    lines.push(format!("result={result}"));
    lines.push(format!("message={message}"));
    if low_space {
        lines.push("status=low-space".to_string());
        match detail.unwrap_or(StorageReportDetail::SuggestedCleanup) {
            StorageReportDetail::SuggestedCleanup => lines.push("suggested_cleanup=rerun with --clean to remove target/; remove dist if regenerated; prune old node_modules only after confirming dependencies".to_string()),
            StorageReportDetail::Cleaned(TargetCleanup::Removed) => lines.push("cleaned=target".to_string()),
            StorageReportDetail::Cleaned(TargetCleanup::AlreadyAbsent) => {
                lines.push("cleaned=target_already_absent".to_string())
            }
            StorageReportDetail::CleanError(error) => lines.push(format!("clean_error=target:{error}")),
        }
    } else {
        lines.push("status=ok".to_string());
    }
    lines.join("\n")
}

fn storage_check(args: &[String]) {
    let config = load_storage_check_config();
    let options = resolve_storage_check_options(args, config);
    let threshold = options.min_free;
    let root = project_path(".");
    let measurement = measure_storage(&root);
    let available = measurement.free_kib.unwrap_or(0) as f64 / 1024.0 / 1024.0;
    let cleanup_targets = cleanup_targets(available, options);
    let detail = if cleanup_targets.contains(&"target") {
        match cleanup_project_storage_dir(&root, "target") {
            Ok(cleaned) => Some(StorageReportDetail::Cleaned(cleaned)),
            Err(error) => Some(StorageReportDetail::CleanError(error.to_string())),
        }
    } else {
        None
    };
    println!("{}", format_storage_report(&measurement, threshold, detail));
}

fn cleanup_targets(available_gb: f64, options: StorageCheckOptions) -> Vec<&'static str> {
    if options.clean && available_gb < options.min_free.gb() {
        vec!["target"]
    } else {
        Vec::new()
    }
}

fn cleanup_project_storage_dir(root: &Path, directory: &str) -> io::Result<TargetCleanup> {
    if directory != "target" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean any path other than the project target directory",
        ));
    }
    cleanup_project_target(root)
}

fn cleanup_project_target(root: &Path) -> io::Result<TargetCleanup> {
    let root = root.canonicalize()?;
    let target = root.join("target");
    let metadata = match fs::symlink_metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(TargetCleanup::AlreadyAbsent);
        }
        Err(error) => return Err(error),
    };

    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean target because it is a symlink",
        ));
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean target because it is not a directory",
        ));
    }

    let resolved_target = target.canonicalize()?;
    let resolved_parent = resolved_target.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean target because its parent cannot be resolved",
        )
    })?;
    if resolved_target.file_name().and_then(|name| name.to_str()) != Some("target")
        || resolved_parent != root
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean any path other than the project target directory",
        ));
    }

    fs::remove_dir_all(&target)?;
    Ok(TargetCleanup::Removed)
}

fn post_response(args: &[String]) {
    let input = read_stdin_if_needed();
    if input.trim().is_empty() {
        return;
    }
    let fallback_prompt = arg_value(args, "--prompt")
        .map(str::to_string)
        .or_else(|| env::var("CODEXPLAIN_PROMPT").ok())
        .or_else(|| env::var("CLAUDEX_PROMPT").ok())
        .unwrap_or_default();
    let prompt = extract_json_string(&input, "prompt")
        .or_else(|| extract_json_string(&input, "userPrompt"))
        .unwrap_or_else(|| fallback_prompt.clone());
    let response = extract_json_string(&input, "response")
        .or_else(|| extract_json_string(&input, "answer"))
        .or_else(|| extract_json_string(&input, "text"))
        .unwrap_or(input);
    let width = arg_value(args, "--width")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let profile = load_profile_for_args(args);
    print!("{}", shape(&prompt, &response, &profile, width));
}

fn usage() -> &'static str {
    "Usage:
  codexplain shape --prompt <text> [--response <text>] [--width <n>]
  codexplain post-response --prompt <text> [--width <n>]
  codexplain profile --show|--theme <name>|--frame <unicode|ascii|fallback|auto>|--index-style <style>|--detail <level>
  codexplain profile --explanation-depth <light|standard|deep>|--architecture-depth <overview|system|internals>|--abstraction-level <concrete|architecture|strategy>
  codexplain profile --detail-scale <0-100>|--ux-density <0-100>|--risk-sensitivity <0-100>
  codexplain demo
  codexplain storage-check [--min-free-gb 5] [--clean]

Storage-check output contract:
  contract=codexplain.storage-check.v1
  free_gb=<decimal>
  min_free_gb=<integer>
  effective_min_free_gb=<integer>
  target_mb=<decimal>
  dist_mb=<decimal>
  node_modules_mb=<decimal>
  result=pass|fail
  message=<pass/fail detail using effective_min_free_gb>
  status=ok|low-space
  cleaned=target|target_already_absent, clean_error=target:<message>, or suggested_cleanup=<text> may appear only when status=low-space

Themes: none, ocean, forest, warm, sunset, grape, slate, rose, mono
Index styles: decimal, zero-padded, alpha-lower, alpha-upper, roman-lower, roman-upper"
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("demo");
    match command {
        "shape" => {
            let profile = load_profile_for_args(&args);
            let prompt = arg_value(&args, "--prompt").unwrap_or("");
            let response = arg_value(&args, "--response")
                .map(str::to_string)
                .unwrap_or_else(read_stdin_if_needed);
            let width = arg_value(&args, "--width")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100);
            println!("{}", shape(prompt, &response, &profile, width));
        }
        "post-response" => post_response(&args),
        "profile" => {
            let mut profile = load_profile();
            if args.iter().any(|arg| arg == "--show") {
                print_profile(&profile);
                return;
            }
            if let Some(theme) = arg_value(&args, "--theme") {
                profile.theme = Theme::parse(Some(theme));
            }
            if let Some(frame) = arg_value(&args, "--frame") {
                profile.frame = Frame::parse(Some(frame));
            }
            if let Some(index_style) = arg_value(&args, "--index-style") {
                profile.index_style = IndexStyle::parse(Some(index_style));
            }
            if let Some(detail) = arg_value(&args, "--detail") {
                profile.detail = detail.to_string();
            }
            if let Some(style) = arg_value(&args, "--set-style") {
                profile.style = style.to_string();
            }
            if let Some(audience) = arg_value(&args, "--audience") {
                profile.audience = audience.to_string();
            }
            if let Some(structure) = arg_value(&args, "--structure") {
                profile.preferred_structure = structure.to_string();
            }
            if let Some(layers) = arg_value(&args, "--layers") {
                profile.layers = parse_layers(layers);
            }
            if let Some(value) = arg_value(&args, "--explanation-depth") {
                profile.explanation_depth =
                    normalize_explanation_depth(value, &profile.explanation_depth);
            }
            if let Some(value) = arg_value(&args, "--architecture-depth") {
                profile.architecture_depth =
                    normalize_architecture_depth(value, &profile.architecture_depth);
            }
            if let Some(value) = arg_value(&args, "--abstraction-level") {
                profile.abstraction_level =
                    normalize_abstraction_level(value, &profile.abstraction_level);
            }
            if let Some(value) = arg_value(&args, "--detail-scale").and_then(parse_control_value) {
                profile.detail_scale = value;
            }
            if let Some(value) = arg_value(&args, "--ux-density").and_then(parse_control_value) {
                profile.ux_density = value;
            }
            if let Some(value) =
                arg_value(&args, "--risk-sensitivity").and_then(parse_control_value)
            {
                profile.risk_sensitivity = value;
            }
            if let Some(range) = arg_value(&args, "--abstraction-range") {
                let parts: Vec<&str> = range.split(':').collect();
                if let Some(min) = parts.first() {
                    profile.abstraction_min = (*min).to_string();
                }
                if let Some(max) = parts.get(1) {
                    profile.abstraction_max = (*max).to_string();
                }
            }
            if let Some(max) = arg_value(&args, "--abstraction") {
                profile.abstraction_max = max.to_string();
            }
            if let Err(error) = save_profile(&profile) {
                eprintln!("failed to save profile: {error}");
                std::process::exit(1);
            }
            print_profile(&profile);
        }
        "storage-check" => storage_check(&args),
        "pros-cons" => println!("{}", pros_cons(&load_profile_for_args(&args))),
        "formula" => println!(
            "{}",
            formula(
                &load_profile_for_args(&args),
                "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다."
            )
        ),
        "demo" => {
            let profile = load_profile_for_args(&args);
            println!(
                "{}\n\n{}",
                pros_cons(&profile),
                formula(
                    &profile,
                    "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다."
                )
            );
        }
        "--help" | "-h" => println!("{}", usage()),
        other => {
            eprintln!("Unknown command: {other}\n\n{}", usage());
            std::process::exit(2);
        }
    }
}

fn print_profile(profile: &Profile) {
    println!(
        concat!(
            "{{\n",
            "  \"theme\": \"{}\",\n",
            "  \"frame\": \"{}\",\n",
            "  \"indexStyle\": \"{}\",\n",
            "  \"detail\": \"{}\",\n",
            "  \"style\": \"{}\",\n",
            "  \"audience\": \"{}\",\n",
            "  \"preferredStructure\": \"{}\",\n",
            "  \"abstractionRange\": {{\"min\": \"{}\", \"max\": \"{}\"}},\n",
            "  \"detailLayers\": [\"{}\"],\n",
            "  \"explanationDepth\": \"{}\",\n",
            "  \"architectureDepth\": \"{}\",\n",
            "  \"abstractionLevel\": \"{}\",\n",
            "  \"detailScale\": {},\n",
            "  \"uxDensity\": {},\n",
            "  \"riskSensitivity\": {}\n",
            "}}"
        ),
        profile.theme.name(),
        if profile.frame == Frame::Ascii {
            "ascii"
        } else {
            "unicode"
        },
        profile.index_style.name(),
        profile.detail,
        profile.style,
        profile.audience,
        profile.preferred_structure,
        profile.abstraction_min,
        profile.abstraction_max,
        profile.layers.join("\", \""),
        profile.explanation_depth,
        profile.architecture_depth,
        profile.abstraction_level,
        profile.detail_scale,
        profile.ux_density,
        profile.risk_sensitivity
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visible_line_widths(output: &str) -> Vec<usize> {
        output.lines().map(visible_width).collect()
    }

    fn assert_visible_lines_fit(output: &str, max_width: usize) {
        let too_wide = visible_line_widths(output)
            .into_iter()
            .filter(|width| *width > max_width)
            .collect::<Vec<_>>();
        assert!(
            too_wide.is_empty(),
            "line widths exceeded {max_width}: {too_wide:?}\n{output}"
        );
    }

    #[test]
    fn renders_unicode_pros_cons() {
        let output = pros_cons(&Profile::default());
        assert!(output.contains('┌'));
        assert!(output.contains("JS / Node"));
        assert!(output.contains("Rust"));
    }

    #[test]
    fn pros_cons_table_uses_shared_table_primitives_with_labeled_columns() {
        let model = pros_cons_table(120);
        let layout = model.layout(Frame::Unicode);
        let output = render_table_model(&model, Frame::Unicode, Theme::None);

        assert_eq!(model.column_count(), 4);
        assert_eq!(
            model
                .headers
                .cells
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>(),
            vec!["선택지", "장점", "단점", "적합한 때"]
        );
        assert_eq!(
            layout.border(FrameRule::Top, Theme::None),
            "┌───────────┬───────────────────────────────────┬──────────────────────────────────┬───────────────────────┐"
        );
        assert!(output.contains("│ 선택지"));
        assert!(output.contains("│ 장점"));
        assert!(output.contains("│ 단점"));
        assert!(output.contains("provider 연동"));
        assert!(output.contains("초기 구현 비용"));
        assert!(output.contains('├'));
    }

    #[test]
    fn pros_cons_snapshot_uses_row_divided_table_output() {
        let output = pros_cons(&Profile {
            theme: Theme::None,
            ..Profile::default()
        });

        assert_eq!(
            output,
            [
                "┌───────────┬───────────────────────────────────┬──────────────────────────────────┬───────────────────────┐",
                "│ 선택지    │ 장점                              │ 단점                             │ 적합한 때             │",
                "├───────────┼───────────────────────────────────┼──────────────────────────────────┼───────────────────────┤",
                "│ JS / Node │ 빠른 수정                         │ 런타임 의존성                    │ UX 실험과 피드백 루프 │",
                "│           │ provider 연동                     │ 단일 바이너리 약함               │                       │",
                "│           │ JSON 처리                         │                                  │                       │",
                "├───────────┼───────────────────────────────────┼──────────────────────────────────┼───────────────────────┤",
                "│ Rust      │ 단일 바이너리                     │ 초기 구현 비용                   │ 안정화된 CLI core     │",
                "│           │ 빠른 시작                         │ provider 실험 비용               │                       │",
                "│           │ 낮은 메모리                       │                                  │                       │",
                "└───────────┴───────────────────────────────────┴──────────────────────────────────┴───────────────────────┘",
            ]
            .join("\n")
        );
    }

    #[test]
    fn renders_ascii_formula() {
        let profile = Profile {
            frame: Frame::Ascii,
            theme: Theme::None,
            ..Profile::default()
        };
        let output = formula(&profile, "테스트");
        assert!(output.contains('+'));
        assert!(output.contains("설명 품질 = f"));
    }

    #[test]
    fn formula_box_snapshot_uses_specialized_label_value_layout() {
        let output = formula(
            &Profile {
                theme: Theme::None,
                ..Profile::default()
            },
            "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다.",
        );

        assert_eq!(
            output,
            [
                "┌─────────────────────────────────────────────────────────────────────┐",
                "│ 수식 박스                                                           │",
                "├─────────────────────────────────────────────────────────────────────┤",
                "│ 핵심식 : 설명 품질 = f(명확성, 구조, 근거, 다음 행동)               │",
                "├─────────────────────────────────────────────────────────────────────┤",
                "│ 의미   : 색은 보조 신호이고 텍스트 label/value가 의미를 보존합니다. │",
                "│ 설명   : 초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니  │",
                "│          다.                                                        │",
                "└─────────────────────────────────────────────────────────────────────┘",
            ]
            .join("\n")
        );
        assert!(output.contains("핵심식 :"));
        assert!(output.contains("의미   :"));
        assert!(!output.contains("구분"));
        assert!(!output.contains("수식/의미"));
    }

    #[test]
    fn narrow_formula_box_wraps_and_fits_visible_width() {
        let model = FormulaBox::new(
            "Decision Rule",
            [
                FormulaField::new("Rule", "choice = f(iteration, distribution, safety)"),
                FormulaField::new(
                    "Risk",
                    "Keep color supplemental and preserve visible labels.",
                ),
            ],
            44,
        );
        let output = render_formula_box(&model, Frame::Unicode, Theme::None);

        assert_visible_lines_fit(&output, 44);
        assert!(output.contains("│ Rule : choice = f(iteration"));
        assert!(output.contains("│        n, safety)                        │"));
        assert!(output.contains("│ Risk : Keep color supplemental"));
        assert!(!output.contains("----"));
        assert!(!output.contains("===="));
    }

    #[test]
    fn ascii_formula_box_uses_terminal_safe_fallback_glyphs() {
        let model = FormulaBox::new(
            "Formula",
            [FormulaField::new("Core", "quality = f(clear, grounded)")],
            42,
        );
        let output = render_formula_box(&model, Frame::Ascii, Theme::None);

        assert_visible_lines_fit(&output, 42);
        assert!(output.starts_with('+'));
        assert!(output.contains("| Core : quality = f(clear, grounded) |"));
        assert!(!output.contains(['┌', '┬', '┐', '│', '├', '┼', '┤', '└', '┴', '┘']));
    }

    #[test]
    fn ansi_colored_and_plain_formula_boxes_have_same_visible_widths() {
        let model = FormulaBox::new(
            "Formula",
            [
                FormulaField::new("TLDR", "choice = f(signal, evidence)"),
                FormulaField::new("위험", "ANSI color is supplemental, not semantic-only."),
            ],
            72,
        );
        let plain = render_formula_box(&model, Frame::Unicode, Theme::None);
        let colored = render_formula_box(&model, Frame::Unicode, Theme::Ocean);

        assert_eq!(visible_line_widths(&colored), visible_line_widths(&plain));
        assert!(colored.contains("\x1b[36m┌"));
        assert!(colored.contains("\x1b[1;32m TLDR"));
        assert!(colored.contains("\x1b[1;33m 위험"));
        assert!(colored.contains("choice = f(signal, evidence)"));
        assert!(colored.contains("ANSI color is supplemental"));
    }

    #[test]
    fn shapes_indexed_list() {
        let output = shape(
            "1,2,3 번호로 나눠서 설명",
            "첫째입니다. 둘째입니다.",
            &Profile {
                theme: Theme::None,
                ..Profile::default()
            },
            80,
        );
        assert!(output.contains("1. │"));
        assert!(output.contains("2. │"));
    }

    #[test]
    fn indexed_list_snapshot_renders_multiple_items() {
        let output = indexed(
            &[
                "첫 번째 설명".to_string(),
                "두 번째 설명".to_string(),
                "세 번째 설명".to_string(),
            ],
            Frame::Unicode,
            Theme::None,
            60,
            IndexStyle::Decimal,
        );

        assert_eq!(
            output,
            [
                "1. │ 첫 번째 설명",
                "2. │ 두 번째 설명",
                "3. │ 세 번째 설명",
            ]
            .join("\n")
        );
    }

    #[test]
    fn indexed_list_wraps_continuation_lines_under_content_column() {
        let output = indexed(
            &["Alpha beta gamma delta epsilon".to_string()],
            Frame::Unicode,
            Theme::None,
            18,
            IndexStyle::Decimal,
        );
        let lines = output.lines().collect::<Vec<_>>();

        assert!(lines.len() > 1, "{output}");
        assert!(lines[0].starts_with("1. │ "));
        assert!(lines[1].starts_with("   │ "));
        assert_visible_lines_fit(&output, 18);
    }

    #[test]
    fn indexed_list_supports_numbering_styles() {
        let zero_padded = indexed(
            &(1..=12)
                .map(|number| format!("item{number}"))
                .collect::<Vec<_>>(),
            Frame::Unicode,
            Theme::None,
            40,
            IndexStyle::ZeroPadded,
        );
        let alpha = indexed(
            &(1..=27)
                .map(|number| format!("item{number}"))
                .collect::<Vec<_>>(),
            Frame::Unicode,
            Theme::None,
            40,
            IndexStyle::AlphaUpper,
        );
        let roman = indexed(
            &[
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
            ],
            Frame::Unicode,
            Theme::None,
            40,
            IndexStyle::RomanLower,
        );

        assert!(zero_padded.contains("01. │ item1"));
        assert!(zero_padded.contains("12. │ item12"));
        assert!(alpha.contains(" A. │ item1"));
        assert!(alpha.contains(" Z. │ item26"));
        assert!(alpha.contains("AA. │ item27"));
        assert!(roman.contains("  i. │ one"));
        assert!(roman.contains(" iv. │ four"));
    }

    #[test]
    fn indexed_list_uses_ascii_gutter_when_requested() {
        let output = indexed(
            &["first".to_string(), "second".to_string()],
            Frame::Ascii,
            Theme::None,
            40,
            IndexStyle::Decimal,
        );

        assert!(output.contains("1. | first"));
        assert!(output.contains("2. | second"));
        assert!(!output.contains('│'));
    }

    #[test]
    fn index_style_parse_accepts_aliases() {
        assert_eq!(IndexStyle::parse(Some("decimal")), IndexStyle::Decimal);
        assert_eq!(IndexStyle::parse(Some("01")), IndexStyle::ZeroPadded);
        assert_eq!(
            IndexStyle::parse(Some("upper-alpha")),
            IndexStyle::AlphaUpper
        );
        assert_eq!(IndexStyle::parse(Some("A")), IndexStyle::AlphaUpper);
        assert_eq!(IndexStyle::parse(Some("roman")), IndexStyle::RomanLower);
        assert_eq!(
            IndexStyle::parse(Some("upper-roman")),
            IndexStyle::RomanUpper
        );
        assert_eq!(IndexStyle::parse(Some("I")), IndexStyle::RomanUpper);
    }

    #[test]
    fn ansi_theme_primitives_apply_semantic_styles() {
        assert_eq!(
            Theme::Ocean.style(AnsiRole::Border).apply("│"),
            "\x1b[36m│\x1b[0m"
        );
        assert_eq!(
            Theme::Sunset.style(AnsiRole::Danger).apply("위험"),
            "\x1b[1;38;5;196m위험\x1b[0m"
        );
        assert_eq!(Theme::None.style(AnsiRole::Heading).apply("핵심"), "핵심");
        assert_eq!(AnsiRole::parse("unknown"), AnsiRole::Accent);
    }

    #[test]
    fn theme_plain_aliases_disable_ansi_without_losing_text() {
        for value in ["none", "plain", "no-color", "no_color", "off", "false", "0"] {
            assert_eq!(Theme::parse(Some(value)), Theme::None);
            assert_eq!(color(Theme::parse(Some(value)), "success", "장점"), "장점");
        }
    }

    #[test]
    fn no_color_environment_forces_plain_theme() {
        assert_eq!(
            Theme::Ocean.apply_terminal_policy(|key| match key {
                "NO_COLOR" => Some("1".to_string()),
                _ => None,
            }),
            Theme::None
        );
        assert_eq!(
            Theme::Forest.apply_terminal_policy(|key| match key {
                "CODEXPLAIN_NO_COLOR" => Some("true".to_string()),
                _ => None,
            }),
            Theme::None
        );
        assert_eq!(
            Theme::Warm.apply_terminal_policy(|key| match key {
                "TERM" => Some("dumb".to_string()),
                _ => None,
            }),
            Theme::None
        );
    }

    #[test]
    fn ansi_fallback_preserves_rendered_structure() {
        let profile = Profile {
            theme: Theme::Ocean.apply_terminal_policy(|key| match key {
                "NO_COLOR" => Some("1".to_string()),
                _ => None,
            }),
            ..Profile::default()
        };
        let output = table(
            &["구분", "내용"],
            &[
                vec![
                    "장점".to_string(),
                    "색 없이도 의미가 텍스트로 남습니다.".to_string(),
                ],
                vec![
                    "위험".to_string(),
                    "ANSI는 보조 표현일 뿐입니다.".to_string(),
                ],
            ],
            profile.frame,
            profile.theme,
            true,
            80,
        );

        assert!(output.contains("장점"));
        assert!(output.contains("위험"));
        assert!(output.contains('┌'));
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn frame_spec_exposes_reusable_primitives() {
        let spec = Frame::Unicode.preset(FramePreset::Table);
        assert_eq!(spec.border.horizontal, '─');
        assert_eq!(spec.border.vertical, '│');
        assert_eq!(spec.corners.top_left, '┌');
        assert_eq!(spec.separators.middle_join, '┼');
        assert_eq!(spec.padding.total(), 2);
        assert_eq!(spec.rule(FrameRule::Top, &[3, 2]), "┌─────┬────┐");
        assert_eq!(spec.padded_cell("x", 3), " x   ");
    }

    #[test]
    fn unicode_frame_presets_use_connected_box_drawing() {
        let table_spec = Frame::Unicode.preset(FramePreset::Table);
        let flow_spec = Frame::Unicode.preset(FramePreset::Flow);
        let indexed_spec = Frame::Unicode.preset(FramePreset::Indexed);

        assert_eq!(table_spec.rule(FrameRule::Top, &[4, 4]), "┌──────┬──────┐");
        assert_eq!(
            table_spec.rule(FrameRule::Middle, &[4, 4]),
            "├──────┼──────┤"
        );
        assert_eq!(
            table_spec.rule(FrameRule::Bottom, &[4, 4]),
            "└──────┴──────┘"
        );
        assert_eq!(flow_spec.separators.up_join, '┴');
        assert_eq!(flow_spec.separators.down_join, '┬');
        assert_eq!(flow_spec.separators.arrow_down, '▼');
        assert_eq!(indexed_spec.border.vertical, '│');
    }

    #[test]
    fn unicode_renderers_use_presets_without_pseudo_borders() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let table_output = table(
            &["구분", "내용"],
            &[vec!["핵심".to_string(), "연결형 프레임".to_string()]],
            profile.frame,
            profile.theme,
            true,
            60,
        );
        let flow_output = render_flow_diagram(
            &FlowDiagram::new([FlowStep::new("입력"), FlowStep::new("렌더링")], 80),
            profile.frame,
            profile.theme,
        );
        let indexed_output = indexed(
            &["첫 번째 설명".to_string(), "두 번째 설명".to_string()],
            profile.frame,
            profile.theme,
            60,
            IndexStyle::Decimal,
        );
        let combined = format!("{table_output}\n{flow_output}\n{indexed_output}");

        for glyph in ['┌', '┬', '┐', '│', '├', '┼', '┤', '└', '┴', '┘'] {
            assert!(combined.contains(glyph), "missing connected glyph {glyph}");
        }
        assert!(combined.contains("1. │"));
        assert!(!combined.contains("----"));
        assert!(!combined.contains("===="));
        assert!(!combined.contains("ㅡㅡㅡㅡ"));
    }

    #[test]
    fn flow_renderer_snapshot_uses_step_boxes_and_sequence_connectors() {
        let output = render_flow_diagram(
            &FlowDiagram::new(
                [
                    FlowStep::new("입력"),
                    FlowStep::new("정책 검사"),
                    FlowStep::new("출력"),
                ],
                80,
            ),
            Frame::Unicode,
            Theme::None,
        );

        assert_eq!(
            output,
            [
                "┌───────────┐",
                "│ 입력      │",
                "└─────┬─────┘",
                "      │",
                "      ▼",
                "┌─────┴─────┐",
                "│ 정책 검사 │",
                "└─────┬─────┘",
                "      │",
                "      ▼",
                "┌─────┴─────┐",
                "│ 출력      │",
                "└───────────┘",
            ]
            .join("\n")
        );
        assert!(!output.contains("----"));
        assert!(!output.contains("===="));
    }

    #[test]
    fn flow_renderer_formats_branching_decisions() {
        let diagram = FlowDiagram::new(
            [
                FlowStep::new("Input"),
                FlowStep::with_branches("Policy", ["JSON safe".to_string(), "Explain".to_string()]),
                FlowStep::new("Render"),
            ],
            60,
        );
        let output = render_flow_diagram(&diagram, Frame::Unicode, Theme::None);

        assert_eq!(
            output,
            [
                "┌───────────┐",
                "│ Input     │",
                "└─────┬─────┘",
                "      │",
                "      ▼",
                "┌─────┴─────┐",
                "│ Policy    │",
                "└─────┬─────┘",
                "      │",
                "      ▼",
                "      ├─▶ JSON safe",
                "      └─▶ Explain",
                "      │",
                "      ▼",
                "┌─────┴─────┐",
                "│ Render    │",
                "└───────────┘",
            ]
            .join("\n")
        );
        assert_visible_lines_fit(&output, 60);
        assert!(output.contains("JSON safe"));
        assert!(output.contains("Explain"));
    }

    #[test]
    fn ascii_flow_renderer_keeps_branching_terminal_safe() {
        let diagram = FlowDiagram::new(
            [
                FlowStep::with_branches(
                    "Route",
                    ["strict artifact".to_string(), "readable answer".to_string()],
                ),
                FlowStep::new("Done"),
            ],
            48,
        );
        let output = render_flow_diagram(&diagram, Frame::Ascii, Theme::None);

        assert!(output.contains("+-> strict artifact"));
        assert!(output.contains("`-> readable answer"));
        assert!(output.contains('v'));
        assert!(!output.contains(['┌', '┬', '┐', '│', '└', '┴', '┘', '▼', '▶']));
        assert_visible_lines_fit(&output, 48);
    }

    #[test]
    fn ansi_colored_and_plain_flows_have_same_visible_widths() {
        let diagram = FlowDiagram::new(
            [
                FlowStep::new("TLDR"),
                FlowStep::with_branches("위험", ["텍스트 의미 보존".to_string()]),
            ],
            72,
        );
        let plain = render_flow_diagram(&diagram, Frame::Unicode, Theme::None);
        let colored = render_flow_diagram(&diagram, Frame::Unicode, Theme::Ocean);

        assert_eq!(visible_line_widths(&colored), visible_line_widths(&plain));
        assert!(colored.contains("\x1b[36m┌"));
        assert!(colored.contains("TLDR"));
        assert!(colored.contains("위험"));
        assert!(colored.contains("텍스트 의미 보존"));
    }

    #[test]
    fn unicode_table_snapshot_uses_connected_frame_glyphs() {
        let output = table(
            &["구분", "내용"],
            &[vec!["핵심".to_string(), "Rust core".to_string()]],
            Frame::Unicode,
            Theme::None,
            false,
            60,
        );

        assert_eq!(
            output,
            [
                "┌──────┬───────────┐",
                "│ 구분 │ 내용      │",
                "├──────┼───────────┤",
                "│ 핵심 │ Rust core │",
                "└──────┴───────────┘",
            ]
            .join("\n")
        );
    }

    #[test]
    fn unicode_table_draws_connected_row_dividers_between_body_rows() {
        let output = table(
            &["계층", "역할"],
            &[
                vec!["CLI".to_string(), "명령 입구".to_string()],
                vec!["Policy".to_string(), "strict 출력 보호".to_string()],
                vec!["Renderer".to_string(), "표와 흐름도 출력".to_string()],
            ],
            Frame::Unicode,
            Theme::None,
            true,
            80,
        );
        let divider = Frame::Unicode
            .preset(FramePreset::Table)
            .rule(FrameRule::RowDivider, &[8, 16]);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(divider, "├──────────┼──────────────────┤");
        assert_eq!(lines.iter().filter(|line| **line == divider).count(), 3);
        assert_eq!(lines[2], divider);
        assert_eq!(lines[4], divider);
        assert_eq!(lines[6], divider);
        assert!(output.contains("│ CLI      │ 명령 입구        │"));
        assert!(output.contains("│ Policy   │ strict 출력 보호 │"));
        assert!(output.contains("│ Renderer │ 표와 흐름도 출력 │"));
    }

    #[test]
    fn narrow_width_table_snapshot_wraps_and_fits_visible_width() {
        let output = table(
            &["구분", "내용"],
            &[vec![
                "다음 행동".to_string(),
                "필요하면 abstraction range와 detail layers를 조절합니다.".to_string(),
            ]],
            Frame::Unicode,
            Theme::None,
            true,
            40,
        );

        assert_visible_lines_fit(&output, 40);
        assert_eq!(
            output,
            [
                "┌───────────┬──────────────────────────┐",
                "│ 구분      │ 내용                     │",
                "├───────────┼──────────────────────────┤",
                "│ 다음 행동 │ 필요하면 abstraction ran │",
                "│           │ ge와 detail layers를 조  │",
                "│           │ 절합니다.                │",
                "└───────────┴──────────────────────────┘",
            ]
            .join("\n")
        );
    }

    #[test]
    fn narrow_pros_cons_table_keeps_dividers_and_terminal_width() {
        let output = render_table_model(&pros_cons_table(50), Frame::Unicode, Theme::None);

        assert_visible_lines_fit(&output, 50);
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with('├') && line.ends_with('┤'))
                .count(),
            2
        );
        assert!(output.contains("선택지"));
        assert!(output.contains("장점"));
        assert!(output.contains("단점"));
        assert!(output.contains("│ 단일 바"));
        assert!(output.contains("provider"));
        assert!(output.contains("실험"));
    }

    #[test]
    fn frame_line_composes_glyphs_repeats_and_text() {
        let output = FrameLine::new()
            .glyph('┌')
            .repeat('─', 2)
            .text("내용")
            .glyph('┐')
            .render();
        assert_eq!(output, "┌──내용┐");
    }

    #[test]
    fn ansi_safe_width_helpers_ignore_escape_sequences() {
        assert_eq!(visible_width("\x1b[1;32m장점\x1b[0m"), 4);
        assert_eq!(visible_width("\x1b[Kclear"), 5);
        assert_eq!(visible_width("e\u{0301}"), 1);
        assert_eq!(visible_width("\t表"), 6);

        let padded = pad("\x1b[31mA\x1b[0m", 3);
        assert_eq!(visible_width(&padded), 3);
        assert!(padded.ends_with("  "));
    }

    #[test]
    fn wrap_text_preserves_ansi_sequences_without_counting_them() {
        assert_eq!(
            wrap_text("\x1b[31mAB\x1b[0mC", 2),
            vec!["\x1b[31mAB\x1b[0m".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn table_model_and_layout_share_width_padding_and_border_rules() {
        let model = Table::new(
            &["\x1b[1m구분\x1b[0m", "내용"],
            &[vec!["핵심".to_string(), "Rust core".to_string()]],
            true,
            60,
        );
        let layout = model.layout(Frame::Unicode);

        assert_eq!(model.column_count(), 2);
        assert_eq!(layout.widths, vec![4, 9]);
        assert_eq!(
            layout.border(FrameRule::Top, Theme::None),
            "┌──────┬───────────┐"
        );
        assert_eq!(
            layout.padded_cell("\x1b[32mA\x1b[0m", 3),
            " \x1b[32mA\x1b[0m   "
        );
    }

    #[test]
    fn ascii_frame_spec_uses_safe_fallback_glyphs() {
        let spec = Frame::Ascii.preset(FramePreset::Table);
        assert_eq!(spec.rule(FrameRule::Bottom, &[1]), "+---+");
        assert_eq!(spec.separators.arrow_down, 'v');
    }

    #[test]
    fn ascii_frame_presets_cover_all_renderers() {
        let table_spec = Frame::Ascii.preset(FramePreset::Table);
        let flow_spec = Frame::Ascii.preset(FramePreset::Flow);
        let indexed_spec = Frame::Ascii.preset(FramePreset::Indexed);

        assert_eq!(table_spec.rule(FrameRule::Top, &[4, 4]), "+------+------+");
        assert_eq!(flow_spec.separators.up_join, '+');
        assert_eq!(flow_spec.separators.down_join, '+');
        assert_eq!(flow_spec.separators.arrow_down, 'v');
        assert_eq!(indexed_spec.border.vertical, '|');
    }

    #[test]
    fn ascii_fallback_table_snapshot_avoids_unicode_frame_glyphs() {
        let output = table(
            &["Part", "Meaning"],
            &[vec!["Core".to_string(), "Rust".to_string()]],
            Frame::Ascii,
            Theme::None,
            false,
            60,
        );

        assert_eq!(
            output,
            [
                "+------+---------+",
                "| Part | Meaning |",
                "+------+---------+",
                "| Core | Rust    |",
                "+------+---------+",
            ]
            .join("\n")
        );
        assert!(!output.contains(['┌', '┬', '┐', '│', '├', '┼', '┤', '└', '┴', '┘']));
    }

    #[test]
    fn frame_selection_accepts_fallback_aliases() {
        for value in [
            "ascii",
            "plain-ascii",
            "fallback",
            "non-unicode",
            "no-unicode",
        ] {
            assert_eq!(Frame::select(Some(value), |_| None), Frame::Ascii);
        }
        for value in ["unicode", "box", "utf8", "utf-8"] {
            assert_eq!(Frame::select(Some(value), |_| None), Frame::Unicode);
        }
    }

    #[test]
    fn frame_auto_falls_back_for_non_unicode_terminals() {
        assert_eq!(
            Frame::select(Some("auto"), |key| match key {
                "LANG" => Some("C".to_string()),
                _ => None,
            }),
            Frame::Ascii
        );
        assert_eq!(
            Frame::select(Some("auto"), |key| match key {
                "LANG" => Some("en_US.UTF-8".to_string()),
                _ => None,
            }),
            Frame::Unicode
        );
        assert_eq!(
            Frame::select(Some("auto"), |key| match key {
                "CODEXPLAIN_NO_UNICODE" => Some("1".to_string()),
                "LANG" => Some("en_US.UTF-8".to_string()),
                _ => None,
            }),
            Frame::Ascii
        );
        assert_eq!(
            Frame::select(Some("terminal"), |key| match key {
                "TERM" => Some("dumb".to_string()),
                "LANG" => Some("en_US.UTF-8".to_string()),
                _ => None,
            }),
            Frame::Ascii
        );
    }

    #[test]
    fn ansi_themed_table_snapshot_colors_structure_without_hiding_text() {
        let output = table(
            &["Label", "Meaning"],
            &[
                vec!["TLDR".to_string(), "ready".to_string()],
                vec!["위험".to_string(), "color is supplemental".to_string()],
            ],
            Frame::Unicode,
            Theme::Ocean,
            true,
            80,
        );

        assert!(output.starts_with("\x1b[36m┌"));
        assert!(output.contains("\x1b[1;34m Label "));
        assert!(output.contains("\x1b[1;32m TLDR "));
        assert!(output.contains("\x1b[1;33m 위험 "));
        assert!(output.contains("\x1b[96m color is supplemental "));
        assert!(output.contains("TLDR"));
        assert!(output.contains("위험"));
        assert!(output.contains("ready"));
    }

    #[test]
    fn ansi_colored_and_plain_tables_have_same_visible_widths() {
        let rows = [
            vec!["장점".to_string(), "색은 보조 신호입니다.".to_string()],
            vec![
                "위험".to_string(),
                "텍스트 의미가 항상 남습니다.".to_string(),
            ],
        ];
        let plain = table(
            &["Label", "Meaning"],
            &rows,
            Frame::Unicode,
            Theme::None,
            true,
            72,
        );
        let colored = table(
            &["Label", "Meaning"],
            &rows,
            Frame::Unicode,
            Theme::Ocean,
            true,
            72,
        );

        assert_eq!(visible_line_widths(&colored), visible_line_widths(&plain));
        assert!(colored.contains("\x1b[36m┌"));
        assert!(colored.contains("\x1b[1;32m 장점 "));
        assert!(colored.contains("\x1b[1;33m 위험 "));
        assert!(colored.contains("색은 보조 신호입니다."));
        assert!(colored.contains("텍스트 의미가 항상 남습니다."));
    }

    #[test]
    fn reports_storage_candidates() {
        assert!(dir_size(Path::new("definitely-missing-dir")) == 0);
    }

    #[test]
    fn measures_storage_dirs_and_treats_missing_dirs_as_zero() {
        let root = env::temp_dir().join(format!("codexplain-storage-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("dist/assets")).unwrap();
        fs::write(root.join("dist/assets/app.js"), b"12345").unwrap();

        let measurement = measure_storage(&root);
        let sizes: Vec<(&str, u64)> = measurement
            .directories
            .iter()
            .map(|directory| (directory.name, directory.bytes))
            .collect();

        assert!(measurement.free_kib.is_some());
        assert_eq!(sizes, vec![("target", 0), ("dist", 5), ("node_modules", 0)]);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_policy_only_targets_cargo_artifacts_under_low_space() {
        let clean_options = StorageCheckOptions {
            min_free: FreeSpaceThreshold {
                amount: 5,
                unit: StorageUnit::Gb,
            },
            clean: true,
        };
        let dry_run_options = StorageCheckOptions {
            clean: false,
            ..clean_options
        };

        assert_eq!(cleanup_targets(4.9, clean_options), vec!["target"]);
        assert!(!cleanup_targets(4.9, clean_options).contains(&"dist"));
        assert!(!cleanup_targets(4.9, clean_options).contains(&"node_modules"));
        assert!(cleanup_targets(5.1, clean_options).is_empty());
        assert!(cleanup_targets(4.9, dry_run_options).is_empty());
    }

    #[test]
    fn cleanup_policy_covers_threshold_gate_and_target_only_refusal() {
        let root = env::temp_dir().join(format!(
            "codexplain-storage-policy-refusal-test-{}",
            std::process::id()
        ));
        let options = StorageCheckOptions {
            min_free: FreeSpaceThreshold {
                amount: 5,
                unit: StorageUnit::Gb,
            },
            clean: true,
        };
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("target/debug/artifact.tmp"), b"cargo").unwrap();
        fs::write(root.join("dist/bundle.js"), b"bundle").unwrap();
        fs::write(root.join("node_modules/package.txt"), b"dependency").unwrap();

        assert_eq!(cleanup_targets(4.99, options), vec!["target"]);
        assert!(cleanup_targets(5.0, options).is_empty());
        assert!(cleanup_targets(5.01, options).is_empty());

        let dist_error = cleanup_project_storage_dir(&root, "dist").unwrap_err();
        let node_modules_error = cleanup_project_storage_dir(&root, "node_modules").unwrap_err();
        assert_eq!(dist_error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(node_modules_error.kind(), io::ErrorKind::InvalidInput);
        assert!(root.join("dist/bundle.js").exists());
        assert!(root.join("node_modules/package.txt").exists());
        assert!(root.join("target/debug/artifact.tmp").exists());

        let cleaned = cleanup_project_storage_dir(&root, "target").unwrap();
        assert_eq!(cleaned, TargetCleanup::Removed);
        assert!(!root.join("target").exists());
        assert!(root.join("dist/bundle.js").exists());
        assert!(root.join("node_modules/package.txt").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_project_target_resolves_root_and_deletes_only_target() {
        let root = env::temp_dir().join(format!(
            "codexplain-target-clean-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::create_dir_all(root.join("dist")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("target/debug/artifact.tmp"), b"cargo").unwrap();
        fs::write(root.join("dist/app.js"), b"bundle").unwrap();
        fs::write(root.join("node_modules/package.txt"), b"dependency").unwrap();

        let cleaned = cleanup_project_target(&root.join("nested/..")).unwrap();

        assert_eq!(cleaned, TargetCleanup::Removed);
        assert!(!root.join("target").exists());
        assert!(root.join("dist/app.js").exists());
        assert!(root.join("node_modules/package.txt").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_project_target_refuses_target_symlink() {
        let root = env::temp_dir().join(format!(
            "codexplain-target-symlink-test-{}",
            std::process::id()
        ));
        let external = env::temp_dir().join(format!(
            "codexplain-target-external-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&external);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&external).unwrap();
        fs::write(external.join("artifact.tmp"), b"outside").unwrap();
        std::os::unix::fs::symlink(&external, root.join("target")).unwrap();

        let error = cleanup_project_target(&root).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(external.join("artifact.tmp").exists());
        assert!(fs::symlink_metadata(root.join("target")).is_ok());

        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(external).unwrap();
    }

    #[test]
    fn storage_check_resolves_effective_threshold_and_message() {
        let config = StorageCheckConfig {
            min_free: FreeSpaceThreshold {
                amount: 2,
                unit: StorageUnit::Gb,
            },
        };
        let from_config = resolve_storage_threshold(&[], config);
        let from_cli = resolve_storage_threshold(&["--min-free-gb".into(), "7".into()], config);
        let invalid_cli_falls_back =
            resolve_storage_threshold(&["--min-free-gb".into(), "nope".into()], config);
        let clean_from_config = resolve_storage_check_options(&["--clean".into()], config);

        assert_eq!(from_config.amount, 2);
        assert_eq!(from_cli.amount, 7);
        assert_eq!(invalid_cli_falls_back.amount, 2);
        assert_eq!(
            clean_from_config,
            StorageCheckOptions {
                min_free: from_config,
                clean: true,
            }
        );
        assert_eq!(cleanup_targets(1.75, clean_from_config), vec!["target"]);

        assert_eq!(
            storage_result_message(2.25, from_config),
            (
                "pass",
                "pass: free_gb 2.25 meets effective_min_free_gb 2".to_string()
            )
        );
        assert_eq!(
            storage_result_message(1.75, from_config),
            (
                "fail",
                "fail: free_gb 1.75 is below effective_min_free_gb 2".to_string()
            )
        );
    }

    #[test]
    fn storage_threshold_decision_covers_fail_boundary_pass_boundary_and_above() {
        let threshold = FreeSpaceThreshold {
            amount: 5,
            unit: StorageUnit::Gb,
        };

        let cases = [
            (
                4.999,
                "fail",
                "fail: free_gb 5.00 is below effective_min_free_gb 5",
            ),
            (
                5.0,
                "pass",
                "pass: free_gb 5.00 meets effective_min_free_gb 5",
            ),
            (
                5.001,
                "pass",
                "pass: free_gb 5.00 meets effective_min_free_gb 5",
            ),
        ];

        for (available_gb, expected_result, expected_message) in cases {
            let (result, message) = storage_result_message(available_gb, threshold);
            assert_eq!(result, expected_result);
            assert_eq!(message, expected_message);
        }
    }

    #[test]
    fn storage_report_status_uses_raw_threshold_boundary_not_rounded_display() {
        let threshold = FreeSpaceThreshold {
            amount: 5,
            unit: StorageUnit::Gb,
        };
        let just_below = (5 * 1024 * 1024) - 1;
        let exactly_at = 5 * 1024 * 1024;
        let just_above = (5 * 1024 * 1024) + 1;

        let below_report = format_storage_report(
            &storage_report_fixture(just_below),
            threshold,
            Some(StorageReportDetail::SuggestedCleanup),
        );
        let exact_report =
            format_storage_report(&storage_report_fixture(exactly_at), threshold, None);
        let above_report =
            format_storage_report(&storage_report_fixture(just_above), threshold, None);

        assert!(below_report.contains("free_gb=5.00"));
        assert!(below_report.contains("result=fail"));
        assert!(below_report.contains("status=low-space"));
        assert!(below_report.contains("suggested_cleanup=rerun with --clean to remove target/"));

        assert!(exact_report.contains("free_gb=5.00"));
        assert!(exact_report.contains("result=pass"));
        assert!(exact_report.contains("status=ok"));

        assert!(above_report.contains("free_gb=5.00"));
        assert!(above_report.contains("result=pass"));
        assert!(above_report.contains("status=ok"));
    }

    fn storage_report_fixture(free_kib: u64) -> StorageMeasurement {
        StorageMeasurement {
            free_kib: Some(free_kib),
            directories: vec![
                DirectoryMeasurement {
                    name: "target",
                    bytes: 1_572_864,
                },
                DirectoryMeasurement {
                    name: "dist",
                    bytes: 262_144,
                },
                DirectoryMeasurement {
                    name: "node_modules",
                    bytes: 0,
                },
            ],
        }
    }

    fn storage_report_keys(output: &str) -> Vec<&str> {
        output
            .lines()
            .map(|line| line.split_once('=').map(|(key, _)| key).unwrap())
            .collect()
    }

    #[test]
    fn storage_report_output_has_required_fields_in_stable_order() {
        let threshold = FreeSpaceThreshold {
            amount: 5,
            unit: StorageUnit::Gb,
        };
        let output = format_storage_report(
            &storage_report_fixture(6 * 1024 * 1024),
            threshold,
            Some(StorageReportDetail::Cleaned(TargetCleanup::Removed)),
        );
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(
            storage_report_keys(&output),
            vec![
                "contract",
                "free_gb",
                "min_free_gb",
                "effective_min_free_gb",
                "target_mb",
                "dist_mb",
                "node_modules_mb",
                "result",
                "message",
                "status",
            ]
        );
        assert!(lines.iter().all(|line| line.matches('=').count() == 1));
        assert_eq!(lines[0], "contract=codexplain.storage-check.v1");
        assert_eq!(lines[1], "free_gb=6.00");
        assert_eq!(lines[2], "min_free_gb=5");
        assert_eq!(lines[3], "effective_min_free_gb=5");
        assert_eq!(lines[4], "target_mb=1.5");
        assert_eq!(lines[5], "dist_mb=0.2");
        assert_eq!(lines[6], "node_modules_mb=0.0");
        assert_eq!(lines[7], "result=pass");
        assert_eq!(
            lines[8],
            "message=pass: free_gb 6.00 meets effective_min_free_gb 5"
        );
        assert_eq!(lines[9], "status=ok");
        assert!(!output.contains("\x1b["));
        assert!(!output.contains(['┌', '┬', '┐', '│', '├', '┼', '┤', '└', '┴', '┘']));
    }

    #[test]
    fn storage_report_low_space_adds_exactly_one_action_field() {
        let threshold = FreeSpaceThreshold {
            amount: 5,
            unit: StorageUnit::Gb,
        };
        let suggested =
            format_storage_report(&storage_report_fixture(4 * 1024 * 1024), threshold, None);
        let cleaned = format_storage_report(
            &storage_report_fixture(4 * 1024 * 1024),
            threshold,
            Some(StorageReportDetail::Cleaned(TargetCleanup::AlreadyAbsent)),
        );
        let clean_error = format_storage_report(
            &storage_report_fixture(4 * 1024 * 1024),
            threshold,
            Some(StorageReportDetail::CleanError(
                "permission denied".to_string(),
            )),
        );

        assert_eq!(
            storage_report_keys(&suggested),
            vec![
                "contract",
                "free_gb",
                "min_free_gb",
                "effective_min_free_gb",
                "target_mb",
                "dist_mb",
                "node_modules_mb",
                "result",
                "message",
                "status",
                "suggested_cleanup",
            ]
        );
        assert!(suggested.contains("result=fail"));
        assert!(suggested.contains("status=low-space"));
        assert!(suggested.contains("suggested_cleanup=rerun with --clean to remove target/"));
        assert!(!suggested.contains("cleaned="));
        assert!(cleaned.ends_with("cleaned=target_already_absent"));
        assert!(!cleaned.contains("suggested_cleanup="));
        assert!(clean_error.ends_with("clean_error=target:permission denied"));
        assert!(!clean_error.contains("suggested_cleanup="));
    }

    #[test]
    fn storage_check_config_defines_default_threshold_units_and_safe_fallbacks() {
        let default = StorageCheckConfig::default();
        assert_eq!(default.min_free.amount, 5);
        assert_eq!(default.min_free.unit.name(), "gb");

        assert_eq!(
            parse_storage_check_config(r#"{"storageCheck":{"minFree":{"value":4,"unit":"gb"}}}"#),
            StorageCheckConfig {
                min_free: FreeSpaceThreshold {
                    amount: 4,
                    unit: StorageUnit::Gb,
                },
            }
        );
        assert_eq!(
            parse_storage_check_config(r#"{"storageCheck":{"minFreeGb":6}}"#),
            StorageCheckConfig {
                min_free: FreeSpaceThreshold {
                    amount: 6,
                    unit: StorageUnit::Gb,
                },
            }
        );
        assert_eq!(
            parse_storage_check_config(
                r#"{"storageCheck":{"minFree":{"value":"large","unit":"tb"}}}"#
            ),
            default
        );
    }

    #[test]
    fn strict_artifacts_are_preserved_before_shaping() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        assert_eq!(
            shape("JSON만 출력해", "{\"ok\":true}", &profile, 80),
            "{\"ok\":true}"
        );
        assert_eq!(
            shape(
                "테스트 출력만 보여줘",
                "PASS test/example.test.js",
                &profile,
                80
            ),
            "PASS test/example.test.js"
        );
    }

    #[test]
    fn prompt_signal_catalog_covers_required_renderer_intents() {
        let catalog = prompt_signal_map();

        for renderer in [
            RendererKind::Table,
            RendererKind::ProsCons,
            RendererKind::Formula,
            RendererKind::IndexedList,
            RendererKind::Flow,
            RendererKind::Progress,
            RendererKind::TldrProse,
            RendererKind::Prose,
        ] {
            assert!(
                catalog.iter().any(|signal| signal.renderer == renderer),
                "missing renderer signal for {renderer:?}"
            );
        }
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::Comparison));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::DecisionRule));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::OrderedSteps));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::ProcessFlow));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::ProgressReport));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::StructuredSummary));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::GeneralAnswer));
    }

    #[test]
    fn rust_renderer_selection_maps_prompt_signals() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };

        let cases = [
            (
                "표로 정리해줘",
                RendererKind::Table,
                ExplanationIntent::StructuredSummary,
            ),
            (
                "JS와 Rust 장단점 알려줘",
                RendererKind::ProsCons,
                ExplanationIntent::Comparison,
            ),
            (
                "판단 공식을 수식으로 보여줘",
                RendererKind::Formula,
                ExplanationIntent::DecisionRule,
            ),
            (
                "1,2,3 번호 목록으로 설명",
                RendererKind::IndexedList,
                ExplanationIntent::OrderedSteps,
            ),
            (
                "처리 흐름을 보여줘",
                RendererKind::Flow,
                ExplanationIntent::ProcessFlow,
            ),
            (
                "진행상황을 progress bar로 보고해줘",
                RendererKind::Progress,
                ExplanationIntent::ProgressReport,
            ),
            (
                "현재 상태를 TLDR로 요약해줘",
                RendererKind::TldrProse,
                ExplanationIntent::StatusSummary,
            ),
            (
                "그냥 설명해줘",
                RendererKind::Prose,
                ExplanationIntent::GeneralAnswer,
            ),
        ];

        for (prompt, renderer, intent) in cases {
            let selection = select_renderer(prompt, &profile);
            assert_eq!(selection.renderer, renderer, "{prompt}");
            assert_eq!(selection.intent, intent, "{prompt}");
        }
    }

    #[test]
    fn renderer_selection_keyword_paths_shape_distinct_terminal_outputs() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let response = "작업 완료. 검증 완료. 다음 단계는 배포입니다.";
        let cases = [
            ("표로 정리해줘", RendererKind::Table, "┌", "TLDR"),
            (
                "JS와 Rust 장단점 알려줘",
                RendererKind::ProsCons,
                "JS / Node",
                "Rust",
            ),
            (
                "판단 공식을 수식으로 보여줘",
                RendererKind::Formula,
                "핵심식 : 설명 품질 = f",
                "설명",
            ),
            (
                "1,2,3 번호 목록으로 설명",
                RendererKind::IndexedList,
                "1. │ 작업 완료.",
                "2. │ 검증 완료.",
            ),
            ("처리 흐름을 보여줘", RendererKind::Flow, "▼", "정책 검사"),
            (
                "진행상황을 progress bar로 보고해줘",
                RendererKind::Progress,
                "진행상황: ",
                "[████████████████████████████████████] 100%",
            ),
            (
                "현재 상태를 TLDR로 요약해줘",
                RendererKind::TldrProse,
                "TLDR: ",
                "요약하면, ",
            ),
            (
                "그냥 설명해줘",
                RendererKind::Prose,
                "요약하면, ",
                "작업 완료.",
            ),
        ];

        for (prompt, renderer, expected_primary, expected_secondary) in cases {
            let selection = select_renderer(prompt, &profile);
            let output = shape(prompt, response, &profile, 80);

            assert_eq!(selection.renderer, renderer, "{prompt}");
            assert!(output.contains(expected_primary), "{prompt}: {output}");
            assert!(output.contains(expected_secondary), "{prompt}: {output}");
        }
    }

    #[test]
    fn progress_renderer_reports_status_text_bar_and_detail_table() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "진행상황을 progress bar로 보고해줘",
            "현재 3/5 단계 진행 중입니다. 검증은 계속 진행 중입니다.",
            &profile,
            80,
        );

        assert!(output.contains("진행상황: 진행 중"), "{output}");
        assert!(
            output.contains("[██████████████████████░░░░░░░░░░░░░░]  60%"),
            "{output}"
        );
        assert!(output.contains("│ 진척      │ 진행 중 · 60%"), "{output}");
        assert!(output.contains("│ 다음 행동 │"), "{output}");
    }

    #[test]
    fn three_stage_depth_controls_change_architecture_detail_rows() {
        let light = Profile {
            theme: Theme::None,
            explanation_depth: "light".to_string(),
            architecture_depth: "overview".to_string(),
            abstraction_level: "concrete".to_string(),
            ..Profile::default()
        };
        let deep = Profile {
            theme: Theme::None,
            explanation_depth: "deep".to_string(),
            architecture_depth: "internals".to_string(),
            abstraction_level: "strategy".to_string(),
            ..Profile::default()
        };
        let response = "Codexplain은 Rust core와 Node wrapper를 함께 씁니다. Rust는 렌더링을 맡습니다. Node는 설치를 맡습니다. 검증은 cargo test입니다.";

        let light_output = shape("아키텍처를 표로 설명해줘", response, &light, 100);
        let deep_output = shape("아키텍처를 표로 설명해줘", response, &deep, 100);

        assert!(
            light_output.contains("CLI → Policy → Renderer"),
            "{light_output}"
        );
        assert!(
            light_output.contains("바로 실행 가능한 수준"),
            "{light_output}"
        );
        assert!(!light_output.contains("선택기"), "{light_output}");
        assert!(!light_output.contains("다음 행동"), "{light_output}");
        assert!(deep_output.contains("Rust selector"), "{deep_output}");
        assert!(deep_output.contains("선택적 planner hint"), "{deep_output}");
        assert!(deep_output.contains("상위 의사결정"), "{deep_output}");
        assert!(deep_output.contains("다음 행동"), "{deep_output}");
    }

    #[test]
    fn ux_density_numerically_controls_implicit_progress_components() {
        let sparse = Profile {
            theme: Theme::None,
            ux_density: 0,
            ..Profile::default()
        };
        let dense = Profile {
            theme: Theme::None,
            ux_density: 100,
            ..Profile::default()
        };

        let sparse_items = requested_ux_components(
            "진행상황을 보고해줘",
            "현재 2/5 단계 진행 중입니다.",
            &sparse,
        );
        let dense_items = requested_ux_components(
            "진행상황을 보고해줘",
            "현재 2/5 단계 진행 중입니다.",
            &dense,
        );

        assert!(
            !sparse_items.contains(&UxComponent::Checklist),
            "{sparse_items:?}"
        );
        assert!(
            dense_items.contains(&UxComponent::Checklist),
            "{dense_items:?}"
        );
        assert!(
            dense_items.contains(&UxComponent::NextAction),
            "{dense_items:?}"
        );
    }

    #[test]
    fn ux_planner_plan_parser_accepts_llm_style_component_names() {
        let plan = parse_ux_component_plan(
            r#"{"components":["status-badge","risk-panel","decision_matrix","next-action"]}"#,
        );

        assert!(plan.contains(&UxComponent::StatusBadge), "{plan:?}");
        assert!(plan.contains(&UxComponent::RiskPanel), "{plan:?}");
        assert!(plan.contains(&UxComponent::DecisionMatrix), "{plan:?}");
        assert!(plan.contains(&UxComponent::NextAction), "{plan:?}");
    }

    #[test]
    fn rich_ux_prompt_combines_all_visual_status_components() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "모든 UX 요소를 풍부하게 보여줘: status badge, checklist, risk, confidence, diff, decision matrix, next action, eta, callout",
            "현재 4/5 단계 진행 중입니다. Rust 테스트는 통과했고 릴리즈 검증이 남았습니다.",
            &profile,
            100,
        );

        assert!(output.contains("[RUNNING] 마무리 중"), "{output}");
        assert!(output.contains("체크포인트"), "{output}");
        assert!(output.contains("│ 위험"), "{output}");
        assert!(output.contains("확신도"), "{output}");
        assert!(output.contains("│ 변경"), "{output}");
        assert!(output.contains("│ 선택"), "{output}");
        assert!(output.contains("다음 행동:"), "{output}");
        assert!(output.contains("ETA:"), "{output}");
        assert!(output.contains("│ 주의"), "{output}");
    }

    #[test]
    fn ux_components_are_selected_dynamically_from_prompt_and_failure_text() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "리스크와 다음 행동만 알려줘",
            "실패: provider timeout 때문에 검증이 중단됐습니다.",
            &profile,
            80,
        );

        assert!(output.contains("[BLOCKED] 확인 필요"), "{output}");
        assert!(output.contains("│ 위험"), "{output}");
        assert!(output.contains("다음 행동:"), "{output}");
        assert!(output.contains("│ 주의"), "{output}");
        assert!(!output.contains("확신도"), "{output}");
    }

    #[test]
    fn compound_prompts_combine_architecture_tradeoff_and_formula_renderers() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "아키텍처를 표와 flow로 보여주고 JS와 Rust pros and cons와 수식도 비교해줘",
            "Codexplain은 Rust core와 Node wrapper를 함께 씁니다. Rust는 렌더링을 맡고 Node는 설치와 Codex wrapper를 맡습니다.",
            &profile,
            132,
        );

        assert!(output.contains("│ 계층"), "{output}");
        assert!(output.contains("│ 입력"), "{output}");
        assert!(output.contains("JS / Node"), "{output}");
        assert!(output.contains("Rust"), "{output}");
        assert!(output.contains("핵심식 : 설명 품질 = f"), "{output}");
    }

    #[test]
    fn responsive_architecture_panels_stack_when_terminal_is_narrow() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘",
            "CLI가 입력을 받고 Rust core가 렌더링합니다.",
            &profile,
            60,
        );

        assert!(output.contains("│ 계층"), "{output}");
        assert!(output.contains("│ 입력"), "{output}");
        let table_pos = output.find("│ 계층").unwrap();
        let flow_pos = output.find("│ 입력").unwrap();
        assert!(flow_pos > table_pos, "{output}");
    }

    #[test]
    fn ambiguous_prompts_fall_back_to_plain_prose_without_structural_renderers() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let response = "작업 완료. 검증 완료.";

        for prompt in ["", "이 부분 설명해줘", "어떻게 보면 될까?"] {
            let selection = select_renderer(prompt, &profile);
            let output = shape(prompt, response, &profile, 80);

            assert_eq!(selection.renderer, RendererKind::Prose, "{prompt}");
            assert_eq!(
                selection.intent,
                ExplanationIntent::GeneralAnswer,
                "{prompt}"
            );
            assert_eq!(selection.signal.kind, PromptSignalKind::Default, "{prompt}");
            assert!(output.starts_with("요약하면, "), "{prompt}: {output}");
            assert!(!output.contains('┌'), "{prompt}: {output}");
            assert!(!output.contains('▼'), "{prompt}: {output}");
            assert!(!output.contains("TLDR: "), "{prompt}: {output}");
        }
    }

    #[test]
    fn invalid_profile_structure_does_not_block_ambiguous_prompt_fallback() {
        let profile = Profile {
            theme: Theme::None,
            preferred_structure: "unknown-renderer".to_string(),
            ..Profile::default()
        };

        let selection = select_renderer("설명해줘", &profile);

        assert_eq!(selection.renderer, RendererKind::Prose);
        assert_eq!(selection.intent, ExplanationIntent::GeneralAnswer);
        assert_eq!(selection.signal.kind, PromptSignalKind::Default);
    }

    #[test]
    fn profile_preferred_structure_overrides_prompt_default() {
        let profile = Profile {
            theme: Theme::None,
            preferred_structure: "flow".to_string(),
            ..Profile::default()
        };
        let selection = select_renderer("그냥 설명해줘", &profile);

        assert_eq!(selection.renderer, RendererKind::Flow);
        assert_eq!(selection.intent, ExplanationIntent::ProcessFlow);
        assert_eq!(selection.signal.kind, PromptSignalKind::ProfilePreference);
    }

    #[test]
    fn dispatch_explanation_routes_selected_intents_to_renderers() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let summary = "첫째입니다. 둘째입니다.";
        let signal = PromptSignal {
            renderer: RendererKind::Prose,
            intent: ExplanationIntent::GeneralAnswer,
            kind: PromptSignalKind::Default,
            pattern: "test",
        };
        let cases = [
            (ExplanationIntent::Comparison, "JS / Node"),
            (ExplanationIntent::DecisionRule, "핵심식 : 설명 품질 = f"),
            (ExplanationIntent::OrderedSteps, "1. │ 첫째입니다."),
            (ExplanationIntent::ProcessFlow, "▼"),
            (ExplanationIntent::StructuredSummary, "┌"),
            (ExplanationIntent::StatusSummary, "TLDR: "),
            (ExplanationIntent::GeneralAnswer, "요약하면, "),
        ];

        for (intent, expected) in cases {
            let output = dispatch_explanation(
                RendererSelection {
                    renderer: RendererKind::Prose,
                    intent,
                    signal,
                },
                "",
                summary,
                summary,
                &profile,
                80,
            );
            assert!(output.contains(expected), "{intent:?}: {output}");
        }
    }

    #[test]
    fn shape_uses_rust_selection_model_without_rewriting_strict_outputs() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };

        let table_output = shape("표로 정리해줘", "작업 완료. 검증 완료.", &profile, 80);
        let formula_output = shape("공식으로 설명", "작업 완료. 검증 완료.", &profile, 80);
        let prose_output = shape("설명해줘", "작업 완료. 검증 완료.", &profile, 80);

        assert!(table_output.contains('┌'));
        assert!(table_output.contains("TLDR"));
        assert!(formula_output.contains("핵심식 : 설명 품질 = f"));
        assert!(prose_output.starts_with("요약하면, "));
        assert_eq!(
            shape("JSON만 출력해", "{\"ok\":true}", &profile, 80),
            "{\"ok\":true}"
        );
    }
}
