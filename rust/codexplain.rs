use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
enum ColorOutput {
    Terminal,
    Ansi,
    Html,
    Markdown,
    Plain,
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
    Command,
    Path,
    Artifact,
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
    command: AnsiStyle,
    path: AnsiStyle,
    artifact: AnsiStyle,
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
    max_width: usize,
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
    CauseEffect,
    ProblemDiagnosis,
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
    CauseEffectReport,
    ProblemDiagnosis,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct CustomStyle {
    name: String,
    trigger: String,
    renderers: Vec<RendererKind>,
    body: String,
    tone: String,
    example: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UxComponent {
    StatusBadge,
    WorkflowProgress,
    Toggle,
    Quote,
    Divider,
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
    emoji_cues: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            theme: Theme::Mono,
            frame: Frame::Unicode,
            index_style: IndexStyle::Decimal,
            detail: "deep".to_string(),
            style: "technical".to_string(),
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
            emoji_cues: true,
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
    if env_flag_enabled(env_value("CODEXPLAIN_NO_COLOR"))
        || env_flag_enabled(env_value("CLAUDEX_NO_COLOR"))
    {
        return false;
    }
    if let Some(value) = env_value("CODEXPLAIN_COLOR").or_else(|| env_value("CLAUDEX_COLOR")) {
        match value.trim().to_ascii_lowercase().as_str() {
            "0" | "false" | "off" | "none" | "never" | "no-color" | "plain" => return false,
            "1" | "true" | "on" | "yes" | "always" | "force" | "color" => return true,
            _ => {}
        }
    }
    if env_flag_enabled(env_value("CLICOLOR_FORCE")) {
        return true;
    }
    if env_nonempty(env_value("NO_COLOR")) {
        return false;
    }
    if matches!(env_value("TERM").as_deref(), Some("dumb")) {
        return false;
    }
    if matches!(env_value("CLICOLOR").as_deref(), Some("0")) {
        return false;
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
        let text = value.into();
        Self {
            text: normalize_table_cell_text(&text),
        }
    }

    fn width(&self) -> usize {
        self.text.lines().map(visible_width).max().unwrap_or(0)
    }

    fn wrapped(&self, width: usize) -> Vec<String> {
        wrap_text(&self.text, width)
    }
}

fn normalize_table_cell_text(text: &str) -> String {
    text.replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
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
            max_width: diagram.max_width,
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
            "cause-effect" | "cause_effect" | "causal" | "cause" | "effect" | "원인-결과"
            | "원인결과" | "원인" | "결과" => Some(Self::CauseEffect),
            "problem-diagnosis" | "diagnosis" | "debug-explain" | "why-not" | "why-failed"
            | "문제진단" | "문제-진단" | "왜안됨" | "왜-안됨" => {
                Some(Self::ProblemDiagnosis)
            }
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
            Self::CauseEffect => ExplanationIntent::CauseEffectReport,
            Self::ProblemDiagnosis => ExplanationIntent::ProblemDiagnosis,
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
        renderer: RendererKind::ProblemDiagnosis,
        intent: ExplanationIntent::ProblemDiagnosis,
        kind: PromptSignalKind::Keyword,
        pattern: "왜.*안|왜.*안되|왜.*안돼|왜.*실패|왜.*오류|안되는.*이유|안 되는.*이유|문제.*설명|문제.*원인|원인.*해결책|diagnose|diagnosis|why.*not|why.*fail|why.*failed|why.*error|what.*wrong|root cause.*fix",
    },
    PromptSignal {
        renderer: RendererKind::CauseEffect,
        intent: ExplanationIntent::CauseEffectReport,
        kind: PromptSignalKind::Keyword,
        pattern: "원인-결과|원인결과|원인.*결과|왜.*그래서|cause-effect|cause and effect|cause.*effect|causal|root cause|인과|결과 리포트",
    },
    PromptSignal {
        renderer: RendererKind::IndexedList,
        intent: ExplanationIntent::OrderedSteps,
        kind: PromptSignalKind::Keyword,
        pattern:
            "1,2,3|번호|순번|목록|리스트|단계별|numbered|indexed|list|두 가지|2가지|세 가지|3가지|크게 두|과정|순서|첫째|둘째",
    },
    PromptSignal {
        renderer: RendererKind::Progress,
        intent: ExplanationIntent::ProgressReport,
        kind: PromptSignalKind::Keyword,
        pattern:
            "progress|progress bar|진행상황|진행 상황|진척|몇 퍼센트|percent|상태 보고|작업 상태|작업 로그|탐색 로그|transcript|전체적인|매크로|macro|프로세스",
    },
    PromptSignal {
        renderer: RendererKind::Progress,
        intent: ExplanationIntent::ProgressReport,
        kind: PromptSignalKind::Keyword,
        pattern:
            "workflow|workflow progress|harness|progress block|개발 워크플로우|워크플로우|하네스|사용자 정의 워크플로우",
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

    if architecture_prompt_requested(prompt) {
        return RendererSelection {
            renderer: RendererKind::Flow,
            intent: ExplanationIntent::ProcessFlow,
            signal: PromptSignal {
                renderer: RendererKind::Flow,
                intent: ExplanationIntent::ProcessFlow,
                kind: PromptSignalKind::Keyword,
                pattern: "architecture-diagram-required",
            },
        };
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
    if architecture_prompt_requested(prompt) {
        renderers.push(RendererKind::Flow);
        renderers.push(RendererKind::Table);
    }
    for signal in prompt_signal_map()
        .iter()
        .copied()
        .filter(|signal| signal.kind != PromptSignalKind::Default)
    {
        if prompt_matches_signal(prompt, signal) && !renderers.contains(&signal.renderer) {
            renderers.push(signal.renderer);
        }
    }
    for style in matching_custom_styles(prompt) {
        for renderer in style.renderers {
            if !renderers.contains(&renderer) {
                renderers.push(renderer);
            }
        }
    }
    renderers
}

fn architecture_prompt_requested(prompt: &str) -> bool {
    [
        "아키텍처",
        "구조",
        "설계",
        "컴포넌트",
        "구성",
        "순서도",
        "전개도",
        "다이어그램",
        "도식",
        "architecture",
        "system design",
        "component",
        "components",
        "structure",
        "diagram",
        "map",
    ]
    .iter()
    .any(|pattern| prompt_matches_pattern(prompt, pattern))
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
        "workflow" | "workflow-progress" | "harness-progress" | "dev-progress" => {
            Some(UxComponent::WorkflowProgress)
        }
        "toggle" | "notion-toggle" | "fold" | "foldout" => Some(UxComponent::Toggle),
        "quote" | "blockquote" | "notion-quote" => Some(UxComponent::Quote),
        "divider" | "separator" | "section-line" => Some(UxComponent::Divider),
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
        || text.contains("workflow")
        || text.contains("harness")
        || text.contains("progress block")
        || prompt.contains("워크플로우")
        || prompt.contains("하네스")
        || prompt.contains("개발")
        || prompt.contains("사용자 정의 워크플로우")
    {
        add_component_score(&mut scores, UxComponent::WorkflowProgress, explicit_score);
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
    if full_kit
        || text.contains("notion")
        || prompt.contains("노션")
        || text.contains("toggle")
        || prompt.contains("토글")
        || prompt.contains("접기")
    {
        add_component_score(&mut scores, UxComponent::Toggle, explicit_score);
    }
    if full_kit || text.contains("quote") || prompt.contains("인용") || prompt.contains("핵심 문장")
    {
        add_component_score(&mut scores, UxComponent::Quote, explicit_score);
    }
    if full_kit
        || text.contains("divider")
        || prompt.contains("구분선")
        || prompt.contains("분리선")
    {
        add_component_score(&mut scores, UxComponent::Divider, explicit_score);
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
        add_component_score(&mut scores, UxComponent::WorkflowProgress, progress_score);
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
        let separator_width = 1;
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
            "command" => Self::Command,
            "path" => Self::Path,
            "artifact" => Self::Artifact,
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
        command: AnsiStyle::PLAIN,
        path: AnsiStyle::PLAIN,
        artifact: AnsiStyle::PLAIN,
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
            AnsiRole::Command => self.command,
            AnsiRole::Path => self.path,
            AnsiRole::Artifact => self.artifact,
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
                command: AnsiStyle::new("\x1b[1;35m"),
                path: AnsiStyle::new("\x1b[1;36m"),
                artifact: AnsiStyle::new("\x1b[1;33m"),
            },
            Self::Forest => ThemeSpec {
                border: AnsiStyle::new("\x1b[32m"),
                heading: AnsiStyle::new("\x1b[1;32m"),
                accent: AnsiStyle::new("\x1b[92m"),
                muted: AnsiStyle::new("\x1b[2;32m"),
                success: AnsiStyle::new("\x1b[1;32m"),
                warning: AnsiStyle::new("\x1b[1;33m"),
                danger: AnsiStyle::new("\x1b[1;31m"),
                command: AnsiStyle::new("\x1b[1;36m"),
                path: AnsiStyle::new("\x1b[1;34m"),
                artifact: AnsiStyle::new("\x1b[1;33m"),
            },
            Self::Warm => ThemeSpec {
                border: AnsiStyle::new("\x1b[33m"),
                heading: AnsiStyle::new("\x1b[1;31m"),
                accent: AnsiStyle::new("\x1b[93m"),
                muted: AnsiStyle::new("\x1b[2;33m"),
                success: AnsiStyle::new("\x1b[1;32m"),
                warning: AnsiStyle::new("\x1b[1;33m"),
                danger: AnsiStyle::new("\x1b[1;31m"),
                command: AnsiStyle::new("\x1b[1;35m"),
                path: AnsiStyle::new("\x1b[1;36m"),
                artifact: AnsiStyle::new("\x1b[1;33m"),
            },
            Self::Sunset => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;208m"),
                heading: AnsiStyle::new("\x1b[1;38;5;196m"),
                accent: AnsiStyle::new("\x1b[38;5;214m"),
                muted: AnsiStyle::new("\x1b[2;38;5;208m"),
                success: AnsiStyle::new("\x1b[1;38;5;118m"),
                warning: AnsiStyle::new("\x1b[1;38;5;220m"),
                danger: AnsiStyle::new("\x1b[1;38;5;196m"),
                command: AnsiStyle::new("\x1b[1;38;5;199m"),
                path: AnsiStyle::new("\x1b[1;38;5;45m"),
                artifact: AnsiStyle::new("\x1b[1;38;5;220m"),
            },
            Self::Grape => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;141m"),
                heading: AnsiStyle::new("\x1b[1;38;5;135m"),
                accent: AnsiStyle::new("\x1b[38;5;183m"),
                muted: AnsiStyle::new("\x1b[2;38;5;141m"),
                success: AnsiStyle::new("\x1b[1;38;5;120m"),
                warning: AnsiStyle::new("\x1b[1;38;5;222m"),
                danger: AnsiStyle::new("\x1b[1;38;5;204m"),
                command: AnsiStyle::new("\x1b[1;38;5;213m"),
                path: AnsiStyle::new("\x1b[1;38;5;87m"),
                artifact: AnsiStyle::new("\x1b[1;38;5;222m"),
            },
            Self::Slate => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;67m"),
                heading: AnsiStyle::new("\x1b[1;38;5;110m"),
                accent: AnsiStyle::new("\x1b[38;5;153m"),
                muted: AnsiStyle::new("\x1b[2;38;5;67m"),
                success: AnsiStyle::new("\x1b[1;38;5;114m"),
                warning: AnsiStyle::new("\x1b[1;38;5;179m"),
                danger: AnsiStyle::new("\x1b[1;38;5;167m"),
                command: AnsiStyle::new("\x1b[1;38;5;141m"),
                path: AnsiStyle::new("\x1b[1;38;5;153m"),
                artifact: AnsiStyle::new("\x1b[1;38;5;179m"),
            },
            Self::Rose => ThemeSpec {
                border: AnsiStyle::new("\x1b[38;5;211m"),
                heading: AnsiStyle::new("\x1b[1;38;5;199m"),
                accent: AnsiStyle::new("\x1b[38;5;218m"),
                muted: AnsiStyle::new("\x1b[2;38;5;211m"),
                success: AnsiStyle::new("\x1b[1;38;5;120m"),
                warning: AnsiStyle::new("\x1b[1;38;5;222m"),
                danger: AnsiStyle::new("\x1b[1;38;5;197m"),
                command: AnsiStyle::new("\x1b[1;38;5;135m"),
                path: AnsiStyle::new("\x1b[1;38;5;45m"),
                artifact: AnsiStyle::new("\x1b[1;38;5;220m"),
            },
            Self::Mono => ThemeSpec {
                border: AnsiStyle::new("\x1b[90m"),
                heading: AnsiStyle::new("\x1b[1m"),
                accent: AnsiStyle::new("\x1b[37m"),
                muted: AnsiStyle::new("\x1b[2m"),
                success: AnsiStyle::new("\x1b[1m"),
                warning: AnsiStyle::new("\x1b[1m"),
                danger: AnsiStyle::new("\x1b[1m"),
                command: AnsiStyle::new("\x1b[1m"),
                path: AnsiStyle::new("\x1b[4m"),
                artifact: AnsiStyle::new("\x1b[1m"),
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

fn semantic_highlight(theme: Theme, text: &str, fallback_role: &str) -> String {
    if theme == Theme::None || text.trim().is_empty() || text.contains("[") {
        return color(theme, fallback_role, text);
    }
    if !text
        .split_whitespace()
        .any(|token| highlight_role(token).is_some())
    {
        return if fallback_role == "heading" || fallback_role == "border" {
            color(theme, fallback_role, text)
        } else {
            text.to_string()
        };
    }
    let mut out = String::new();
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            flush_highlight_token(theme, &mut out, &mut token, fallback_role);
            out.push(ch);
        } else {
            token.push(ch);
        }
    }
    flush_highlight_token(theme, &mut out, &mut token, fallback_role);
    out
}

fn flush_highlight_token(theme: Theme, out: &mut String, token: &mut String, fallback_role: &str) {
    if token.is_empty() {
        return;
    }
    if let Some(role) = highlight_role(token) {
        out.push_str(&color(theme, role, token));
    } else if fallback_role == "heading" || fallback_role == "border" {
        out.push_str(&color(theme, fallback_role, token));
    } else {
        out.push_str(token);
    }
    token.clear();
}

fn highlight_role(token: &str) -> Option<&'static str> {
    let normalized = token.trim_matches(highlight_trim_char).to_ascii_lowercase();
    let normalized = strip_korean_particle(&normalized);
    match normalized.as_str() {
        "pass" | "passed" | "success" | "succeeded" | "done" | "complete" | "completed"
        | "approved" | "ok" | "완료" | "통과" | "성공" | "승인" => Some("success"),
        "fail" | "failed" | "failure" | "error" | "blocked" | "unsafe" | "denied" | "rejected"
        | "broken" | "oom" | "panic" | "실패" | "오류" | "차단" | "불가" | "위험" => {
            Some("danger")
        }
        "warn" | "warning" | "risk" | "risky" | "caution" | "drift" | "regression" | "missing"
        | "required" | "needs" | "주의" | "위험성" | "드리프트" | "회귀" | "필요" => {
            Some("warning")
        }
        "json" | "code" | "diff" | "patch" | "log" | "logs" | "test" | "tests" | "stderr"
        | "stdout" | "artifact" | "artifacts" | "테스트" | "로그" | "패치" => {
            Some("artifact")
        }
        "on" | "off" | "install" | "uninstall" | "install-codex" | "uninstall-codex" | "shape"
        | "post-response" | "quality-check" | "compat-check" | "storage-check" => Some("command"),
        _ => {
            if is_artifact_combo(&normalized) {
                Some("artifact")
            } else if is_command_like(&normalized) {
                Some("command")
            } else if is_path_like(&normalized) {
                Some("path")
            } else {
                None
            }
        }
    }
}

fn is_artifact_combo(value: &str) -> bool {
    let separators = ['/', ',', '+', '|'];
    value.chars().any(|ch| separators.contains(&ch))
        && [
            "json", "code", "diff", "patch", "log", "test", "stdout", "stderr",
        ]
        .iter()
        .any(|needle| value.contains(needle))
}

fn is_command_like(value: &str) -> bool {
    value.starts_with("codexplain")
        || value.starts_with("codex")
        || value.starts_with("npm")
        || value.starts_with("cargo")
        || value.starts_with("git")
        || value.starts_with("--")
}

fn is_path_like(value: &str) -> bool {
    value.starts_with("~/")
        || value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with('/')
        || value.contains("agents.md")
        || value.contains(".codexplain")
        || value.contains(".codex/")
        || value.ends_with(".json")
        || value.ends_with(".md")
        || value.ends_with(".rs")
        || value.ends_with(".toml")
        || value.ends_with(".patch")
}

fn highlight_trim_char(ch: char) -> bool {
    matches!(
        ch,
        ',' | '.' | ';' | ':' | ')' | '(' | '[' | ']' | '`' | '"' | '\'' | '“' | '”' | '‘' | '’'
    )
}

fn strip_korean_particle(value: &str) -> String {
    for suffix in [
        "으로", "에게", "에서", "부터", "까지", "은", "는", "이", "가", "을", "를", "와", "과",
        "에", "로",
    ] {
        if let Some(stripped) = value.strip_suffix(suffix) {
            if !stripped.is_empty() {
                return stripped.to_string();
            }
        }
    }
    value.to_string()
}

fn color_output_mode(args: &[String]) -> ColorOutput {
    if args.iter().any(|arg| arg == "--chat-color") {
        return if color_feature_enabled() {
            ColorOutput::Ansi
        } else {
            ColorOutput::Plain
        };
    }
    if let Some(value) = arg_value(args, "--color-output") {
        return parse_color_output(value);
    }
    if let Ok(value) = env::var("CODEXPLAIN_CHAT_COLOR") {
        if env_flag_enabled(Some(value.clone())) {
            return ColorOutput::Markdown;
        }
        if matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "html" | "markdown" | "chat"
        ) {
            return parse_color_output(&value);
        }
    }
    if let Ok(value) = env::var("CODEXPLAIN_COLOR_OUTPUT") {
        return parse_color_output(&value);
    }
    configured_color_output().unwrap_or(ColorOutput::Terminal)
}

fn configured_color_output() -> Option<ColorOutput> {
    fs::read_to_string(config_path()).ok().and_then(|raw| {
        extract_json_string(&raw, "defaultColorOutput")
            .or_else(|| extract_json_string(&raw, "colorOutput"))
            .map(|value| parse_color_output(&value))
    })
}

fn color_feature_enabled() -> bool {
    !matches!(configured_color_output(), Some(ColorOutput::Plain))
        && !env_flag_enabled(env::var("CODEXPLAIN_NO_COLOR").ok())
        && !env_flag_enabled(env::var("CLAUDEX_NO_COLOR").ok())
}

fn configured_tui_color_mode() -> String {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|raw| extract_json_string(&raw, "tuiAssistantColor"))
        .unwrap_or_else(|| {
            if color_feature_enabled() {
                "semantic".to_string()
            } else {
                "off".to_string()
            }
        })
}

fn tui_color_feature_enabled() -> bool {
    if !color_feature_enabled() {
        return false;
    }
    !matches!(
        configured_tui_color_mode()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "plain" | "none" | "off" | "no-color" | "false" | "0"
    )
}

fn tui_color_env_value() -> String {
    let value = configured_tui_color_mode();
    if matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "full" | "semantic"
    ) {
        value
    } else if tui_color_feature_enabled() {
        "semantic".to_string()
    } else {
        "off".to_string()
    }
}

fn parse_color_output(value: &str) -> ColorOutput {
    match value.trim().to_ascii_lowercase().as_str() {
        "ansi" | "terminal-force" | "force" => ColorOutput::Ansi,
        "html" => ColorOutput::Html,
        "chat" | "markdown" | "md" | "chat-markdown" | "markdown-chat" => ColorOutput::Markdown,
        "plain" | "none" | "off" | "no-color" => ColorOutput::Plain,
        _ => ColorOutput::Terminal,
    }
}

fn color_output_name(mode: ColorOutput) -> &'static str {
    match mode {
        ColorOutput::Terminal => "terminal",
        ColorOutput::Ansi => "ansi",
        ColorOutput::Html => "html",
        ColorOutput::Markdown => "markdown",
        ColorOutput::Plain => "plain",
    }
}

fn apply_color_output(rendered: &str, mode: ColorOutput, strict: bool) -> String {
    if strict {
        return rendered.to_string();
    }
    match mode {
        ColorOutput::Terminal | ColorOutput::Ansi => rendered.to_string(),
        ColorOutput::Plain => strip_ansi(rendered),
        ColorOutput::Html => ansi_to_html_pre(rendered),
        ColorOutput::Markdown => ansi_to_markdown_highlight(rendered),
    }
}

fn shape_for_output(
    prompt: &str,
    response: &str,
    profile: &Profile,
    width: usize,
    mode: ColorOutput,
) -> String {
    let strict = should_back_off(prompt, response);
    let rendered = shape(prompt, response, profile, width);
    let rendered = if strict {
        rendered
    } else {
        enforce_unicode_table_row_dividers(&rendered)
    };
    apply_color_output(&rendered, mode, strict)
}

fn enforce_unicode_table_row_dividers(value: &str) -> String {
    let lines = value.lines().map(str::to_string).collect::<Vec<_>>();
    if lines.is_empty() {
        return value.to_string();
    }
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if is_unicode_table_top(&lines[index]) {
            let (block, next_index) = take_unicode_table_block(&lines, index);
            out.extend(normalize_unicode_table_block(block));
            index = next_index;
        } else {
            out.push(lines[index].clone());
            index += 1;
        }
    }
    let mut joined = out.join("\n");
    if value.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

fn take_unicode_table_block(lines: &[String], start: usize) -> (Vec<String>, usize) {
    let mut block = Vec::new();
    let mut index = start;
    while index < lines.len() {
        block.push(lines[index].clone());
        if is_unicode_table_bottom(&lines[index]) {
            return (block, index + 1);
        }
        index += 1;
    }
    (block, index)
}

fn normalize_unicode_table_block(block: Vec<String>) -> Vec<String> {
    let Some(header_divider_index) = block.iter().position(|line| is_unicode_table_divider(line))
    else {
        return block;
    };
    let body_dividers = block
        .iter()
        .skip(header_divider_index + 1)
        .filter(|line| is_unicode_table_divider(line))
        .count();
    if body_dividers > 0 {
        return block;
    }
    let Some(bottom_index) = block.iter().rposition(|line| is_unicode_table_bottom(line)) else {
        return block;
    };
    let body_row_count = block
        .iter()
        .take(bottom_index)
        .skip(header_divider_index + 1)
        .filter(|line| is_unicode_table_data_row(line))
        .count();
    if body_row_count < 2 {
        return block;
    }

    let divider = block[header_divider_index].clone();
    let mut normalized = Vec::new();
    for (index, line) in block.iter().enumerate() {
        normalized.push(line.clone());
        if index > header_divider_index
            && index + 1 < bottom_index
            && is_unicode_table_data_row(line)
            && block
                .get(index + 1)
                .map(|next| is_unicode_table_data_row(next))
                .unwrap_or(false)
        {
            normalized.push(divider.clone());
        }
    }
    normalized
}

fn is_unicode_table_top(line: &str) -> bool {
    let stripped = strip_ansi(line).trim().to_string();
    stripped.starts_with('┌') && stripped.ends_with('┐')
}

fn is_unicode_table_bottom(line: &str) -> bool {
    let stripped = strip_ansi(line).trim().to_string();
    stripped.starts_with('└') && stripped.ends_with('┘')
}

fn is_unicode_table_divider(line: &str) -> bool {
    let stripped = strip_ansi(line).trim().to_string();
    stripped.starts_with('├') && stripped.ends_with('┤')
}

fn is_unicode_table_data_row(line: &str) -> bool {
    let stripped = strip_ansi(line).trim().to_string();
    stripped.starts_with('│') && stripped.ends_with('│')
}

fn strip_ansi(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            consume_ansi_escape(&mut chars);
        } else {
            out.push(ch);
        }
    }
    out
}

fn ansi_to_html_pre(value: &str) -> String {
    format!(
        "<pre class=\"codexplain-chat-color\" style=\"white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; line-height: 1.35;\">{}</pre>",
        ansi_to_html_spans(value)
    )
}

fn ansi_to_html_spans(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    let mut span_open = false;
    while let Some(ch) = chars.next() {
        if ch != '\x1b' {
            push_html_escaped(&mut out, ch);
            continue;
        }
        let Some('[') = chars.peek().copied() else {
            continue;
        };
        chars.next();
        let mut code = String::new();
        for item in chars.by_ref() {
            if item == 'm' {
                break;
            }
            code.push(item);
        }
        if span_open {
            out.push_str("</span>");
            span_open = false;
        }
        if let Some(style) = sgr_style(&code) {
            out.push_str("<span style=\"");
            out.push_str(style);
            out.push_str("\">");
            span_open = true;
        }
    }
    if span_open {
        out.push_str("</span>");
    }
    out
}

fn ansi_to_markdown_highlight(value: &str) -> String {
    let plain = strip_ansi(value);
    let terms = markdown_highlight_terms(&plain);
    if terms.is_empty() {
        return plain;
    }
    format!("{}\n\n{}", markdown_highlight_panel(&terms), plain)
}

fn markdown_highlight_terms(value: &str) -> Vec<(String, &'static str)> {
    let mut items = Vec::new();
    let mut token = String::new();
    for ch in value.chars() {
        if ch.is_whitespace() || is_box_drawing(ch) {
            push_markdown_highlight_term(&mut items, &mut token);
        } else {
            token.push(ch);
        }
    }
    push_markdown_highlight_term(&mut items, &mut token);
    items
}

fn push_markdown_highlight_term(items: &mut Vec<(String, &'static str)>, token: &mut String) {
    if token.is_empty() {
        return;
    }
    if let Some(role) = highlight_role(token) {
        let Some(label) = markdown_highlight_label(token) else {
            token.clear();
            return;
        };
        if should_add_markdown_highlight(items, &label)
            && !items
                .iter()
                .any(|(existing, _)| existing.eq_ignore_ascii_case(&label))
        {
            if label == "JSON/code/diff/log/test" {
                items.retain(|(existing, _)| {
                    !matches!(
                        existing.to_ascii_uppercase().as_str(),
                        "JSON" | "CODE" | "DIFF" | "LOG" | "TEST"
                    )
                });
            }
            items.push((label, role));
        }
    }
    token.clear();
}

fn should_add_markdown_highlight(items: &[(String, &'static str)], label: &str) -> bool {
    let upper = label.to_ascii_uppercase();
    if matches!(upper.as_str(), "JSON" | "CODE" | "DIFF" | "LOG" | "TEST")
        && items
            .iter()
            .any(|(existing, _)| existing == "JSON/code/diff/log/test")
    {
        return false;
    }
    true
}

fn markdown_highlight_label(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(highlight_trim_char);
    if trimmed.is_empty() {
        return None;
    }
    let stripped = strip_korean_particle(trimmed);
    let canonical = match stripped.to_ascii_lowercase().as_str() {
        value
            if value.contains("json")
                || value.contains("code")
                || value.contains("diff")
                || value.contains("log")
                || value.contains("test") =>
        {
            if value.contains('/') {
                "JSON/code/diff/log/test".to_string()
            } else {
                stripped.to_ascii_uppercase()
            }
        }
        "cli" => "CLI".to_string(),
        "tui" => "TUI".to_string(),
        "ux" => "UX".to_string(),
        "codexplain" => "Codexplain".to_string(),
        "renderer" | "renderers" => "Renderer".to_string(),
        "selector" => "Selector".to_string(),
        "policy" => "Policy".to_string(),
        "profile" => "Profile".to_string(),
        "gateway" => "Gateway".to_string(),
        "runner" => "Runner".to_string(),
        "hook" | "hooks" => "hook".to_string(),
        _ => stripped,
    };
    if canonical.chars().count() <= 1 && canonical != "안" {
        None
    } else {
        Some(canonical)
    }
}

fn markdown_highlight_panel(items: &[(String, &'static str)]) -> String {
    let mut out = String::from("**Codexplain highlights**: ");
    let parts = select_chat_highlights(items)
        .iter()
        .map(|(label, role)| match *role {
            "success" => format!("**[OK]** {}", escape_markdown_token(label)),
            "danger" => format!("**[RISK]** {}", escape_markdown_token(label)),
            "warning" => format!("**[WARN]** {}", escape_markdown_token(label)),
            "heading" => format!("**[KEY]** {}", escape_markdown_token(label)),
            "command" | "path" | "artifact" => format!("**[REF]** `{}`", label.replace('`', "\\`")),
            _ => escape_markdown_token(label),
        })
        .collect::<Vec<_>>();
    out.push_str(&parts.join(" · "));
    out
}

fn select_chat_highlights(items: &[(String, &'static str)]) -> Vec<(String, &'static str)> {
    let mut selected = Vec::new();
    let mut heading_count = 0;
    let mut warning_count = 0;
    let mut danger_count = 0;
    let mut success_count = 0;
    let mut utility_count = 0;
    let mut artifact_count = 0;

    for (label, role) in items {
        if selected.len() >= 6 {
            break;
        }
        let allowed = match *role {
            "danger" if danger_count < 1 => {
                danger_count += 1;
                true
            }
            "warning" if warning_count < 2 => {
                warning_count += 1;
                true
            }
            "success" if success_count < 1 => {
                success_count += 1;
                true
            }
            "heading" if heading_count < 2 => {
                heading_count += 1;
                true
            }
            "artifact" if artifact_count < 1 => {
                artifact_count += 1;
                true
            }
            "command" | "path" if utility_count < 1 => {
                utility_count += 1;
                true
            }
            _ => false,
        };
        if allowed {
            selected.push((label.clone(), *role));
        }
    }
    selected
}

#[allow(dead_code)]
fn inline_markdown_highlight(value: &str) -> String {
    let plain = strip_ansi(value);
    let mut out = String::new();
    let mut token = String::new();
    for ch in plain.chars() {
        if ch.is_whitespace() || is_box_drawing(ch) {
            flush_markdown_token(&mut out, &mut token);
            out.push(ch);
        } else {
            token.push(ch);
        }
    }
    flush_markdown_token(&mut out, &mut token);
    out
}

fn flush_markdown_token(out: &mut String, token: &mut String) {
    if token.is_empty() {
        return;
    }
    if let Some(role) = highlight_role(token) {
        let escaped = escape_markdown_token(token);
        match role {
            "heading" | "success" | "danger" | "warning" => {
                out.push_str("**");
                out.push_str(&escaped);
                out.push_str("**");
            }
            "command" | "path" | "artifact" => {
                out.push('`');
                out.push_str(&token.replace('`', "\\`"));
                out.push('`');
            }
            _ => out.push_str(&escaped),
        }
    } else {
        out.push_str(token);
    }
    token.clear();
}

fn escape_markdown_token(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
}

fn is_box_drawing(ch: char) -> bool {
    matches!(
        ch,
        '┌' | '┬' | '┐' | '│' | '├' | '┼' | '┤' | '└' | '┴' | '┘' | '─' | '▼' | '▶'
    )
}

fn push_html_escaped(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => out.push(ch),
    }
}

fn sgr_style(code: &str) -> Option<&'static str> {
    match code {
        "0" | "" => None,
        "1m" => Some("font-weight: 700"),
        _ => sgr_style_from_params(
            &code
                .split(';')
                .filter_map(|part| part.parse::<u16>().ok())
                .collect::<Vec<_>>(),
        ),
    }
}

fn sgr_style_from_params(params: &[u16]) -> Option<&'static str> {
    let mut bold = false;
    let mut dim = false;
    let mut color = None;
    let mut index = 0;
    while index < params.len() {
        match params[index] {
            0 => return None,
            1 => bold = true,
            2 => dim = true,
            30 => color = Some("#111827"),
            31 => color = Some("#dc2626"),
            32 => color = Some("#16a34a"),
            33 => color = Some("#ca8a04"),
            34 => color = Some("#2563eb"),
            35 => color = Some("#9333ea"),
            36 => color = Some("#0891b2"),
            37 => color = Some("#e5e7eb"),
            90 => color = Some("#6b7280"),
            91 => color = Some("#ef4444"),
            92 => color = Some("#22c55e"),
            93 => color = Some("#eab308"),
            94 => color = Some("#3b82f6"),
            95 => color = Some("#a855f7"),
            96 => color = Some("#06b6d4"),
            97 => color = Some("#f9fafb"),
            38 if params.get(index + 1) == Some(&5) => {
                if let Some(code) = params.get(index + 2).copied() {
                    color = ansi_256_hex(code);
                    index += 2;
                }
            }
            _ => {}
        }
        index += 1;
    }
    match (color, bold, dim) {
        (Some("#0891b2"), false, false) => Some("color: #0891b2"),
        (Some("#2563eb"), true, false) => Some("color: #2563eb; font-weight: 700"),
        (Some("#06b6d4"), false, false) => Some("color: #06b6d4"),
        (Some("#0891b2"), false, true) => Some("color: #0891b2; opacity: 0.72"),
        (Some("#16a34a"), true, false) => Some("color: #16a34a; font-weight: 700"),
        (Some("#ca8a04"), true, false) => Some("color: #ca8a04; font-weight: 700"),
        (Some("#dc2626"), true, false) => Some("color: #dc2626; font-weight: 700"),
        (Some("#ca8a04"), false, false) => Some("color: #ca8a04"),
        (Some("#dc2626"), false, false) => Some("color: #dc2626"),
        (Some("#22c55e"), false, false) => Some("color: #22c55e"),
        (Some("#6b7280"), false, false) => Some("color: #6b7280"),
        (Some("#e5e7eb"), false, false) => Some("color: #e5e7eb"),
        (Some(color), true, false) => match color {
            "#f97316" => Some("color: #f97316; font-weight: 700"),
            "#dc2626" => Some("color: #dc2626; font-weight: 700"),
            "#84cc16" => Some("color: #84cc16; font-weight: 700"),
            "#facc15" => Some("color: #facc15; font-weight: 700"),
            "#a855f7" => Some("color: #a855f7; font-weight: 700"),
            "#e879f9" => Some("color: #e879f9; font-weight: 700"),
            "#f472b6" => Some("color: #f472b6; font-weight: 700"),
            "#60a5fa" => Some("color: #60a5fa; font-weight: 700"),
            "#86efac" => Some("color: #86efac; font-weight: 700"),
            _ => Some("font-weight: 700"),
        },
        (Some(color), false, true) => match color {
            "#f97316" => Some("color: #f97316; opacity: 0.72"),
            "#a855f7" => Some("color: #a855f7; opacity: 0.72"),
            "#60a5fa" => Some("color: #60a5fa; opacity: 0.72"),
            "#f472b6" => Some("color: #f472b6; opacity: 0.72"),
            _ => Some("opacity: 0.72"),
        },
        (Some(color), false, false) => match color {
            "#f97316" => Some("color: #f97316"),
            "#fb923c" => Some("color: #fb923c"),
            "#a855f7" => Some("color: #a855f7"),
            "#e879f9" => Some("color: #e879f9"),
            "#60a5fa" => Some("color: #60a5fa"),
            "#93c5fd" => Some("color: #93c5fd"),
            "#f472b6" => Some("color: #f472b6"),
            "#f9a8d4" => Some("color: #f9a8d4"),
            _ => None,
        },
        (None, true, false) => Some("font-weight: 700"),
        (None, false, true) => Some("opacity: 0.72"),
        _ => None,
    }
}

fn ansi_256_hex(code: u16) -> Option<&'static str> {
    match code {
        67 => Some("#60a5fa"),
        110 => Some("#93c5fd"),
        114 => Some("#86efac"),
        118 | 120 => Some("#84cc16"),
        135 => Some("#a855f7"),
        141 => Some("#a855f7"),
        153 => Some("#93c5fd"),
        167 => Some("#dc2626"),
        179 => Some("#ca8a04"),
        183 => Some("#e879f9"),
        196 | 197 => Some("#dc2626"),
        199 => Some("#db2777"),
        204 => Some("#f472b6"),
        208 => Some("#f97316"),
        211 => Some("#f472b6"),
        214 => Some("#fb923c"),
        218 => Some("#f9a8d4"),
        220 | 222 => Some("#facc15"),
        _ => None,
    }
}

fn role_for(value: &str, fallback: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "tldr"
        | "핵심"
        | "결론"
        | "장점"
        | "pros"
        | "success"
        | "완료"
        | "pass"
        | "gateway"
        | "runner"
        | "profile"
        | "codex 자체 색상" => "success",
        "단점"
        | "위험"
        | "주의"
        | "cons"
        | "risk"
        | "warning"
        | "진행"
        | "남음"
        | "policy"
        | "hook"
        | "외부 후처리 hook"
        | "가능한 우회"
        | "외부 후처리" => "warning",
        "오류" | "실패" | "danger" | "error" | "blocked" | "진짜 통합" => "danger",
        "다음 행동"
        | "선택기"
        | "아키텍처"
        | "추상화"
        | "구현"
        | "selector"
        | "renderer"
        | "lifecycle"
        | "levels"
        | "색상 적용 대상"
        | "색상"
        | "렌더링" => "heading",
        _ => match fallback {
            "heading" => "heading",
            "border" => "border",
            "muted" => "muted",
            _ => "accent",
        },
    }
}

fn role_for_cell(value: &str, fallback: &str, cell_index: usize) -> &'static str {
    let direct = role_for(value, fallback);
    if direct != "accent" {
        return direct;
    }
    match (fallback, cell_index) {
        ("heading", _) => "heading",
        (_, 0) => "heading",
        (_, 1) => "accent",
        (_, 2) => "warning",
        (_, _) => "muted",
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
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    let mut word = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            word.push(ch);
            append_ansi_escape(&mut chars, &mut word);
            continue;
        }
        if ch == '\n' {
            emit_wrapped_word(
                &mut lines,
                &mut current,
                &mut current_width,
                &mut word,
                width,
            );
            lines.push(current.trim_end().to_string());
            current.clear();
            current_width = 0;
            continue;
        }
        if ch.is_whitespace() {
            emit_wrapped_word(
                &mut lines,
                &mut current,
                &mut current_width,
                &mut word,
                width,
            );
            continue;
        }
        word.push(ch);
    }

    emit_wrapped_word(
        &mut lines,
        &mut current,
        &mut current_width,
        &mut word,
        width,
    );
    if !current.is_empty() {
        lines.push(current.trim_end().to_string());
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn emit_wrapped_word(
    lines: &mut Vec<String>,
    current: &mut String,
    current_width: &mut usize,
    word: &mut String,
    width: usize,
) {
    if word.is_empty() {
        return;
    }

    let word_width = visible_width(word);
    if word_width > width {
        if !current.is_empty() {
            lines.push(current.trim_end().to_string());
            current.clear();
            *current_width = 0;
        }
        let hard_lines = wrap_text_hard(word, width);
        for (index, line) in hard_lines.iter().enumerate() {
            if index + 1 == hard_lines.len() {
                current.push_str(line);
                *current_width = visible_width(current);
            } else {
                lines.push(line.clone());
            }
        }
        word.clear();
        return;
    }

    let separator_width = if current.is_empty() { 0 } else { 1 };
    if *current_width + separator_width + word_width > width {
        lines.push(current.trim_end().to_string());
        current.clear();
        *current_width = 0;
    }
    if !current.is_empty() {
        current.push(' ');
        *current_width += 1;
    }
    current.push_str(word);
    *current_width += word_width;
    word.clear();
}

fn wrap_text_hard(text: &str, width: usize) -> Vec<String> {
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
        if current_width > 0 && current_width + ch_width > width {
            lines.push(current.trim_end().to_string());
            current.clear();
            current_width = 0;
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
    let normalized = text.replace('\n', " ");
    let mut chars = normalized.chars().peekable();
    while let Some(ch) = chars.next() {
        sentence.push(ch);
        if is_sentence_boundary(ch, chars.peek().copied()) {
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

fn is_sentence_boundary(ch: char, next: Option<char>) -> bool {
    match ch {
        '!' | '?' | '。' | '！' | '？' => true,
        '.' => !next.is_some_and(|value| value.is_ascii_alphanumeric()),
        _ => false,
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

fn wide_divider_table(
    headers: &[&str; 2],
    rows: &[Vec<String>],
    theme: Theme,
    width: usize,
) -> String {
    let width = width.max(32);
    let left_widest = rows
        .iter()
        .filter_map(|row| row.first())
        .map(|value| visible_width(value))
        .chain(headers.iter().take(1).map(|value| visible_width(value)))
        .max()
        .unwrap_or(8);
    let left_width = left_widest
        .max(8)
        .min(40)
        .min(width.saturating_mul(45) / 100)
        .max(8);
    let right_width = width.saturating_sub(left_width + 3).max(12);
    let heavy = format!(" {}  {}", "━".repeat(left_width), "━".repeat(right_width));
    let thin = format!(" {}  {}", "─".repeat(left_width), "─".repeat(right_width));
    let mut lines = Vec::new();
    lines.push(format!(
        " {}  {}",
        color(theme, "heading", &pad(headers[0], left_width)),
        color(theme, "heading", &pad(headers[1], right_width))
    ));
    lines.push(color(theme, "border", &heavy));

    for (index, row) in rows.iter().enumerate() {
        let left = row.first().map(String::as_str).unwrap_or("");
        let right = row.get(1).map(String::as_str).unwrap_or("");
        let left_lines = wrap_text(left, left_width);
        let right_lines = wrap_text(right, right_width);
        let row_height = left_lines.len().max(right_lines.len()).max(1);
        for line_index in 0..row_height {
            let left_line = left_lines.get(line_index).map(String::as_str).unwrap_or("");
            let right_line = right_lines
                .get(line_index)
                .map(String::as_str)
                .unwrap_or("");
            lines.push(format!(
                " {}  {}",
                color(
                    theme,
                    role_for(left_line, "heading"),
                    &pad(left_line, left_width)
                ),
                color(
                    theme,
                    role_for(right_line, "accent"),
                    &pad(right_line, right_width)
                )
            ));
        }
        if index + 1 < rows.len() {
            lines.push(color(theme, "border", &thin));
        }
    }
    lines.join("\n")
}

#[allow(dead_code)]
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
            let role = role_for_cell(cell, default_role, cell_index);
            let cell_text = if cell
                .split_whitespace()
                .any(|token| highlight_role(token).is_some())
            {
                let highlighted = semantic_highlight(theme, cell, role);
                layout.padded_cell(&highlighted, *width)
            } else {
                color(theme, role, &layout.padded_cell(cell, *width))
            };
            line = line.text(cell_text);
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
            FlowStep::new("Prompt Input"),
            FlowStep::new("Project Shim"),
            FlowStep::new("Codex Runner"),
            FlowStep::with_branches(
                "Strict Policy",
                [
                    "artifact passthrough".to_string(),
                    "explanation shaping".to_string(),
                ],
            ),
            FlowStep::new("Profile Resolver"),
            FlowStep::new("Renderer Selector"),
            FlowStep::new("ANSI Terminal Output"),
        ],
        max_width,
    );
    render_flow_diagram(&diagram, frame, theme)
}

fn architecture_panels(profile: &Profile, summary: &str, width: usize) -> String {
    let flow_panel = codexplain_flow(profile.frame, profile.theme, width);
    let capability_map = render_expansion_diagram(
        &[
            "Capability Boundary",
            "Runtime Path",
            "Safety Gate",
            "Renderer UX",
            "Lifecycle Control",
        ],
        profile.frame,
        profile.theme,
        width,
    );
    let rows = architecture_showcase_rows(summary, profile);
    let showcase = wide_divider_table(&["영역", "역할"], &rows, profile.theme, width.max(50));
    format!(
        "{}\n\n{}",
        architecture_tldr(profile, summary, width),
        format!("{flow_panel}\n\n{capability_map}\n\n{showcase}")
    )
}

fn architecture_tldr(profile: &Profile, summary: &str, width: usize) -> String {
    let label = color(profile.theme, "heading", "• TLDR");
    let body = if summary.trim().is_empty() {
        "Codexplain은 Codex의 추론 계층이 아니라 설명 표현 계층을 프로젝트 로컬에서 제어합니다."
            .to_string()
    } else {
        format!(
            "Codexplain은 Codex 응답 위에 설명 UX 레이어를 얹습니다. {}",
            compact(summary, 1)
        )
    };
    let content_width = width.saturating_sub(2).max(24);
    let body_lines = wrap_text(&body, content_width)
        .into_iter()
        .map(|line| {
            format!(
                "  {}",
                color(profile.theme, role_for(&line, "accent"), &line)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{label}\n{body_lines}")
}

fn architecture_showcase_rows(summary: &str, profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            "개념".to_string(),
            "Codex의 생각을 바꾸는 것이 아니라 답변의 표현층을 구조화합니다.".to_string(),
        ],
        vec![
            "보호".to_string(),
            "JSON, code, diff, log, test output은 strict policy가 원문 그대로 보존합니다."
                .to_string(),
        ],
        vec![
            "선택".to_string(),
            "prompt 신호와 profile을 보고 TLDR, 흐름도, 표, pros/cons, progress를 조합합니다."
                .to_string(),
        ],
        vec![
            "표현".to_string(),
            "Terminal renderer가 Unicode/ASCII layout, ANSI highlight, wrapping을 담당합니다."
                .to_string(),
        ],
        vec![
            "연결".to_string(),
            "project-local shim과 TUI adapter가 켜기/끄기 가능한 연결 계층을 제공합니다."
                .to_string(),
        ],
    ];
    if profile.abstraction_level == "concrete" || profile.architecture_depth == "internals" {
        rows.extend(architecture_layer_rows(profile));
    }
    if profile.explanation_depth != "light" {
        rows.push(vec![
            "Levels".to_string(),
            "Level Controls: explanation-depth, architecture-depth, abstraction-level을 light/standard/deep 계열 3단계로 조절합니다."
                .to_string(),
        ]);
    }
    if !summary.trim().is_empty() {
        rows.insert(0, vec!["요약".to_string(), compact(summary, 1)]);
    }
    rows
}

fn render_expansion_diagram(labels: &[&str], frame: Frame, theme: Theme, width: usize) -> String {
    let stages = labels
        .iter()
        .map(|label| FlowStep::new(*label))
        .collect::<Vec<_>>();
    render_flow_diagram(&FlowDiagram::new(stages, width), frame, theme)
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
    vec![flow_connector_rail(layout, theme)]
}

fn flow_connector_rail(layout: &FlowLayout, theme: Theme) -> String {
    let width = layout.content_width + layout.spec.padding.total() + 2;
    let arrow = layout.spec.separators.arrow_down;
    let center = width / 2;
    format!(
        "{}{}{}",
        color(
            theme,
            "border",
            &layout.spec.border.horizontal.to_string().repeat(center)
        ),
        color(theme, "heading", &arrow.to_string()),
        color(
            theme,
            "border",
            &layout
                .spec
                .border
                .horizontal
                .to_string()
                .repeat(width.saturating_sub(center + 1))
        )
    )
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
    let branch_width = layout
        .max_width
        .saturating_sub(visible_width(&spine) + 4)
        .min(layout.content_width)
        .max(8);

    for (index, branch) in branches.iter().enumerate() {
        let connector = branch_connector(frame, index + 1 == branch_count);
        for (line_index, line) in wrap_text(branch, branch_width).iter().enumerate() {
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
                        "{marker} {}",
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

fn looks_like_codex_activity_transcript(response: &str) -> bool {
    response.contains("• Explored")
        || response.contains("• Ran")
        || response.contains("└ Read")
        || response.contains("└ Search")
        || response.contains("ctrl + t to view transcript")
}

fn macro_progress_rows(response: &str) -> Option<Vec<Vec<String>>> {
    if !looks_like_codex_activity_transcript(response) {
        return None;
    }

    let explored = response.matches("Explored").count() + response.matches("Read ").count();
    let commands = response.matches("• Ran").count() + response.matches("\n• Ran").count();
    let searches = response.matches("Search ").count() + response.matches("rg ").count();
    let config = response.matches(".codexplain").count()
        + response.matches("config.json").count()
        + response.matches("AGENTS.md").count();

    let mut rows = vec![vec![
        "🧭 탐색".to_string(),
        "기능 경계와 책임 흐름을 훑어 작업 지도를 만든 단계".to_string(),
        format!("{explored}개 읽기/탐색 신호"),
    ]];

    if searches > 0 {
        rows.push(vec![
            "🔎 검색".to_string(),
            "관련 심볼, 설정, 렌더러 위치를 좁힌 단계".to_string(),
            format!("{searches}개 검색 신호"),
        ]);
    }

    rows.push(vec![
        "⚙️ 실행".to_string(),
        "명령을 실행해 실제 기능/설정/출력 상태를 확인한 단계".to_string(),
        format!("{commands}개 command 신호"),
    ]);

    if config > 0 {
        rows.push(vec![
            "🎛️ 설정".to_string(),
            "프로젝트 로컬 Codexplain adapter와 profile/config를 확인한 단계".to_string(),
            format!("{config}개 설정 신호"),
        ]);
    }

    rows.push(vec![
        "✅ 결론".to_string(),
        "마이크로 로그 대신 현재 목표, 근거, 다음 액션으로 보고할 수 있음".to_string(),
        "macro-progress UX 적용".to_string(),
    ]);

    Some(rows)
}

fn shared_workflow_fields_present(fields: &SharedWorkflowProgressFields) -> bool {
    fields.workflow_type.is_some()
        || fields.phase_label.is_some()
        || fields.progress_percent.is_some()
        || fields.completed_step.is_some()
        || fields.current_step.is_some()
        || fields.next_step.is_some()
        || fields.next_action.is_some()
        || !fields.evidence.is_empty()
        || fields.trigger_source.is_some()
}

fn workflow_progress_context(prompt: &str, response: &str) -> bool {
    let fields = shared_workflow_progress_fields(prompt, response);
    if shared_workflow_fields_present(&fields) {
        return true;
    }

    let text = format!("{prompt} {response}").to_ascii_lowercase();
    text.contains("workflow")
        || text.contains("harness")
        || text.contains("progress block")
        || prompt.contains("워크플로우")
        || prompt.contains("하네스")
        || prompt.contains("사용자 정의 워크플로우")
}

fn workflow_progress_rows(profile: &Profile, model: &WorkflowProgressBlock) -> Vec<Vec<String>> {
    let evidence = if model.evidence.is_empty() {
        "증거 없음".to_string()
    } else {
        model.evidence.join("; ")
    };
    vec![
        vec![
            emoji_label(profile, "architecture", "유형"),
            model.workflow_type.clone(),
        ],
        vec![
            emoji_label(profile, "time", "단계"),
            model.phase_label.clone(),
        ],
        vec![
            emoji_label(profile, "time", "진척"),
            format!("매크로 {}%", model.progress_percent),
        ],
        vec![
            emoji_label(profile, "time", "막대"),
            model.progress_bar.clone(),
        ],
        vec![
            emoji_label(profile, "success", "완료"),
            model.completed_step.clone(),
        ],
        vec![
            emoji_label(profile, "inspect", "현재"),
            model.current_step.clone(),
        ],
        vec![
            emoji_label(profile, "next", "다음"),
            model.next_step.clone(),
        ],
        vec![
            emoji_label(profile, "fix", "행동"),
            model.next_action.clone(),
        ],
        vec![emoji_label(profile, "inspect", "근거"), evidence],
        vec![
            emoji_label(profile, "note", "트리거"),
            model.trigger_source.clone(),
        ],
    ]
}

fn progress_report(
    profile: &Profile,
    prompt: &str,
    response: &str,
    summary: &str,
    width: usize,
) -> String {
    if workflow_progress_context(prompt, response) {
        let bar_width = width.saturating_sub(18).min(34).max(12);
        let model = workflow_progress_model_with_bar(
            prompt,
            response,
            summary,
            bar_width,
            profile.frame,
            profile.theme,
        );
        let status = progress_label(model.progress_percent);
        let headline = format!(
            "{}{}",
            color(
                profile.theme,
                "heading",
                &format!("{}: ", emoji_label(profile, "time", "진행상황")),
            ),
            color(profile.theme, role_for(status, "accent"), status)
        );
        return format!(
            "{headline}\n{}\n\n{}",
            model.progress_bar,
            table(
                &["항목", "값"],
                &workflow_progress_rows(profile, &model),
                profile.frame,
                profile.theme,
                true,
                width,
            )
        );
    }

    let percent = progress_percent(response);
    let status = progress_label(percent);
    let bar_width = width.saturating_sub(18).min(36).max(12);
    let headline = format!(
        "{}{}",
        color(
            profile.theme,
            "heading",
            &format!("{}: ", emoji_label(profile, "time", "진행상황")),
        ),
        color(profile.theme, role_for(status, "accent"), status)
    );
    let bar = render_progress_bar(percent, bar_width, profile.frame, profile.theme);
    let detail = if let Some(rows) = macro_progress_rows(response) {
        table(
            &["단계", "전체 의미", "근거"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            width,
        )
    } else {
        let rows = vec![
            vec![emoji_label(profile, "inspect", "현재"), compact(summary, 1)],
            vec![
                emoji_label(profile, "time", "진척"),
                format!("{status} · {percent}%"),
            ],
            vec![
                emoji_label(profile, "next", "다음 행동"),
                "막힌 지점, 실패 로그, 남은 검증을 한 줄로 확인합니다.".to_string(),
            ],
        ];
        table(
            &["항목", "보고"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            width,
        )
    };
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
    let marker = ux_emoji_for_role(profile, role);
    format!(
        "{} {} {}",
        marker,
        color(profile.theme, role, &format!("[{label}]")),
        color(profile.theme, "accent", progress_label(percent))
    )
}

fn checklist(profile: &Profile, summary: &str, width: usize) -> String {
    let current = if looks_like_codex_activity_transcript(summary) {
        "상세 transcript는 macro-progress 단계로 압축됨".to_string()
    } else {
        compact(summary, 1)
    };
    let rows = vec![
        vec![
            emoji_label(profile, "success", "완료"),
            "검증 가능한 사실과 출력 근거를 먼저 확인".to_string(),
        ],
        vec![emoji_label(profile, "inspect", "진행"), current],
        vec![
            emoji_label(profile, "next", "남음"),
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
        vec![emoji_label(profile, "warning", "위험"), risk.to_string()],
        vec![
            emoji_label(profile, "fix", "대응"),
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
        color(
            profile.theme,
            "heading",
            &format!("{} 확신도", ux_emoji_for_role(profile, "inspect"))
        ),
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

fn next_action_footer(profile: &Profile, width: usize) -> String {
    let label = format!("{} 다음 행동:", ux_emoji_for_role(profile, "next"));
    let body = "검증 결과, 위험, 남은 항목 중 하나만 선택해 바로 실행합니다.";
    let label_width = visible_width(&label);
    let body_width = width.saturating_sub(label_width + 1).max(12);
    let body_lines = wrap_text(body, body_width);
    let indent = " ".repeat(label_width + 1);
    let mut output = String::new();

    for (index, line) in body_lines.iter().enumerate() {
        if index == 0 {
            output.push_str(&format!(
                "{} {}",
                color(profile.theme, "heading", &label),
                color(profile.theme, "accent", line)
            ));
        } else {
            output.push('\n');
            output.push_str(&format!(
                "{}{}",
                indent,
                color(profile.theme, "accent", line)
            ));
        }
    }

    output
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
        color(
            profile.theme,
            "heading",
            &format!("{} ETA:", ux_emoji_for_role(profile, "time"))
        ),
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
        &[vec![
            format!("{} 중요", ux_emoji_for_role(profile, "warning")),
            message.to_string(),
        ]],
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn ux_emoji_for_role(profile: &Profile, role: &str) -> &'static str {
    if !profile.emoji_cues {
        return "•";
    }
    match role {
        "success" => "✅",
        "danger" => "🚨",
        "warning" => "⚠️",
        "next" => "🚀",
        "time" => "⏱",
        "inspect" => "🔎",
        "fix" | "action" | "command" => "🛠️",
        "architecture" | "overview" | "heading" => "🧭",
        "docs" | "research" => "📚",
        "settings" => "🎛️",
        "note" => "📌",
        _ => "•",
    }
}

fn emoji_label(profile: &Profile, role: &str, label: &str) -> String {
    let emoji = ux_emoji_for_role(profile, role);
    if emoji == "•" {
        label.to_string()
    } else {
        format!("{emoji} {label}")
    }
}

fn notion_toggle(profile: &Profile, summary: &str, width: usize) -> String {
    let rows = vec![vec![
        "▸ 핵심 접기".to_string(),
        compact(summary, 2),
        "펼침".to_string(),
    ]];
    table(
        &["Toggle", "요약", "용도"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn notion_quote(profile: &Profile, summary: &str, width: usize) -> String {
    let quote = compact(summary, 1);
    let prefix_width = 2;
    let available = width.saturating_sub(prefix_width).max(1);
    wrap_text(&quote, available)
        .into_iter()
        .map(|line| {
            format!(
                "{} {}",
                color(profile.theme, "border", "│"),
                color(profile.theme, "accent", &line)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn notion_divider(profile: &Profile, width: usize) -> String {
    color(profile.theme, "border", &"─".repeat(width.clamp(1, 120)))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkflowProgressBlock {
    workflow_type: String,
    phase_label: String,
    progress_percent: usize,
    progress_bar: String,
    completed_step: String,
    current_step: String,
    next_step: String,
    next_action: String,
    evidence: Vec<String>,
    trigger_source: String,
    strict_artifact_bypass: bool,
}

#[cfg(test)]
const DEFAULT_WORKFLOW_PROGRESS_BAR_WIDTH: usize = 24;
const WORKFLOW_PROGRESS_HINT_ENVS: &[&str] = &[
    "CODEXPLAIN_WORKFLOW_PROGRESS",
    "CODEXPLAIN_WORKFLOW_PROGRESS_HINTS",
    "CLAUDEX_WORKFLOW_PROGRESS",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SharedWorkflowProgressFields {
    workflow_type: Option<String>,
    phase_label: Option<String>,
    progress_percent: Option<usize>,
    completed_step: Option<String>,
    current_step: Option<String>,
    next_step: Option<String>,
    next_action: Option<String>,
    evidence: Vec<String>,
    trigger_source: Option<String>,
}

impl SharedWorkflowProgressFields {
    fn merge_missing(&mut self, other: Self) {
        self.workflow_type = self.workflow_type.take().or(other.workflow_type);
        self.phase_label = self.phase_label.take().or(other.phase_label);
        self.progress_percent = self.progress_percent.or(other.progress_percent);
        self.completed_step = self.completed_step.take().or(other.completed_step);
        self.current_step = self.current_step.take().or(other.current_step);
        self.next_step = self.next_step.take().or(other.next_step);
        self.next_action = self.next_action.take().or(other.next_action);
        if self.evidence.is_empty() {
            self.evidence = other.evidence;
        }
        self.trigger_source = self.trigger_source.take().or(other.trigger_source);
    }
}

fn explicit_percent(text: &str) -> Option<usize> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_ascii_digit() && ch != '%' && ch != '/');
        if let Some(value) = cleaned.strip_suffix('%') {
            if let Ok(percent) = value.parse::<usize>() {
                return Some(percent.min(100));
            }
        }
        if let Some((done, total)) = cleaned.split_once('/') {
            if let (Ok(done), Ok(total)) = (done.parse::<usize>(), total.parse::<usize>()) {
                if total > 0 {
                    return Some(((done * 100) / total).min(100));
                }
            }
        }
    }
    None
}

fn workflow_field_key(key: &str) -> String {
    key.trim()
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-')
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
        .map(|ch| ch.to_ascii_lowercase())
        .collect::<String>()
        .replace(['-', '_'], "")
}

fn workflow_field_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '"' | '\'' | '`' | '[' | ']' | '{' | '}' | ',' | ';' | ' '
            )
        })
        .trim()
        .to_string()
}

fn workflow_field_segments(text: &str) -> Vec<&str> {
    text.lines()
        .flat_map(|line| line.split([',', ';']))
        .collect()
}

fn parse_progress_percent_value(value: &str) -> Option<usize> {
    value
        .trim()
        .trim_end_matches('%')
        .parse::<usize>()
        .ok()
        .map(|percent| percent.min(100))
        .or_else(|| explicit_percent(value))
}

fn normalize_workflow_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "dev" | "development" | "implementation" | "개발" => "development".to_string(),
        "harness" | "test-harness" | "testharness" | "하네스" => "harness".to_string(),
        "custom" | "user-defined" | "userdefined" | "사용자정의" => "custom".to_string(),
        _ => value.trim().to_string(),
    }
}

fn parse_shared_workflow_progress_fields(text: &str) -> SharedWorkflowProgressFields {
    let mut fields = SharedWorkflowProgressFields::default();
    for segment in workflow_field_segments(text) {
        let Some((raw_key, raw_value)) =
            segment.split_once(':').or_else(|| segment.split_once('='))
        else {
            continue;
        };
        let key = workflow_field_key(raw_key);
        let value = workflow_field_value(raw_value);
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "workflowtype" | "workflow" | "type" => {
                fields
                    .workflow_type
                    .get_or_insert_with(|| normalize_workflow_type(&value));
            }
            "phaselabel" | "currentphase" | "phase" => {
                fields.phase_label.get_or_insert(value);
            }
            "progresspercent" | "percent" | "progress" => {
                if fields.progress_percent.is_none() {
                    fields.progress_percent = parse_progress_percent_value(&value);
                }
            }
            "completedstep" | "completed" | "done" => {
                fields.completed_step.get_or_insert(value);
            }
            "currentstep" | "current" | "active" => {
                fields.current_step.get_or_insert(value);
            }
            "nextstep" | "next" => {
                fields.next_step.get_or_insert(value);
            }
            "nextaction" | "action" => {
                fields.next_action.get_or_insert(value);
            }
            "evidence" | "proof" | "validationevidence" => {
                fields.evidence.extend(
                    value
                        .split('|')
                        .map(str::trim)
                        .filter(|item| !item.is_empty())
                        .map(ToString::to_string),
                );
            }
            "triggersource" | "trigger" | "source" => {
                fields.trigger_source.get_or_insert(value);
            }
            _ => {}
        }
    }
    fields
}

fn planner_workflow_progress_fields() -> SharedWorkflowProgressFields {
    let mut fields = SharedWorkflowProgressFields::default();
    for key in WORKFLOW_PROGRESS_HINT_ENVS {
        if let Ok(value) = env::var(key) {
            fields.merge_missing(parse_shared_workflow_progress_fields(&value));
        }
    }
    fields
}

fn shared_workflow_progress_fields(prompt: &str, response: &str) -> SharedWorkflowProgressFields {
    let mut fields = parse_shared_workflow_progress_fields(prompt);
    fields.merge_missing(planner_workflow_progress_fields());
    fields.merge_missing(parse_shared_workflow_progress_fields(response));
    fields
}

fn workflow_type(prompt: &str, response: &str) -> String {
    let text = format!("{prompt} {response}").to_ascii_lowercase();
    if text.contains("harness") || prompt.contains("하네스") {
        "harness".to_string()
    } else if text.contains("custom") || prompt.contains("사용자 정의") {
        "custom".to_string()
    } else {
        "development".to_string()
    }
}

fn workflow_phase(prompt: &str, response: &str) -> String {
    let text = format!("{prompt} {response}").to_ascii_lowercase();
    if text.contains("release") || response.contains("릴리즈") || response.contains("배포") {
        "release/report".to_string()
    } else if text.contains("test")
        || text.contains("validat")
        || response.contains("검증")
        || response.contains("테스트")
        || response.contains("통과")
    {
        "validation".to_string()
    } else if text.contains("implement")
        || text.contains("code")
        || response.contains("구현")
        || response.contains("수정")
        || response.contains("개발")
    {
        "implementation".to_string()
    } else {
        "planning".to_string()
    }
}

fn phase_default_percent(phase: &str) -> usize {
    match phase {
        "planning" => 25,
        "implementation" => 50,
        "validation" => 75,
        "release/report" => 90,
        _ => 50,
    }
}

fn workflow_percent(
    prompt: &str,
    response: &str,
    phase: &str,
    shared: &SharedWorkflowProgressFields,
) -> usize {
    shared
        .progress_percent
        .or_else(|| explicit_percent(prompt))
        .or_else(|| {
            WORKFLOW_PROGRESS_HINT_ENVS
                .iter()
                .filter_map(|key| env::var(key).ok())
                .find_map(|value| explicit_percent(&value))
        })
        .or_else(|| explicit_percent(response))
        .unwrap_or_else(|| phase_default_percent(phase))
}

fn workflow_evidence(response: &str, summary: &str) -> Vec<String> {
    let lower = response.to_ascii_lowercase();
    let evidence = if looks_like_codex_activity_transcript(response) {
        "macro transcript evidence: phase-level activity signal".to_string()
    } else if lower.contains("cargo test")
        || response.contains("테스트")
        || response.contains("통과")
    {
        "validation evidence: test/pass signal".to_string()
    } else if lower.contains("quality-check")
        || lower.contains("build")
        || response.contains("검증")
    {
        "validation evidence: build/quality signal".to_string()
    } else if lower.contains("harness") || response.contains("하네스") {
        "harness evidence: workflow runner signal".to_string()
    } else if lower.contains("file") || lower.contains("renderer") || response.contains("구현") {
        "implementation evidence: changed file or renderer signal".to_string()
    } else {
        compact(summary, 1)
    };
    vec![evidence]
}

fn workflow_trigger(prompt: &str) -> String {
    let lower = prompt.to_ascii_lowercase();
    if lower.contains("workflow") || prompt.contains("워크플로우") {
        "explicit workflow prompt".to_string()
    } else if lower.contains("harness") || prompt.contains("하네스") {
        "explicit harness prompt".to_string()
    } else if lower.contains("progress") || prompt.contains("진행") {
        "progress signal".to_string()
    } else {
        "inferred workflow signal".to_string()
    }
}

fn workflow_progress_model_with_bar(
    prompt: &str,
    response: &str,
    summary: &str,
    bar_width: usize,
    frame: Frame,
    theme: Theme,
) -> WorkflowProgressBlock {
    let shared = shared_workflow_progress_fields(prompt, response);
    let phase = shared
        .phase_label
        .clone()
        .unwrap_or_else(|| workflow_phase(prompt, response));
    let percent = workflow_percent(prompt, response, &phase, &shared);
    let status = progress_label(percent);
    let default_completed_step = if percent >= 75 {
        "요구사항 고정과 구현 확인".to_string()
    } else {
        "목표/범위 확인".to_string()
    };
    let default_next_step = if percent >= 90 {
        "릴리즈/보고와 회귀 확인".to_string()
    } else if percent >= 75 {
        "품질 게이트와 회귀 테스트".to_string()
    } else {
        "구현 후 검증 증거 확보".to_string()
    };
    WorkflowProgressBlock {
        workflow_type: shared
            .workflow_type
            .clone()
            .unwrap_or_else(|| workflow_type(prompt, response)),
        phase_label: phase,
        progress_percent: percent,
        progress_bar: render_progress_bar(percent, bar_width, frame, theme),
        completed_step: shared.completed_step.unwrap_or(default_completed_step),
        current_step: shared
            .current_step
            .unwrap_or_else(|| format!("{status} 단계에서 증거를 수집 중")),
        next_step: shared
            .next_step
            .unwrap_or_else(|| default_next_step.clone()),
        next_action: shared.next_action.unwrap_or(default_next_step),
        evidence: if shared.evidence.is_empty() {
            workflow_evidence(response, summary)
        } else {
            shared.evidence
        },
        trigger_source: shared
            .trigger_source
            .unwrap_or_else(|| workflow_trigger(prompt)),
        strict_artifact_bypass: should_back_off(prompt, response),
    }
}

#[cfg(test)]
fn workflow_progress_model(prompt: &str, response: &str, summary: &str) -> WorkflowProgressBlock {
    workflow_progress_model_with_bar(
        prompt,
        response,
        summary,
        DEFAULT_WORKFLOW_PROGRESS_BAR_WIDTH,
        Frame::Unicode,
        Theme::None,
    )
}

fn workflow_progress_block(
    profile: &Profile,
    prompt: &str,
    response: &str,
    summary: &str,
    width: usize,
) -> String {
    let bar_width = width.saturating_sub(18).min(34).max(12);
    let model = workflow_progress_model_with_bar(
        prompt,
        response,
        summary,
        bar_width,
        profile.frame,
        profile.theme,
    );
    let headline = format!(
        "{} {}",
        color(
            profile.theme,
            "heading",
            &emoji_label(profile, "time", "Workflow")
        ),
        model.progress_bar
    );
    let rows = workflow_progress_rows(profile, &model);
    format!(
        "{headline}\n{}",
        table(
            &["항목", "값"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            width,
        )
    )
}

fn ux_component_output(
    component: UxComponent,
    profile: &Profile,
    prompt: &str,
    response: &str,
    summary: &str,
    width: usize,
) -> String {
    match component {
        UxComponent::StatusBadge => status_badge(profile, response),
        UxComponent::WorkflowProgress => {
            workflow_progress_block(profile, prompt, response, summary, width)
        }
        UxComponent::Toggle => notion_toggle(profile, summary, width),
        UxComponent::Quote => notion_quote(profile, summary, width),
        UxComponent::Divider => notion_divider(profile, width),
        UxComponent::Checklist => checklist(profile, summary, width),
        UxComponent::RiskPanel => risk_panel(profile, response, width),
        UxComponent::ConfidenceMeter => confidence_meter(profile, response, width),
        UxComponent::DiffSummary => diff_summary_card(profile, summary, width),
        UxComponent::DecisionMatrix => decision_matrix(profile, width),
        UxComponent::NextAction => next_action_footer(profile, width),
        UxComponent::EtaStrip => eta_strip(profile, response),
        UxComponent::AttentionCallout => attention_callout(profile, response, width),
    }
}

fn ux_component_sections(
    profile: &Profile,
    prompt: &str,
    response: &str,
    summary: &str,
    width: usize,
    components: &[UxComponent],
) -> Vec<String> {
    components
        .iter()
        .copied()
        .map(|component| ux_component_output(component, profile, prompt, response, summary, width))
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

#[cfg(test)]
fn pros_cons(profile: &Profile) -> String {
    pros_cons_for_width(profile, 120)
}

fn pros_cons_for_width(profile: &Profile, width: usize) -> String {
    let table = pros_cons_table(width);
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

fn cause_effect_report(profile: &Profile, response: &str, summary: &str, width: usize) -> String {
    let rows = cause_effect_rows(response, summary);
    table(
        &["원인", "결과", "대응"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn cause_effect_rows(response: &str, summary: &str) -> Vec<Vec<String>> {
    let text = if response.trim().is_empty() {
        summary
    } else {
        response
    };
    let mut rows = Vec::new();
    for clause in split_claim_clauses(text) {
        let clause = clause.trim();
        if clause.is_empty() || is_index_intro_clause(clause) {
            continue;
        }
        let (cause, effect) = split_causal_clause(clause)
            .unwrap_or_else(|| (compact(clause, 1), infer_effect_from_clause(clause)));
        rows.push(vec![
            prewrap_table_cell(&cause, 30),
            prewrap_table_cell(&effect, 26),
            prewrap_table_cell(&infer_action_from_clause(clause), 32),
        ]);
        if rows.len() >= 4 {
            break;
        }
    }
    if rows.is_empty() {
        rows.push(vec![
            "입력 정보가 부족함".to_string(),
            compact(summary, 1),
            "원인, 영향, 원하는 대응 기준을 한 줄 더 제공합니다.".to_string(),
        ]);
    }
    rows
}

fn prewrap_table_cell(value: &str, width: usize) -> String {
    wrap_text(value, width).join("\n")
}

fn split_causal_clause(clause: &str) -> Option<(String, String)> {
    let lower_clause = clause.to_ascii_lowercase();
    for marker in [
        " 때문에 ",
        " 때문에",
        "해서 ",
        "어서 ",
        "라서 ",
        "하여 ",
        "그래서 ",
        "따라서 ",
        "so ",
        "therefore ",
        "because ",
        "causes ",
        "leads to ",
        "results in ",
        "->",
        "→",
    ] {
        if let Some(index) = lower_clause.find(marker.to_ascii_lowercase().as_str()) {
            let cause = clause[..index].trim().trim_matches(highlight_trim_char);
            let effect = clause[index + marker.len()..]
                .trim()
                .trim_matches(highlight_trim_char);
            if !cause.is_empty() && !effect.is_empty() {
                return Some((cause.to_string(), effect.to_string()));
            }
        }
    }
    None
}

fn infer_effect_from_clause(clause: &str) -> String {
    if clause.contains("깨") || clause.contains("overflow") || clause.contains("벗어나") {
        "표/다이어그램의 가독성이 떨어지고 신뢰도가 낮아집니다.".to_string()
    } else if clause.contains("색") || clause.to_ascii_lowercase().contains("color") {
        "중요 정보의 attention 신호가 약해집니다.".to_string()
    } else if clause.contains("캐시") || clause.to_ascii_lowercase().contains("cache") {
        "저장공간 사용량이 커지고 빌드/검증 비용이 증가합니다.".to_string()
    } else {
        "사용자가 원인과 다음 행동을 바로 연결하기 어렵습니다.".to_string()
    }
}

fn infer_action_from_clause(clause: &str) -> String {
    if clause.contains("깨") || clause.contains("overflow") || clause.contains("벗어나") {
        "width-safe renderer와 quality-check로 폭 초과를 실패 처리합니다.".to_string()
    } else if clause.contains("색") || clause.to_ascii_lowercase().contains("color") {
        "semantic highlight role을 적용하고 plain/ANSI 모드를 분리합니다.".to_string()
    } else if clause.contains("캐시") || clause.to_ascii_lowercase().contains("cache") {
        "project-local target만 삭제하는 build-clean 정책을 사용합니다.".to_string()
    } else {
        "원인, 결과, 대응을 같은 행에 묶어 판단 비용을 줄입니다.".to_string()
    }
}

fn problem_diagnosis_report(
    profile: &Profile,
    prompt: &str,
    response: &str,
    summary: &str,
    width: usize,
) -> String {
    let conclusion = compact(summary, 1);
    let evidence = diagnosis_evidence(response, summary);
    let fix = diagnosis_fix(response, summary);
    let answer = compact(response, summary_sentence_limit(profile).min(4).max(2));
    let korean = contains_korean(prompt) || contains_korean(response);
    let headers = if korean {
        ["흐름", "내용"]
    } else {
        ["Flow", "Message"]
    };
    let rows = if korean {
        vec![
            vec!["결론".to_string(), conclusion],
            vec!["왜 그런가".to_string(), evidence],
            vec!["해결 흐름".to_string(), fix],
            vec!["네 질문".to_string(), compact(prompt, 1)],
            vec!["답".to_string(), answer],
        ]
    } else {
        vec![
            vec!["Conclusion".to_string(), conclusion],
            vec!["Evidence".to_string(), evidence],
            vec!["Fix".to_string(), fix],
            vec!["Your question".to_string(), compact(prompt, 1)],
            vec!["Answer".to_string(), answer],
        ]
    };
    wide_divider_table(&headers, &rows, profile.theme, width)
}

fn contains_korean(value: &str) -> bool {
    value.chars().any(|ch| ('가'..='힣').contains(&ch))
}

fn diagnosis_evidence(response: &str, summary: &str) -> String {
    if let Some(value) = labeled_diagnosis_value(response, "Evidence:") {
        return value;
    }
    let mut evidence = Vec::new();
    for sentence in split_sentences(response) {
        let lower = sentence.to_ascii_lowercase();
        if lower.contains("because")
            || lower.contains("error")
            || lower.contains("fail")
            || lower.contains("missing")
            || lower.contains("dirty")
            || lower.contains("skip")
            || sentence.contains("때문")
            || sentence.contains("오류")
            || sentence.contains("실패")
            || sentence.contains("안 ")
            || sentence.contains("없")
        {
            evidence.push(compact(&sentence, 1));
        }
        if evidence.len() >= 2 {
            break;
        }
    }
    if evidence.is_empty() {
        compact(summary, 2)
    } else {
        evidence.join(" / ")
    }
}

fn diagnosis_fix(response: &str, summary: &str) -> String {
    if let Some(value) = labeled_diagnosis_value(response, "Fix:") {
        return value;
    }
    for sentence in split_sentences(response) {
        let lower = sentence.to_ascii_lowercase();
        if lower.contains("fix")
            || lower.contains("solution")
            || lower.contains("run ")
            || lower.contains("use ")
            || lower.contains("should")
            || sentence.contains("해결")
            || sentence.contains("실행")
            || sentence.contains("수정")
            || sentence.contains("해야")
        {
            return compact(&sentence, 1);
        }
    }
    infer_action_from_clause(summary)
}

fn labeled_diagnosis_value(response: &str, label: &str) -> Option<String> {
    let start = response.find(label)? + label.len();
    let tail = response[start..].trim();
    if tail.is_empty() {
        return None;
    }
    let mut end = tail.len();
    for marker in [" Evidence:", " Fix:", " Conclusion:", " Answer:"] {
        if marker.trim() == label {
            continue;
        }
        if let Some(index) = tail.find(marker) {
            end = end.min(index);
        }
    }
    let value = tail[..end].trim().trim_matches(highlight_trim_char);
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
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
    let mut output = dispatch_explanation(selection, prompt, response, &summary, profile, width);
    let matched_styles = matching_custom_styles(prompt);
    if !matched_styles.is_empty() {
        let style_section = render_custom_style_section(&matched_styles, profile, width);
        output = format!("{style_section}\n\n{output}");
    }
    output
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
    let wants_architecture = architecture_prompt_requested(prompt);

    if requested.len() > 1 || wants_architecture {
        let mut sections = Vec::new();
        if wants_architecture {
            sections.push(architecture_panels(profile, summary, width));
        } else if requested.contains(&RendererKind::Table)
            && !specialized_table_renderer_requested(&requested)
        {
            sections.push(table(
                &["구분", "내용"],
                &structured_summary_rows(response, summary, profile),
                profile.frame,
                profile.theme,
                true,
                width,
            ));
        }

        if requested.contains(&RendererKind::ProsCons) {
            sections.push(pros_cons_for_width(profile, width));
        }
        if requested.contains(&RendererKind::Formula) {
            sections.push(formula(
                profile,
                "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다.",
            ));
        }
        if requested.contains(&RendererKind::CauseEffect) {
            sections.push(cause_effect_report(profile, response, summary, width));
        }
        if requested.contains(&RendererKind::ProblemDiagnosis) {
            sections.push(problem_diagnosis_report(
                profile, prompt, response, summary, width,
            ));
        }
        if requested.contains(&RendererKind::Progress) {
            sections.push(progress_report(profile, prompt, response, summary, width));
        }
        if requested.contains(&RendererKind::IndexedList) {
            let items = indexed_items(prompt, response, summary);
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
            prompt,
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
        let mut sections = Vec::new();
        match selection.renderer {
            RendererKind::Table => sections.push(table(
                &["구분", "내용"],
                &structured_summary_rows(response, summary, profile),
                profile.frame,
                profile.theme,
                true,
                width,
            )),
            RendererKind::Flow => {
                sections.push(codexplain_flow(profile.frame, profile.theme, width))
            }
            RendererKind::ProsCons => sections.push(pros_cons_for_width(profile, width)),
            RendererKind::Formula => sections.push(formula(profile, summary)),
            RendererKind::CauseEffect => {
                sections.push(cause_effect_report(profile, response, summary, width))
            }
            RendererKind::ProblemDiagnosis => sections.push(problem_diagnosis_report(
                profile, prompt, response, summary, width,
            )),
            RendererKind::IndexedList => {
                let items = indexed_items(prompt, response, summary);
                sections.push(indexed(
                    &items,
                    profile.frame,
                    profile.theme,
                    width,
                    profile.index_style,
                ));
            }
            RendererKind::Progress | RendererKind::TldrProse | RendererKind::Prose => {}
        }
        sections.extend(ux_component_sections(
            profile,
            prompt,
            response,
            summary,
            width,
            &ux_components,
        ));
        if requested.contains(&RendererKind::Progress) {
            sections.insert(
                0,
                progress_report(profile, prompt, response, summary, width),
            );
        }
        return sections.join("\n\n");
    }

    match selection.intent {
        ExplanationIntent::Comparison => {
            let mut output = pros_cons_for_width(profile, width);
            if renderer_signal_present(prompt, RendererKind::Formula) {
                output.push_str("\n\n");
                output.push_str(&formula(
                    profile,
                    "초기에는 반복속도, 제품화에는 배포/안정성 가중치가 커집니다.",
                ));
            }
            output
        }
        ExplanationIntent::CauseEffectReport => {
            cause_effect_report(profile, response, summary, width)
        }
        ExplanationIntent::ProblemDiagnosis => {
            problem_diagnosis_report(profile, prompt, response, summary, width)
        }
        ExplanationIntent::OrderedSteps => {
            let items = indexed_items(prompt, response, summary);
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
        ExplanationIntent::ProgressReport => {
            progress_report(profile, prompt, response, summary, width)
        }
        ExplanationIntent::StructuredSummary => table(
            &["구분", "내용"],
            &structured_summary_rows(response, summary, profile),
            profile.frame,
            profile.theme,
            true,
            width,
        ),
        ExplanationIntent::StatusSummary => format!(
            "{}{}\n{}{}",
            color(profile.theme, "heading", "TLDR: "),
            color(profile.theme, "success", &compact(response, 1)),
            color(profile.theme, "heading", "요약하면, "),
            color(profile.theme, "accent", summary)
        ),
        ExplanationIntent::GeneralAnswer => format!(
            "{}{}",
            color(profile.theme, "heading", "요약하면, "),
            color(profile.theme, "accent", summary)
        ),
    }
}

fn specialized_table_renderer_requested(requested: &[RendererKind]) -> bool {
    requested.iter().any(|renderer| {
        matches!(
            renderer,
            RendererKind::ProsCons
                | RendererKind::CauseEffect
                | RendererKind::ProblemDiagnosis
                | RendererKind::Progress
                | RendererKind::IndexedList
        )
    })
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
    let chars = text.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().copied().enumerate() {
        sentence.push(ch);
        let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
        let next = chars.get(index + 1).copied();
        let path_dot = ch == '.'
            && (next == Some('/')
                || previous == Some('/')
                || next.is_some_and(|value| value.is_ascii_alphanumeric()));
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') && !path_dot {
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

fn indexed_items(prompt: &str, response: &str, summary: &str) -> Vec<String> {
    let mut items = split_semantic_sections(response);
    if items.len() < 2 {
        items = split_semantic_sections(summary);
    }
    if items.len() < 2 {
        items = split_sentences(summary);
    }

    let force_numbering = [
        "두 가지",
        "2가지",
        "세 가지",
        "3가지",
        "크게 두",
        "과정",
        "순서",
        "단계",
        "첫째",
        "둘째",
    ]
    .iter()
    .any(|pattern| prompt_matches_pattern(prompt, pattern));
    if force_numbering {
        items.retain(|item| !is_index_intro_clause(item));
    }
    if items.is_empty() {
        split_sentences(summary)
    } else {
        items
    }
}

fn split_semantic_sections(text: &str) -> Vec<String> {
    let normalized = text.replace('\n', " ");
    let clauses = split_claim_clauses(&normalized);
    let mut items = Vec::new();

    for clause in clauses {
        for section in split_ordered_markers(&clause) {
            let section = section
                .trim()
                .trim_matches(highlight_trim_char)
                .trim()
                .to_string();
            if !section.is_empty() {
                items.push(section);
            }
        }
    }

    if items.len() >= 2 {
        return items;
    }

    split_sentences(&normalized)
}

fn split_ordered_markers(text: &str) -> Vec<String> {
    let mut normalized = text.to_string();
    for marker in [
        "첫째",
        "둘째",
        "셋째",
        "넷째",
        "다섯째",
        "1.",
        "2.",
        "3.",
        "4.",
        "5.",
    ] {
        normalized = normalized.replace(marker, &format!("\n{marker}"));
    }
    normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn is_index_intro_clause(item: &str) -> bool {
    let normalized = item.trim();
    let lower = normalized.to_ascii_lowercase();
    let intro_signal = normalized.contains("두 가지")
        || normalized.contains("2가지")
        || normalized.contains("세 가지")
        || normalized.contains("3가지")
        || normalized.contains("크게 두")
        || lower.contains("two paths")
        || lower.contains("two ways")
        || lower.contains("three paths")
        || lower.contains("three ways");
    intro_signal && visible_width(normalized) <= 48
}

fn layer_rows(summary: &str, profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = vec![vec!["TLDR".to_string(), compact(summary, 1)]];

    rows.extend(architecture_layer_rows(profile));

    if profile.explanation_depth != "light" {
        rows.push(vec![
            "Levels".to_string(),
            "Level Controls: explanation-depth, architecture-depth, abstraction-level을 light/standard/deep 계열 3단계로 조절합니다."
                .to_string(),
        ]);
    }

    rows
}

fn structured_summary_rows(response: &str, summary: &str, profile: &Profile) -> Vec<Vec<String>> {
    let claim_rows = claim_rows_from_text(response);
    if claim_rows.len() >= 2 {
        claim_rows
    } else {
        layer_rows(summary, profile)
    }
}

fn claim_rows_from_text(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for clause in split_claim_clauses(text) {
        let clause = clause.trim();
        if clause.is_empty() {
            continue;
        }
        if let Some((label, value)) = split_claim_clause(clause) {
            if label.chars().count() <= 24 && value.chars().count() >= 2 {
                rows.push(vec![label, value]);
            }
        } else if let Some(last) = rows.last_mut() {
            if let Some(value) = last.get_mut(1) {
                if !value.ends_with('.') {
                    value.push('.');
                }
                value.push(' ');
                value.push_str(clause);
            }
        }
    }
    rows
}

fn split_claim_clauses(text: &str) -> Vec<String> {
    let normalized = text.replace('\n', " ");
    let mut clauses = Vec::new();
    let mut current = String::new();
    let mut chars = normalized.chars().peekable();
    while let Some(ch) = chars.next() {
        if matches!(ch, '.' | ';' | '。' | '！' | '？' | '!' | '?') {
            push_claim_clause(&mut clauses, &mut current);
            continue;
        }
        if ch == ',' && next_chunk_has_claim_marker(chars.clone()) {
            push_claim_clause(&mut clauses, &mut current);
            continue;
        }
        current.push(ch);
    }
    push_claim_clause(&mut clauses, &mut current);
    clauses
}

fn push_claim_clause(clauses: &mut Vec<String>, current: &mut String) {
    let value = current.trim();
    if !value.is_empty() {
        clauses.push(value.to_string());
    }
    current.clear();
}

fn next_chunk_has_claim_marker<I>(chars: std::iter::Peekable<I>) -> bool
where
    I: Iterator<Item = char>,
{
    let mut chunk = String::new();
    for ch in chars.take(32) {
        if matches!(ch, '.' | ';' | '。' | '！' | '？' | '!' | '?' | ',') {
            break;
        }
        chunk.push(ch);
    }
    find_claim_marker(&chunk).is_some()
}

fn split_claim_clause(clause: &str) -> Option<(String, String)> {
    if let Some(index) = clause.find(':') {
        return claim_pair(&clause[..index], &clause[index + 1..]);
    }
    if let Some(index) = find_claim_marker(clause) {
        let marker_width = clause[index..].chars().next()?.len_utf8();
        return claim_pair(&clause[..index], &clause[index + marker_width..]);
    }
    None
}

fn find_claim_marker(clause: &str) -> Option<usize> {
    for (index, ch) in clause.char_indices() {
        if matches!(ch, '은' | '는') {
            return Some(index);
        }
    }
    None
}

fn claim_pair(label: &str, value: &str) -> Option<(String, String)> {
    let label = label.trim().trim_matches(highlight_trim_char).to_string();
    let value = value.trim().trim_matches(highlight_trim_char).to_string();
    if label.is_empty() || value.is_empty() {
        None
    } else {
        Some((label, value))
    }
}

fn architecture_layer_rows(profile: &Profile) -> Vec<Vec<String>> {
    match profile.abstraction_level.as_str() {
        "concrete" => concrete_architecture_rows(profile),
        "strategy" => strategy_architecture_rows(profile),
        _ => technical_architecture_rows(profile),
    }
}

fn technical_architecture_rows(profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            "Gateway".to_string(),
            "Input Gateway: project-local shim이 현재 프로젝트의 Codex 호출만 Codexplain wrapper로 전달합니다."
                .to_string(),
        ],
        vec![
            "Runner".to_string(),
            "Codex Runner: real Codex CLI를 찾아 실행하고 stdout/stderr와 exit code를 보존합니다.".to_string(),
        ],
        vec![
            "Policy".to_string(),
            "Strict Policy: JSON, code, diff, log, test output은 renderer 전에 그대로 통과시킵니다.".to_string(),
        ],
        vec![
            "Profile".to_string(),
            "Profile Resolver: theme, depth, abstraction, custom style, UX density 설정을 병합합니다.".to_string(),
        ],
        vec![
            "Selector".to_string(),
            "Renderer Selector: prompt 신호로 table, flow, pros-cons, formula, progress renderer를 조합합니다."
                .to_string(),
        ],
        vec![
            "Renderer".to_string(),
            "Terminal Renderer: Unicode box drawing, ANSI highlight, wrapping, visible-width 계산을 담당합니다."
                .to_string(),
        ],
    ];
    if profile.architecture_depth == "internals" || profile.explanation_depth == "deep" {
        rows.push(vec![
            "Lifecycle".to_string(),
            "Lifecycle Manager: install/on은 shim과 managed guidance를 넣고 off/uninstall은 Codexplain 관리분만 제거합니다."
                .to_string(),
        ]);
    }
    rows
}

fn concrete_architecture_rows(profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = technical_architecture_rows(profile);
    rows.push(vec![
        "Concrete".to_string(),
        "Concrete View: bin/codexplain, .codexplain/bin/codex, AGENTS.md managed block 같은 실행 단위를 함께 설명합니다."
            .to_string(),
    ]);
    rows
}

fn strategy_architecture_rows(profile: &Profile) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            "Boundary".to_string(),
            "모델 자체를 바꾸지 않고 Codex 출력 경계에서 설명 UX를 개선합니다.".to_string(),
        ],
        vec![
            "Safety".to_string(),
            "정확한 산출물은 보존하고 사람에게 읽히는 설명만 재구성합니다.".to_string(),
        ],
        vec![
            "Adapt".to_string(),
            "Adaptation: 사용자 스타일과 피드백을 renderer 선택에 반영해 설명 방식을 진화시킵니다."
                .to_string(),
        ],
    ];
    if profile.explanation_depth == "deep" {
        rows.extend(technical_architecture_rows(profile));
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

fn styles_dir() -> PathBuf {
    project_path(".codexplain/styles")
}

fn load_profile() -> Profile {
    load_profile_from_path(&profile_path())
}

fn load_profile_at(root: &Path) -> Profile {
    load_profile_from_path(&root.join(".codexplain/ux-profile.json"))
}

fn load_profile_from_path(path: &Path) -> Profile {
    let mut profile = Profile::default();
    if let Ok(raw) = fs::read_to_string(path) {
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
        if let Some(emoji_cues) = extract_json_bool(&raw, "emojiCues") {
            profile.emoji_cues = emoji_cues;
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
    if let Ok(value) = env::var("CODEXPLAIN_EMOJI_CUES") {
        profile.emoji_cues = !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "none" | "never" | "no"
        );
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
    if args.iter().any(|arg| arg == "--emoji-cues") {
        profile.emoji_cues = true;
    }
    if args.iter().any(|arg| arg == "--no-emoji-cues") {
        profile.emoji_cues = false;
    }
    profile.theme = match color_output_mode(args) {
        ColorOutput::Terminal => profile
            .theme
            .apply_terminal_policy(|key| env::var(key).ok()),
        ColorOutput::Plain => Theme::None,
        ColorOutput::Ansi | ColorOutput::Html | ColorOutput::Markdown => profile.theme,
    };
    profile
}

fn save_profile(profile: &Profile) -> io::Result<()> {
    save_profile_at(&project_path("."), profile)
}

fn save_profile_at(root: &Path, profile: &Profile) -> io::Result<()> {
    let codexplain_dir = root.join(".codexplain");
    fs::create_dir_all(&codexplain_dir)?;
    fs::write(
        codexplain_dir.join("ux-profile.json"),
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
                "  \"emojiCues\": {},\n",
                "  \"explanationMoves\": [\"tldr\", \"answer-first\", \"problem-diagnosis\", \"evidence\", \"fix\", \"question-answer\", \"next-step\"],\n",
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
            profile.risk_sensitivity,
            profile.emoji_cues
        ),
    )
}

fn color_command(args: &[String]) -> io::Result<()> {
    let action = args.get(1).map(String::as_str).unwrap_or("status");
    match action {
        "on" | "enable" => {
            write_color_config("ansi", "ansi", "semantic")?;
            println!("Codexplain color: on\n- defaultColorOutput: ansi\n- chatHighlightOutput: ansi\n- tuiAssistantColor: semantic\n- TUI env: CLICOLOR_FORCE=1 FORCE_COLOR=3 COLORTERM=truecolor CODEXPLAIN_TUI_COLOR=semantic\n- policy: restrained semantic emphasis");
        }
        "off" | "disable" => {
            write_color_config("plain", "plain", "off")?;
            println!("Codexplain color: off\n- defaultColorOutput: plain\n- chatHighlightOutput: plain\n- tuiAssistantColor: off\n- TUI env: NO_COLOR=1 CODEXPLAIN_TUI_COLOR=off");
        }
        "status" | "--show" | "show" => {
            let raw =
                fs::read_to_string(config_path()).unwrap_or_else(|_| LOCAL_CONFIG.to_string());
            let default = extract_json_string(&raw, "defaultColorOutput")
                .unwrap_or_else(|| "terminal".to_string());
            let chat =
                extract_json_string(&raw, "chatHighlightOutput").unwrap_or_else(|| default.clone());
            let tui = extract_json_string(&raw, "tuiAssistantColor")
                .unwrap_or_else(|| "full".to_string());
            let state = if matches!(parse_color_output(&default), ColorOutput::Plain) {
                "off"
            } else {
                "on"
            };
            println!(
                "Codexplain color: {state}\n- defaultColorOutput: {default}\n- chatHighlightOutput: {chat}\n- tuiAssistantColor: {tui}\n- patchedCodex: {}",
                local_patched_codex_binary()
                    .filter(|path| is_executable_file(path))
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "not-built".to_string())
            );
        }
        "rules" | "policy" => {
            print_color_rules(args);
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown color action: {other}"),
            ));
        }
    }
    Ok(())
}

fn print_color_rules(args: &[String]) {
    let profile = load_profile_for_args(args);
    let rows = vec![
        vec![
            "구조".to_string(),
            "border".to_string(),
            "표/박스/흐름도의 선만 색칠합니다.".to_string(),
        ],
        vec![
            "핵심".to_string(),
            "heading".to_string(),
            "TLDR, 렌더러, 아키텍처 계층처럼 주의를 줄 대상입니다.".to_string(),
        ],
        vec![
            "성공".to_string(),
            "success".to_string(),
            "완료, 통과, 가능, 보존 같은 긍정 상태입니다.".to_string(),
        ],
        vec![
            "주의".to_string(),
            "warning".to_string(),
            "필요, 진행, 우회, hook처럼 확인이 필요한 상태입니다.".to_string(),
        ],
        vec![
            "위험".to_string(),
            "danger".to_string(),
            "실패, 오류, 불가, 안 보임 같은 리스크 상태입니다.".to_string(),
        ],
        vec![
            "참조".to_string(),
            "command/path/artifact".to_string(),
            "명령, 경로, JSON/code/diff/log/test 같은 정확한 참조입니다.".to_string(),
        ],
    ];
    println!(
        "{}",
        table(
            &["의미", "색상 역할", "적용 규칙"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            88,
        )
    );
    println!("policy=semantic-sparse");
    println!("rule=색은 의미가 있는 토큰에만 보조 신호로 적용하고, 본문 전체를 알록달록하게 칠하지 않습니다.");
}

fn tui_color_command(args: &[String]) -> io::Result<()> {
    let action = args.get(1).map(String::as_str).unwrap_or("status");
    match action {
        "on" | "enable" => {
            write_color_config("ansi", "ansi", "semantic")?;
            println!("Codexplain TUI assistant color: on\n- scope: project-local only\n- mode: semantic\n- patchedCodex: {}", patched_codex_status());
        }
        "full" => {
            write_color_config("ansi", "ansi", "full")?;
            println!("Codexplain TUI assistant color: on\n- scope: project-local only\n- mode: full\n- patchedCodex: {}", patched_codex_status());
        }
        "off" | "disable" => {
            write_color_config("ansi", "ansi", "off")?;
            println!("Codexplain TUI assistant color: off\n- scope: project-local only\n- Codexplain exec/review color remains ansi unless `codexplain color off` is used");
        }
        "status" | "--show" | "show" => {
            println!(
                "Codexplain TUI assistant color\n- mode: {}\n- scope: project-local\n- patchedCodex: {}\n- note: interactive TUI color requires a patched Codex binary; exec/review output is still shaped by Codexplain",
                configured_tui_color_mode(),
                patched_codex_status()
            );
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown tui-color action: {other}"),
            ));
        }
    }
    Ok(())
}

fn tui_adapter_command(args: &[String]) -> io::Result<()> {
    let action = args.get(1).map(String::as_str).unwrap_or("status");
    match parse_tui_adapter_action(action)? {
        TuiAdapterAction::EnableFull => {
            write_color_config("ansi", "ansi", "semantic")?;
            println!(
                "{}",
                tui_adapter_status_report("semantic", patched_codex_status())
            );
        }
        TuiAdapterAction::Disable => {
            write_color_config("ansi", "ansi", "off")?;
            println!(
                "{}",
                tui_adapter_status_report("off", patched_codex_status())
            );
        }
        TuiAdapterAction::ApplyPatch => {
            ensure_project_local_codex_upstream()?;
            let outcome = apply_codex_tui_patch()?;
            println!(
                "Codexplain TUI adapter patch\n- scope: project-local only\n- patch: {}\n- result: {}\n- next: codexplain tui-adapter build",
                codex_tui_patch_path().display(),
                outcome
            );
        }
        TuiAdapterAction::Build => {
            ensure_project_local_codex_upstream()?;
            let patch_outcome = apply_codex_tui_patch()?;
            build_patched_codex_binary()?;
            write_color_config("ansi", "ansi", "semantic")?;
            println!(
                "{}\n- patch: {}\n- build: cargo build -p codex-cli --bin codex\n- result: project-local patched Codex TUI ready",
                tui_adapter_status_report("semantic", patched_codex_status()),
                patch_outcome
            );
        }
        TuiAdapterAction::Status => {
            println!(
                "{}",
                tui_adapter_status_report(&configured_tui_color_mode(), patched_codex_status())
            );
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TuiAdapterAction {
    EnableFull,
    Disable,
    ApplyPatch,
    Build,
    Status,
}

fn parse_tui_adapter_action(action: &str) -> io::Result<TuiAdapterAction> {
    match action {
        "on" | "enable" | "full" => Ok(TuiAdapterAction::EnableFull),
        "off" | "disable" => Ok(TuiAdapterAction::Disable),
        "apply" | "patch" => Ok(TuiAdapterAction::ApplyPatch),
        "build" => Ok(TuiAdapterAction::Build),
        "status" | "--show" | "show" => Ok(TuiAdapterAction::Status),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown tui-adapter action: {other}"),
        )),
    }
}

fn tui_adapter_status_report(mode: &str, patched_status: String) -> String {
    let shim = project_path(".codexplain/bin/codex");
    let active_target = if patched_status == "not-built" || mode == "off" {
        "fallback: stock Codex binary via PATH"
    } else {
        "project-local patched Codex binary"
    };
    format!(
        concat!(
            "Codexplain TUI adapter\n",
            "- scope: project-local only\n",
            "- mode: {}\n",
            "- shimPath: {}\n",
            "- activeBinary: {}\n",
            "- patchedCodex: {}\n",
            "- fallback: exec/review shaping remains available; interactive TUI assistant-message recoloring requires a project-local patched Codex binary\n",
            "- rollback: codexplain tui-adapter off, or codexplain off --local to remove managed project files and blocks\n",
            "- build: codexplain tui-adapter build applies the assistant-color and /codexplain slash patches, then builds only the project-local patched Codex binary\n",
            "- cleanup: codexplain build-clean --patched-codex removes only the project-local patched Codex Cargo target cache; the compact persisted binary stays available"
        ),
        mode,
        shim.display(),
        active_target,
        patched_status
    )
}

fn codex_tui_patch_path() -> PathBuf {
    project_path("patches/codex-tui-assistant-color.patch")
}

fn codex_tui_slash_patch_path() -> PathBuf {
    project_path("patches/codex-tui-codexplain-slash.patch")
}

fn codex_tui_patch_paths() -> Vec<PathBuf> {
    vec![codex_tui_patch_path(), codex_tui_slash_patch_path()]
}

fn upstream_codex_root() -> PathBuf {
    project_path(".codexplain/state/codex-upstream")
}

fn upstream_codex_rs_root() -> PathBuf {
    upstream_codex_root().join("codex-rs")
}

fn codex_tui_patch_already_applied() -> bool {
    codex_tui_color_patch_already_applied() && codex_tui_slash_patch_already_applied()
}

fn codex_tui_color_patch_already_applied() -> bool {
    let root = upstream_codex_rs_root();
    let messages = root.join("tui/src/history_cell/messages.rs");
    fs::read_to_string(messages)
        .map(|content| {
            content.contains("codexplain_style_hyperlink_lines")
                && content.contains("codexplain_tui_color_enabled")
                && content.contains("codexplain_semantic_style")
                && content.contains("codexplain_style_agent_span")
        })
        .unwrap_or(false)
}

fn codex_tui_slash_patch_already_applied() -> bool {
    let root = upstream_codex_rs_root();
    let slash_command = root.join("tui/src/slash_command.rs");
    let slash_dispatch = root.join("tui/src/chatwidget/slash_dispatch.rs");
    fs::read_to_string(slash_command)
        .map(|content| {
            content.contains("SlashCommand::Codexplain")
                && content.contains("control project-local Codexplain")
        })
        .unwrap_or(false)
        && fs::read_to_string(slash_dispatch)
            .map(|content| {
                content.contains("run_codexplain_slash_command")
                    && content.contains("run_codexplain_slash_command(\"toggle\")")
                    && content.contains("\"toggle\" | \"on\"")
                    && content.contains("\"settings\" | \"setting\"")
                    && content.contains("Usage: /codexplain [toggle|on|off|status|settings|help]")
            })
            .unwrap_or(false)
}

fn refresh_stale_codex_tui_slash_patch() -> io::Result<bool> {
    let root = upstream_codex_rs_root();
    let slash_dispatch = root.join("tui/src/chatwidget/slash_dispatch.rs");
    let Ok(current) = fs::read_to_string(&slash_dispatch) else {
        return Ok(false);
    };
    if !current.contains("run_codexplain_slash_command") {
        return Ok(false);
    }
    let next = current
        .replace(
            "self.run_codexplain_slash_command(\"status\");",
            "self.run_codexplain_slash_command(\"toggle\");",
        )
        .replace(
            "let action = action.split_whitespace().next().unwrap_or(\"status\");",
            "let action = action.split_whitespace().next().unwrap_or(\"toggle\");",
        )
        .replace(
            "\"on\" | \"enable\" | \"off\" | \"disable\" | \"status\" | \"help\" | \"-h\" | \"--help\"",
            "\"toggle\" | \"on\" | \"enable\" | \"off\" | \"disable\" | \"status\" | \"help\" | \"-h\" | \"--help\"",
        )
        .replace(
            "\"toggle\" | \"on\" | \"enable\" | \"off\" | \"disable\" | \"status\" | \"help\" | \"-h\" | \"--help\"",
            "\"toggle\" | \"on\" | \"enable\" | \"off\" | \"disable\" | \"status\" | \"settings\" | \"setting\" | \"config\" | \"configure\" | \"help\" | \"-h\" | \"--help\"",
        )
        .replace(
            "Usage: /codexplain [on|off|status|help]",
            "Usage: /codexplain [toggle|on|off|status|help]",
        )
        .replace(
            "Usage: /codexplain [toggle|on|off|status|help]",
            "Usage: /codexplain [toggle|on|off|status|settings|help]",
        );
    if next != current {
        fs::write(slash_dispatch, next)?;
        return Ok(true);
    }
    Ok(false)
}

fn apply_codex_tui_patch() -> io::Result<String> {
    let upstream = upstream_codex_root();
    let codex_rs = upstream_codex_rs_root();

    if !codex_rs.join("Cargo.toml").exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing project-local Codex upstream clone: {}",
                codex_rs.display()
            ),
        ));
    }
    if codex_tui_patch_already_applied() {
        return Ok("already-applied".to_string());
    }

    let mut applied = Vec::new();
    for patch in codex_tui_patch_paths() {
        if !patch.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing tracked patch file: {}", patch.display()),
            ));
        }
        let patch = fs::canonicalize(&patch)?;
        let patch_name = patch
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("codex-tui.patch")
            .to_string();
        let patch_needed = if patch_name.contains("slash") {
            !codex_tui_slash_patch_already_applied()
        } else {
            !codex_tui_color_patch_already_applied()
        };
        if !patch_needed {
            continue;
        }
        if patch_name.contains("slash") && refresh_stale_codex_tui_slash_patch()? {
            applied.push(format!("refreshed: {patch_name}"));
            continue;
        }
        run_command_checked(
            Command::new("git")
                .arg("-C")
                .arg(&upstream)
                .arg("apply")
                .arg("--check")
                .arg(&patch),
            &format!("git apply --check {patch_name}"),
        )?;
        run_command_checked(
            Command::new("git")
                .arg("-C")
                .arg(&upstream)
                .arg("apply")
                .arg(&patch),
            &format!("git apply {patch_name}"),
        )?;
        applied.push(patch_name);
    }
    if applied.is_empty() {
        Ok("already-applied".to_string())
    } else {
        Ok(format!("applied: {}", applied.join(", ")))
    }
}

fn build_patched_codex_binary() -> io::Result<()> {
    let codex_rs = upstream_codex_rs_root();
    if !codex_rs.join("Cargo.toml").exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing project-local Codex upstream clone: {}",
                codex_rs.display()
            ),
        ));
    }
    run_command_checked(
        Command::new("cargo")
            .current_dir(&codex_rs)
            .arg("build")
            .arg("-p")
            .arg("codex-cli")
            .arg("--bin")
            .arg("codex"),
        "cargo build -p codex-cli --bin codex",
    )?;
    persist_project_local_patched_codex_binary()?;
    let _ = cleanup_patched_codex_target(&project_path("."));
    Ok(())
}

fn persisted_patched_codex_binary() -> PathBuf {
    project_path(".codexplain/patched-codex/bin/codex")
}

fn built_patched_codex_binary_candidates() -> Vec<PathBuf> {
    vec![
        project_path(".codexplain/state/codex-upstream/codex-rs/target/release/codex"),
        project_path(".codexplain/state/codex-upstream/codex-rs/target/debug/codex"),
    ]
}

fn persist_project_local_patched_codex_binary() -> io::Result<()> {
    let Some(source) = built_patched_codex_binary_candidates()
        .into_iter()
        .find(|path| is_executable_file(path))
    else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "patched Codex build finished but no executable codex binary was found",
        ));
    };
    let destination = persisted_patched_codex_binary();
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&source, &destination)?;
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&destination)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&destination, permissions)?;
    }
    Ok(())
}

fn ensure_project_local_patched_codex_binary() -> io::Result<String> {
    if local_patched_codex_binary()
        .filter(|path| is_executable_file(path))
        .is_some()
    {
        return Ok("already-built".to_string());
    }

    ensure_project_local_codex_upstream()?;

    let patch_outcome = apply_codex_tui_patch()?;
    build_patched_codex_binary()?;
    Ok(format!("built ({patch_outcome})"))
}

fn ensure_project_local_codex_upstream() -> io::Result<String> {
    let upstream = upstream_codex_root();
    let codex_rs = upstream_codex_rs_root();
    if codex_rs.join("Cargo.toml").exists() {
        return Ok("already-cloned".to_string());
    }
    if upstream.exists() {
        return Err(io::Error::other(format!(
            "project-local Codex upstream exists but is incomplete: {}",
            upstream.display()
        )));
    }
    fs::create_dir_all(
        upstream
            .parent()
            .unwrap_or_else(|| Path::new(".codexplain/state")),
    )?;
    run_command_checked(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/openai/codex.git")
            .arg(&upstream),
        "git clone --depth 1 https://github.com/openai/codex.git",
    )?;
    if !codex_rs.join("Cargo.toml").exists() {
        return Err(io::Error::other(format!(
            "cloned Codex upstream but codex-rs/Cargo.toml was not found at {}",
            codex_rs.display()
        )));
    }
    Ok("cloned".to_string())
}

fn run_command_checked(command: &mut Command, label: &str) -> io::Result<()> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(io::Error::other(format!(
        "{label} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )))
}

fn patched_codex_status() -> String {
    local_patched_codex_binary()
        .filter(|path| is_executable_file(path))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "not-built".to_string())
}

fn write_color_config(default_output: &str, chat_output: &str, tui_output: &str) -> io::Result<()> {
    write_color_config_at(&project_path("."), default_output, chat_output, tui_output)
}

fn write_color_config_at(
    root: &Path,
    default_output: &str,
    chat_output: &str,
    tui_output: &str,
) -> io::Result<()> {
    let codexplain_dir = root.join(".codexplain");
    fs::create_dir_all(&codexplain_dir)?;
    fs::write(
        codexplain_dir.join("config.json"),
        local_config_json(default_output, chat_output, tui_output),
    )
}

fn local_config_json(default_output: &str, chat_output: &str, tui_output: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"schemaVersion\": 1,\n",
            "  \"defaultColorOutput\": \"{}\",\n",
            "  \"chatHighlightOutput\": \"{}\",\n",
            "  \"tuiAssistantColor\": \"{}\",\n",
            "  \"colorPolicy\": \"semantic-sparse\",\n",
            "  \"storageCheck\": {{\n",
            "    \"minFree\": {{\n",
            "      \"value\": 5,\n",
            "      \"unit\": \"gb\"\n",
            "    }}\n",
            "  }}\n",
            "}}\n"
        ),
        default_output, chat_output, tui_output
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

fn extract_json_bool(raw: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let index = raw.find(&needle)?;
    let rest = &raw[index + needle.len()..];
    let colon = rest.find(':')?;
    let after_colon = rest[colon + 1..].trim_start();
    if after_colon.starts_with("true") {
        Some(true)
    } else if after_colon.starts_with("false") {
        Some(false)
    } else {
        None
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

fn sanitize_style_name(value: &str) -> Option<String> {
    let name = value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn style_path(name: &str) -> Option<PathBuf> {
    sanitize_style_name(name).map(|safe| styles_dir().join(format!("{safe}.style")))
}

fn parse_renderer_list(value: &str) -> Vec<RendererKind> {
    value
        .split(',')
        .filter_map(RendererKind::from_structure)
        .fold(Vec::new(), |mut items, renderer| {
            if !items.contains(&renderer) {
                items.push(renderer);
            }
            items
        })
}

fn renderers_to_names(renderers: &[RendererKind]) -> String {
    renderers
        .iter()
        .map(|renderer| match renderer {
            RendererKind::Table => "table",
            RendererKind::ProsCons => "pros-cons",
            RendererKind::Formula => "formula",
            RendererKind::CauseEffect => "cause-effect",
            RendererKind::ProblemDiagnosis => "problem-diagnosis",
            RendererKind::IndexedList => "indexed",
            RendererKind::Flow => "flow",
            RendererKind::Progress => "progress",
            RendererKind::TldrProse => "tldr",
            RendererKind::Prose => "prose",
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_custom_style(raw: &str) -> Option<CustomStyle> {
    let mut name = String::new();
    let mut trigger = String::new();
    let mut renderers = Vec::new();
    let mut body = String::new();
    let mut tone = String::new();
    let mut example = String::new();
    let mut in_body = false;
    let mut in_example = false;
    for line in raw.lines() {
        if in_example {
            example.push_str(line);
            example.push('\n');
            continue;
        }
        if in_body {
            if line.trim() == "example:" {
                in_body = false;
                in_example = true;
                continue;
            }
            body.push_str(line);
            body.push('\n');
            continue;
        }
        if let Some(value) = line.strip_prefix("name:") {
            name = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("trigger:") {
            trigger = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("renderers:") {
            renderers = parse_renderer_list(value);
        } else if let Some(value) = line.strip_prefix("tone:") {
            tone = value.trim().to_string();
        } else if line.trim() == "body:" {
            in_body = true;
        } else if line.trim() == "example:" {
            in_example = true;
        }
    }
    let name = sanitize_style_name(&name)?;
    if trigger.trim().is_empty() {
        trigger = name.clone();
    }
    if renderers.is_empty() {
        renderers.push(RendererKind::TldrProse);
    }
    Some(CustomStyle {
        name,
        trigger,
        renderers,
        body: body.trim().to_string(),
        tone: if tone.trim().is_empty() {
            "technical".to_string()
        } else {
            tone
        },
        example: example.trim().to_string(),
    })
}

fn load_custom_styles() -> Vec<CustomStyle> {
    let Ok(entries) = fs::read_dir(styles_dir()) else {
        return Vec::new();
    };
    let mut styles = entries
        .flatten()
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|raw| parse_custom_style(&raw))
        .collect::<Vec<_>>();
    styles.sort_by(|a, b| a.name.cmp(&b.name));
    styles
}

fn matching_custom_styles(prompt: &str) -> Vec<CustomStyle> {
    let prompt_lower = prompt.to_ascii_lowercase();
    load_custom_styles()
        .into_iter()
        .filter(|style| {
            prompt_lower.contains(&style.name.to_ascii_lowercase())
                || prompt_lower.contains(&style.trigger.to_ascii_lowercase())
        })
        .collect()
}

fn render_custom_style_section(styles: &[CustomStyle], profile: &Profile, width: usize) -> String {
    let rows = styles
        .iter()
        .map(|style| {
            vec![
                style.name.clone(),
                format!(
                    "trigger={} · renderers={} · tone={} · rule={}",
                    style.trigger,
                    renderers_to_names(&style.renderers),
                    style.tone,
                    if style.body.is_empty() {
                        "사용자 정의 형식 신호만 적용합니다."
                    } else {
                        style.body.as_str()
                    }
                ),
            ]
        })
        .collect::<Vec<_>>();
    table(
        &["설명방식", "설정"],
        &rows,
        profile.frame,
        profile.theme,
        true,
        width,
    )
}

fn write_custom_style(args: &[String]) -> io::Result<()> {
    let Some(name) = args.get(2).and_then(|value| sanitize_style_name(value)) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "style add requires a safe style name",
        ));
    };
    let trigger = arg_value(args, "--trigger").unwrap_or(&name);
    let renderers = arg_value(args, "--renderers")
        .or_else(|| arg_value(args, "--renderer"))
        .unwrap_or("tldr,table");
    let body = arg_value(args, "--description")
        .or_else(|| arg_value(args, "--template"))
        .unwrap_or("사용자가 추가한 설명 방식입니다.");
    let tone = arg_value(args, "--tone").unwrap_or("technical");
    let example = arg_value(args, "--example").unwrap_or("");
    fs::create_dir_all(styles_dir())?;
    let path = style_path(&name).expect("sanitized name should produce a path");
    fs::write(
        &path,
        format!(
            "name: {name}\ntrigger: {trigger}\nrenderers: {renderers}\ntone: {tone}\nbody:\n{body}\nexample:\n{example}\n"
        ),
    )?;
    println!("Added Codexplain style: {}", path.display());
    println!("Preview with: codexplain style preview {name}");
    Ok(())
}

fn remove_custom_style(args: &[String]) -> io::Result<()> {
    let Some(name) = args.get(2) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "style remove requires a style name",
        ));
    };
    if let Some(path) = style_path(name) {
        remove_file_if_exists(&path)?;
        remove_dir_if_empty(&styles_dir())?;
        println!("Removed Codexplain style: {}", path.display());
    }
    Ok(())
}

fn list_custom_styles() {
    let styles = load_custom_styles();
    if styles.is_empty() {
        println!("No custom Codexplain styles");
        return;
    }
    let profile = load_profile();
    let rows = styles
        .iter()
        .map(|style| {
            vec![
                style.name.clone(),
                style.trigger.clone(),
                renderers_to_names(&style.renderers),
                style.tone.clone(),
            ]
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        table(
            &["Style", "Trigger", "Renderers", "Tone"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            88,
        )
    );
}

fn show_custom_style(args: &[String]) -> io::Result<()> {
    let Some(name) = args.get(2) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "style show requires a style name",
        ));
    };
    let Some(path) = style_path(name) else {
        return Ok(());
    };
    match fs::read_to_string(&path) {
        Ok(raw) => print!("{raw}"),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            println!("Style not found: {name}");
        }
        Err(error) => return Err(error),
    }
    Ok(())
}

fn preview_custom_style(args: &[String]) -> io::Result<()> {
    let Some(name) = args.get(2) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "style preview requires a style name",
        ));
    };
    let Some(style) = load_custom_styles()
        .into_iter()
        .find(|style| style.name == *name || style.trigger == *name)
    else {
        println!("Style not found: {name}");
        return Ok(());
    };
    let profile = load_profile_for_args(args);
    let sample = if style.example.trim().is_empty() {
        format!(
            "{} 방식으로 Codexplain 아키텍처를 기능 기준으로 설명해줘.",
            style.trigger
        )
    } else {
        style.example.clone()
    };
    let response = if style.body.trim().is_empty() {
        "응답 표현을 사용자가 원하는 구조로 재배치합니다. strict artifact는 보존합니다.".to_string()
    } else {
        style.body.clone()
    };
    println!(
        "{}",
        shape_for_output(&sample, &response, &profile, 88, ColorOutput::Ansi)
    );
    Ok(())
}

fn style_command(args: &[String]) -> io::Result<()> {
    match args.get(1).map(String::as_str) {
        Some("add") => write_custom_style(args),
        Some("remove") | Some("rm") | Some("delete") => remove_custom_style(args),
        Some("list") | None => {
            list_custom_styles();
            Ok(())
        }
        Some("show") => show_custom_style(args),
        Some("preview") => preview_custom_style(args),
        Some(other) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown style command: {other}"),
        )),
    }
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
    let mode = color_output_mode(args);
    print!(
        "{}",
        shape_for_output(&prompt, &response, &profile, width, mode)
    );
}

const CODEX_GUIDANCE_START: &str = "<!-- CODEXPLAIN:START -->";
const CODEX_GUIDANCE_END: &str = "<!-- CODEXPLAIN:END -->";
const SHELL_AUTO_START: &str = "# CODEXPLAIN_AUTO:START";
const SHELL_AUTO_END: &str = "# CODEXPLAIN_AUTO:END";
const CODEX_GUIDANCE: &str = r#"<!-- CODEXPLAIN:START -->
# Codexplain Response UX

Shape user-facing answers with a clear, readable, color-aware terminal/chat experience while preserving Codex's coding precision.

Default answer style:
- Start with the outcome or current state, not implementation detail.
- Use English by default for global open-source usage.
- Mirror the user's language when the user writes in Korean or another non-English language.
- Use connected Unicode boxes or tables when structure helps scanning.
- Use semantic ANSI colors for labels, risks, success states, artifact names, commands, paths, and next actions when the terminal supports color.
- Color policy is semantic-sparse: use color as a meaning signal, not decoration. Keep one dominant structure color, one key/accent color, and state colors only for success/warning/danger/reference terms.
- Use ANSI terminal color by default when Codexplain config asks for `defaultColorOutput: ansi`; for Codex CLI chat output, prefer real ANSI text color over emoji chips or raw HTML spans.
- Respect explanationDepth light/standard/deep, architectureDepth overview/system/internals, and abstractionLevel concrete/architecture/strategy.
- Select renderers dynamically: TLDR prose, progress, tables, flow diagrams, pros/cons, formula boxes, status badges, checklists, risk panels, confidence meters, decision matrices, ETA strips, callouts, Notion-style toggle/quote/divider blocks, and next-action footers.
- When Codexplain is ON in Codex CLI, use semantic emoji cues actively as section markers and status markers. Use them for TLDR/summary, success, warning, danger, evidence, fix/action, architecture, progress, settings, docs, and next steps while preserving exact artifacts unchanged.
- Emoji cues are part of the explanation UX, not decoration. Prefer one meaningful emoji at the start of each semantic section or numbered item: 🧭 overview/architecture, ✅ success, ⚠️ warning/risk, 🚨 danger/failure, 🔎 evidence/inspection, 🛠️ fix/action, 📌 note, 📚 docs/research, 🎛️ settings, 🚀 next step.
- Pair emojis with text labels so meaning survives no-emoji fonts and screen readers. Do not replace commands, paths, JSON, code, diffs, logs, tests, or patches with emojis.
- Treat UX blocks like tool choices: combine the smallest useful set from prompt, response, profile, and optional planner hints.
- Split explanations by semantic units with active line breaks. If the answer says "two paths", "두 가지", "과정", or "단계", render them as compact 1. 2. 3. numbered sections. Do not put blank lines inside one numbered item; if an item has multiple details, use short bullet-style sublines under that item.
- Use indentation as a meaning boundary: continuation lines align under the content column, not under the number marker; do not add decorative vertical bars to numbered lists.
- Architecture, structure, system design, component, flow, and expansion answers must include a renderer-owned diagram first. Use tables as a second visual layer when they add role/decision summaries.
- Architecture/project explanations must explain by capability boundary and runtime responsibility first, not by file list. Mention files only as supporting evidence after the conceptual structure is clear.
- Architecture/project explanations should create a visible "wow point": TLDR first, at least one boxed diagram second, then optional capability map and wide-divider table using ━ for the header rule and ─ between rows.
- Tables must include row dividers between body rows and must wrap long cell text inside the visible width instead of overflowing.
- Every visible table row must be separated. Do not produce a dense table where many `│ ... │` body rows appear back-to-back without `├...┤` or `─` row separators.
- Do not hand-draw long Unicode tables from raw model text. If a cell may exceed the terminal width, use Codexplain's width-safe renderer output, a Markdown table, or short per-item boxes so every cell is filled and padded by visible width.
- Process answers should use short numbered sections, with one idea per item and bullet-style sublines for multiple details.
- Problem-diagnosis answers should be answer-first, then flow downward from evidence/reason to fix and the user's exact question. In Korean, use natural labels such as 결론, 왜 그런가, 해결 흐름, 네 질문, 답 instead of forcing rigid English labels. In English, Conclusion/Evidence/Fix/Your question/Answer is acceptable when it improves scanning.
- Keep commands, paths, risks, test evidence, and exact technical facts intact.
- Do not continue an Ouroboros evolve/ralph lineage if drift is detected. Restart with an explicit project-local Seed.

Strict-output safety:
- Do not rewrite JSON, code blocks, diffs, patches, logs, test output, or commit messages when exact formatting matters.
- If exact formatting matters, return the artifact unchanged.

Terminal UX:
- Use connected box-drawing characters such as ┌ ┬ ┐ │ ├ ┼ ┤ └ ┴ ┘.
- Do not use broken pseudo-borders made from repeated hyphens, equals signs, or Korean long vowel marks.
- Do not hand-draw architecture, flow, or expansion diagrams when labels may wrap. Use Codexplain flow/diagram output so box width, connectors, arrows, and branch labels are layout-owned.
- Flow connectors must not rely on leading spaces alone. Prefer renderer-owned full-width connector rails so arrows stay aligned in Codex CLI, terminal scrollback, and chat transcripts.
- Do not hand-draw long raw box tables when cell text may wrap. Prefer Codexplain width-safe tables, Markdown tables, or short boxes with wrapped rows; every row must be layout-owned, padded, and separated, not manually guessed.
- For long tool transcripts such as Explored/Ran/Read, summarize the macro phase first instead of listing every micro event.
- Use blank lines between semantic sections so the user can scan without reading a wall of text.
- Slash control bridge: when a user asks exactly `/codexplain`, run `./bin/codexplain slash toggle` to switch project-local Codexplain UX on/off while preserving the native slash bridge. When a user asks exactly `/codexplain on`, run `./bin/codexplain slash on` for project-local activation. When a user asks exactly `/codexplain off`, run `./bin/codexplain slash off` to disable Codexplain UX but keep the installed `/codexplain` control available. When a user asks `/codexplain status`, run `./bin/codexplain slash status`. Do not treat these as prose requests.
- Settings control bridge: if the user asks to change Codexplain explanation depth, abstraction level, architecture depth, color, emoji, density, or explanation settings, run `./bin/codexplain settings-ui` when an interactive terminal is available. The bare `codexplain` command also opens this settings UI.
<!-- CODEXPLAIN:END -->"#;

const LOCAL_README: &str = r#"# Codexplain Local Adapter

This directory is project-local and Rust-only at runtime. The default output mode is ANSI terminal color for explanation surfaces.

To route this project's terminal Codex calls through Codexplain, activate the local shim:

```bash
source .codexplain/activate
codex exec "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘"
```

The shim only prepends `.codexplain/bin` in the current shell. `codexplain on --local` builds the project-local patched Codex TUI binary only when it is missing. `codexplain uninstall-codex --local` removes the shim files and the managed AGENTS.md block.

`codexplain on --local` also installs a managed zsh hook in `~/.zshrc` so new
shells automatically activate Codexplain when they start inside this project or
`cd` into it. `codexplain off --local` removes only that managed hook and the
project-local Codexplain files. Existing Codex TUI sessions must be reopened
after off/uninstall because slash commands are registered when the TUI binary
starts.

Color can be toggled without uninstalling Codexplain:

```bash
codexplain color on
codexplain color off
codexplain color status
codexplain color rules
```

Open the project-local status control surface or install local app launchers:

```bash
codexplain statusbar status
codexplain install-app
```

`codexplain statusbar on|off|set` controls power, theme, color output,
expression mode, and the three explanation depth levels without touching
unrelated global Codex settings.

Validate project-local OMX/harness compatibility without mutating settings:

```bash
codexplain compat-check
```

`codex exec` and `codex review` can be post-processed with Codexplain ANSI text color. Interactive Codex TUI is passed through to the real Codex process with color env (`CLICOLOR_FORCE`, `FORCE_COLOR`, `COLORTERM`). Assistant-message recoloring inside ratatui requires the project-local patched Codex renderer.

When this shim is active, `codex` startup performs a best-effort GitHub release
check. If a newer Codexplain release exists and this repo is on a branch with no
user-code changes, the shim runs `git pull --ff-only` and rebuilds the release
binary before starting Codex. Dirty Codexplain-managed local adapter files do
not block the check; unrelated dirty files still do. It never blocks Codex
startup on network failure. Disable it for one command with
`CODEXPLAIN_AUTO_UPDATE=off codex`.

Project-local interactive TUI assistant color can be toggled without touching global Codex settings:

```bash
codexplain tui-color on
codexplain tui-color full
codexplain tui-color off
codexplain tui-color status
```

Adapter status and rollback details are available through:

```bash
codexplain tui-adapter status
codexplain tui-adapter on
codexplain tui-adapter full
codexplain tui-adapter off
codexplain tui-adapter apply
codexplain tui-adapter build
```

`tui-adapter on` uses restrained semantic highlighting by default. Use
`tui-adapter full` only when you explicitly want stronger recoloring. If no
patched binary is present, it exits successfully and reports the fallback:
exec/review shaping still works, while interactive TUI assistant-message
recoloring needs a project-local patched Codex binary. `tui-adapter build`
applies `patches/codex-tui-assistant-color.patch` and
`patches/codex-tui-codexplain-slash.patch` to the ignored project-local upstream
clone and builds only the project-local patched Codex binary.

The shim routes to `.codexplain/patched-codex/bin/codex` first, then falls back to `.codexplain/state/codex-upstream/codex-rs/target/release/codex` or `.codexplain/state/codex-upstream/codex-rs/target/debug/codex` when that binary exists and `tuiAssistantColor` is enabled.

Use this adapter when a host can pipe a completed answer into a post-response command:

```bash
.codexplain/post-response --prompt "흐름도로 설명해줘"
```

Input may be plain text or JSON with `prompt` and `response` fields. The Rust core preserves exact JSON, code, diffs, logs, and test output when strict formatting matters.

Explanation depth uses 3-stage controls:

```text
explanationDepth light/standard/deep
architectureDepth overview/system/internals
abstractionLevel concrete/architecture/strategy
```

UX selection combines explicit rules, score thresholds, and optional planner hints through `CODEXPLAIN_UX_PLAN` or `CODEXPLAIN_UX_PLANNER_COMMAND`.

Custom explanation styles:

```bash
codexplain style add research-card --trigger "연구 카드" --renderers "tldr,table,formula" --description "배경, 근거, 한계, 다음 행동을 분리한다." --tone "research" --example "연구 카드로 이 설계를 설명해줘"
codexplain style add problem-diagnosis --trigger "왜 안됨" --renderers "problem-diagnosis" --description "문제 원인과 해결책을 결론부터 말하고 근거, 해결 흐름, 질문-답으로 자연스럽게 내려가며 정리한다." --tone "direct" --example "왜 안되고 있는지 문제와 해결책을 설명해줘"
codexplain style list
codexplain style preview research-card
codexplain style remove research-card
```
"#;

const LOCAL_CONFIG: &str = r#"{
  "schemaVersion": 1,
  "defaultColorOutput": "ansi",
  "chatHighlightOutput": "ansi",
  "tuiAssistantColor": "semantic",
  "colorPolicy": "semantic-sparse",
  "storageCheck": {
    "minFree": {
      "value": 5,
      "unit": "gb"
    }
  }
}
"#;

const POST_RESPONSE_SH: &str = r#"#!/usr/bin/env sh
set -eu
exec codexplain post-response "$@"
"#;

const CODEX_SHIM_SH: &str = r#"#!/usr/bin/env sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
export CODEXPLAIN_PROJECT_DIR="$ROOT"
export CODEXPLAIN_LOCAL_SHAPE=1
export CODEXPLAIN_SHIM_PATH="$0"
if grep -Eq '"defaultColorOutput"[[:space:]]*:[[:space:]]*"(plain|none|off|no-color)"' "$ROOT/.codexplain/config.json" 2>/dev/null; then
  export CODEXPLAIN_COLOR=never
  export CODEXPLAIN_COLOR_OUTPUT=plain
  export CODEXPLAIN_TUI_COLOR=off
  export NO_COLOR=1
  unset CLICOLOR_FORCE FORCE_COLOR
else
  export CODEXPLAIN_COLOR=always
  export CODEXPLAIN_COLOR_OUTPUT=ansi
  CODEXPLAIN_TUI_COLOR_VALUE=$(sed -n 's/.*"tuiAssistantColor"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$ROOT/.codexplain/config.json" 2>/dev/null | head -n 1)
  export CODEXPLAIN_TUI_COLOR="${CODEXPLAIN_TUI_COLOR_VALUE:-semantic}"
  export CLICOLOR_FORCE=1
  export FORCE_COLOR=3
  export COLORTERM=truecolor
  unset NO_COLOR
fi
codexplain_auto_update() {
  case "${CODEXPLAIN_AUTO_UPDATE:-1}" in
    0|false|off|never) return 0 ;;
  esac
  command -v git >/dev/null 2>&1 || return 0
  command -v cargo >/dev/null 2>&1 || return 0
  [ -d "$ROOT/.git" ] || return 0
  dirty_paths=$({
    git -C "$ROOT" diff --name-only 2>/dev/null
    git -C "$ROOT" diff --cached --name-only 2>/dev/null
  } | sort -u)
  for dirty_path in $dirty_paths; do
    case "$dirty_path" in
      AGENTS.md|.codexplain/README.md|.codexplain/config.json|.codexplain/ux-profile.json|.codexplain/activate|.codexplain/post-response|.codexplain/bin/codex) ;;
      *) return 0 ;;
    esac
  done
  branch=$(git -C "$ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || printf HEAD)
  [ "$branch" != "HEAD" ] || return 0
  current=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/v\1/p' "$ROOT/Cargo.toml" 2>/dev/null | head -n 1)
  [ -n "$current" ] || return 0
  latest=$(git ls-remote --tags --refs --sort=v:refname https://github.com/NomaDamas/Codexplain.git 'v*' 2>/dev/null | sed 's#.*refs/tags/##' | tail -n 1)
  [ -n "$latest" ] || return 0
  [ "$latest" != "$current" ] || return 0
  if git -C "$ROOT" pull --ff-only >/dev/null 2>&1; then
    (cd "$ROOT" && cargo build --release >/dev/null 2>&1) || return 0
    printf 'Codexplain auto-updated: %s -> %s\n' "$current" "$latest" >&2
  fi
}
codexplain_auto_update
codexplain_ensure_patched_tui() {
  case "${CODEXPLAIN_TUI_AUTO_BUILD:-0}" in
    0|false|off|never) return 0 ;;
  esac
  case "${CODEXPLAIN_TUI_COLOR:-semantic}" in
    off|plain|none|no-color) return 0 ;;
  esac
  [ -x "$ROOT/.codexplain/patched-codex/bin/codex" ] && return 0
  [ -x "$ROOT/.codexplain/state/codex-upstream/codex-rs/target/release/codex" ] && return 0
  [ -x "$ROOT/.codexplain/state/codex-upstream/codex-rs/target/debug/codex" ] && return 0
  "$ROOT/bin/codexplain" tui-adapter build >/dev/null 2>&1 || {
    printf 'Codexplain warning: /codexplain slash command needs patched Codex TUI; auto-build failed. Run: %s tui-adapter build\n' "$ROOT/bin/codexplain" >&2
  }
}
codexplain_ensure_patched_tui
if [ -d "$ROOT/.codexplain/state/codex-upstream/codex-rs/target" ]; then
  "$ROOT/bin/codexplain" build-clean --patched-codex >/dev/null 2>&1 || true
fi
exec "$ROOT/bin/codexplain" codex --local-shape "$@"
"#;

const ACTIVATE_SH: &str = r#"#!/usr/bin/env sh
# shellcheck shell=sh
CODEXPLAIN_PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE:-$0}")/.." && pwd)
export CODEXPLAIN_PROJECT_DIR
export CODEXPLAIN_LOCAL_SHAPE=1
if grep -Eq '"defaultColorOutput"[[:space:]]*:[[:space:]]*"(plain|none|off|no-color)"' "$CODEXPLAIN_PROJECT_DIR/.codexplain/config.json" 2>/dev/null; then
  export CODEXPLAIN_COLOR=never
  export CODEXPLAIN_COLOR_OUTPUT=plain
  export CODEXPLAIN_TUI_COLOR=off
  export NO_COLOR=1
  unset CLICOLOR_FORCE FORCE_COLOR
else
  export CODEXPLAIN_COLOR=always
  export CODEXPLAIN_COLOR_OUTPUT=ansi
  CODEXPLAIN_TUI_COLOR_VALUE=$(sed -n 's/.*"tuiAssistantColor"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$CODEXPLAIN_PROJECT_DIR/.codexplain/config.json" 2>/dev/null | head -n 1)
  export CODEXPLAIN_TUI_COLOR="${CODEXPLAIN_TUI_COLOR_VALUE:-semantic}"
  export CLICOLOR_FORCE=1
  export FORCE_COLOR=3
  export COLORTERM=truecolor
  unset NO_COLOR
fi
case ":$PATH:" in
  *":$CODEXPLAIN_PROJECT_DIR/.codexplain/bin:"*) ;;
  *) export PATH="$CODEXPLAIN_PROJECT_DIR/.codexplain/bin:$PATH" ;;
esac
alias codex="$CODEXPLAIN_PROJECT_DIR/.codexplain/bin/codex" 2>/dev/null || true
echo "Codexplain on: project-local codex shim is first on PATH"
"#;

const GLOBAL_CODEX_GUIDANCE: &str = r#"<!-- CODEXPLAIN:START -->
# Codexplain Global Response UX

Apply Codexplain's explanation UX globally unless a repository provides stricter local instructions.

Default answer style:
- Preserve exact JSON, code, diffs, patches, logs, test output, and commit messages.
- For explanatory answers, use English by default and mirror the user's language when the user writes in Korean or another non-English language.
- Use TLDR, Unicode tables, flow diagrams, pros/cons, formula boxes, progress UI, Notion-style toggle/quote/divider blocks, and next actions when they improve scanning.
- Prefer Markdown-safe chat highlights in chat hosts; use ANSI terminal colors in terminal hosts; fall back to plain text when exact formatting matters.
- Avoid hand-drawn long raw box tables that can overflow narrow terminals; prefer width-safe renderer output or Markdown tables. If Unicode boxes are used, every body row must be wrapped, padded, and separated by the renderer contract.
- Avoid hand-drawn architecture, flow, or expansion diagrams when labels may wrap; prefer renderer-owned boxes so connectors, arrows, and branch labels remain aligned.
- Collapse verbose Explored/Ran/Read transcripts into macro progress phases before details.
- Split "two paths", "두 가지", "과정", and "단계" explanations into compact numbered sections without blank lines inside an item.
- Architecture explanations must use boxed components and flow boxes before prose; tables should show row dividers and wrap long cells; flow diagrams should keep arrows and branches inside the requested width.
- Architecture explanations must lead with capability boundaries, runtime responsibility, and abstraction level. Do not lead with a file tree or file-by-file walkthrough unless the user explicitly asks for file layout.
- Every visible table row must be separated; never stack body rows directly without a row divider.
- Process answers should use short numbered sections, with one idea per item and bullet-style sublines for multiple details.
- Keep technical facts, commands, file paths, risks, and test evidence intact.
- Slash control bridge: if a repository has Codexplain installed and a user asks exactly `/codexplain`, `/codexplain on`, `/codexplain off`, or `/codexplain status`, treat it as a control command and run the matching `codexplain slash <action>` command instead of answering with prose. Bare `/codexplain` toggles project-local Codexplain on/off.
- Settings control bridge: if the user asks to change Codexplain explanation depth, abstraction level, architecture depth, color, emoji, density, or explanation settings, run `codexplain settings-ui` when an interactive terminal is available. The bare `codexplain` command also opens this settings UI.
<!-- CODEXPLAIN:END -->"#;

fn codex_home_dir() -> PathBuf {
    env::var("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .unwrap_or_else(|_| PathBuf::from(".codex"))
}

fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn install_codex_project(args: &[String]) -> io::Result<()> {
    if args.iter().any(|arg| arg == "--session") {
        print_session_activation_hint();
        return Ok(());
    }

    let install_local = args.iter().any(|arg| arg == "--local")
        || args.iter().any(|arg| arg == "--project")
        || !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--global" | "--session"));
    let install_global = args.iter().any(|arg| arg == "--global");

    if install_local {
        install_local_codex_project()?;
    }
    if install_global {
        install_global_codex_guidance()?;
    }
    Ok(())
}

fn session_activation_hint() -> String {
    format!(
        "Codexplain session activation:\n1. current shell only\n2. run: source ./.codexplain/activate\n3. verify: which codex"
    )
}

fn print_session_activation_hint() {
    println!("{}", session_activation_hint());
}

fn install_local_codex_project() -> io::Result<()> {
    let root = project_path(".");
    install_local_codex_project_at(&root)?;
    install_shell_autoload_for_project(&root)?;
    println!(
        "TUI adapter build: {}",
        ensure_project_local_patched_codex_binary()?
    );
    Ok(())
}

fn install_local_codex_project_at(root: &Path) -> io::Result<()> {
    let codexplain_dir = root.join(".codexplain");
    let codexplain_bin_dir = codexplain_dir.join("bin");
    fs::create_dir_all(&codexplain_dir)?;
    fs::create_dir_all(&codexplain_bin_dir)?;
    fs::write(codexplain_dir.join("README.md"), LOCAL_README)?;
    let local_config = codexplain_dir.join("config.json");
    if !local_config.exists() {
        fs::write(&local_config, LOCAL_CONFIG)?;
    }
    let post_response = codexplain_dir.join("post-response");
    fs::write(&post_response, POST_RESPONSE_SH)?;
    set_executable(&post_response)?;
    let codex_shim = codexplain_bin_dir.join("codex");
    fs::write(&codex_shim, CODEX_SHIM_SH)?;
    set_executable(&codex_shim)?;
    let activate = codexplain_dir.join("activate");
    fs::write(&activate, ACTIVATE_SH)?;
    set_executable(&activate)?;

    let agents_path = root.join("AGENTS.md");
    let next = if let Ok(current) = fs::read_to_string(&agents_path) {
        replace_guidance_block(&current, CODEX_GUIDANCE)
    } else {
        format!(
            "{CODEX_GUIDANCE}
"
        )
    };
    fs::write(agents_path, next)?;
    println!("Installed project-local Codex UX: .codexplain/bin/codex, .codexplain/activate, .codexplain/post-response, .codexplain/README.md, .codexplain/config.json, AGENTS.md");
    Ok(())
}

fn shell_autoload_rc_path() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".zshrc"))
}

fn shell_autoload_function_name(root: &Path) -> String {
    let mut value = root
        .display()
        .to_string()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    value.truncate(80);
    format!("_codexplain_auto_activate_{value}")
}

fn shell_autoload_block(root: &Path) -> String {
    let root_display = root.display().to_string();
    let root_quoted = shell_single_quote(&root_display);
    let function_name = shell_autoload_function_name(root);
    format!(
        "{SHELL_AUTO_START} {root_display}\n\
# Managed by Codexplain. Remove with: codexplain off --local\n\
{function_name}() {{\n\
  [ \"${{CODEXPLAIN_AUTO_ACTIVATING:-}}\" = \"1\" ] && return 0\n\
  _codexplain_root='{root_quoted}'\n\
  case \"$PWD/\" in\n\
    \"$_codexplain_root\"/*)\n\
      if [ -x \"$_codexplain_root/.codexplain/bin/codex\" ]; then\n\
        CODEXPLAIN_AUTO_ACTIVATING=1\n\
        export CODEXPLAIN_AUTO_ACTIVATING\n\
        if [ -z \"${{CODEXPLAIN_PREV_CODEX_ALIAS+x}}\" ]; then\n\
          CODEXPLAIN_PREV_CODEX_ALIAS=$(alias codex 2>/dev/null || true)\n\
        fi\n\
        export CODEXPLAIN_PROJECT_DIR=\"$_codexplain_root\"\n\
        export CODEXPLAIN_LOCAL_SHAPE=1\n\
        if grep -Eq '\"defaultColorOutput\"[[:space:]]*:[[:space:]]*\"(plain|none|off|no-color)\"' \"$_codexplain_root/.codexplain/config.json\" 2>/dev/null; then\n\
          export CODEXPLAIN_COLOR=never\n\
          export CODEXPLAIN_COLOR_OUTPUT=plain\n\
          export CODEXPLAIN_TUI_COLOR=off\n\
          export NO_COLOR=1\n\
          unset CLICOLOR_FORCE FORCE_COLOR\n\
        else\n\
          export CODEXPLAIN_COLOR=always\n\
          export CODEXPLAIN_COLOR_OUTPUT=ansi\n\
          export CODEXPLAIN_TUI_COLOR=semantic\n\
          export CLICOLOR_FORCE=1\n\
          export FORCE_COLOR=3\n\
          export COLORTERM=truecolor\n\
          unset NO_COLOR\n\
        fi\n\
        case \":$PATH:\" in\n\
          *\":$_codexplain_root/.codexplain/bin:\"*) ;;\n\
          *) export PATH=\"$_codexplain_root/.codexplain/bin:$PATH\" ;;\n\
        esac\n\
        alias codex=\"$_codexplain_root/.codexplain/bin/codex\" 2>/dev/null || true\n\
        unset CODEXPLAIN_AUTO_ACTIVATING\n\
      fi\n\
      ;;\n\
    *)\n\
      if [ \"${{CODEXPLAIN_PROJECT_DIR:-}}\" = \"$_codexplain_root\" ]; then\n\
        case \":$PATH:\" in\n\
          *\":$_codexplain_root/.codexplain/bin:\"*)\n\
            PATH=$(printf '%s' \"$PATH\" | sed \"s|^$_codexplain_root/.codexplain/bin:||;s|:$_codexplain_root/.codexplain/bin:|:|;s|:$_codexplain_root/.codexplain/bin$||;s|^$_codexplain_root/.codexplain/bin$||\")\n\
            export PATH\n\
            ;;\n\
        esac\n\
        if [ -n \"${{CODEXPLAIN_PREV_CODEX_ALIAS+x}}\" ]; then\n\
          if [ -n \"$CODEXPLAIN_PREV_CODEX_ALIAS\" ]; then\n\
            eval \"alias $CODEXPLAIN_PREV_CODEX_ALIAS\"\n\
          else\n\
            unalias codex 2>/dev/null || true\n\
          fi\n\
          unset CODEXPLAIN_PREV_CODEX_ALIAS\n\
        fi\n\
        unset CODEXPLAIN_PROJECT_DIR CODEXPLAIN_LOCAL_SHAPE CODEXPLAIN_COLOR CODEXPLAIN_COLOR_OUTPUT CODEXPLAIN_TUI_COLOR CLICOLOR_FORCE FORCE_COLOR COLORTERM CODEXPLAIN_AUTO_ACTIVATING\n\
      fi\n\
      ;;\n\
  esac\n\
}}\n\
autoload -Uz add-zsh-hook 2>/dev/null || true\n\
add-zsh-hook chpwd {function_name} 2>/dev/null || true\n\
{function_name}\n\
{SHELL_AUTO_END} {root_display}\n"
    )
}

fn install_shell_autoload_for_project(root: &Path) -> io::Result<()> {
    let Some(rc_path) = shell_autoload_rc_path() else {
        println!("Shell auto-activation skipped: HOME is not set");
        return Ok(());
    };
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let current = fs::read_to_string(&rc_path).unwrap_or_default();
    let without_old = remove_shell_autoload_block_for_root(&current, &root);
    let block = shell_autoload_block(&root);
    let mut next = without_old.trim_end().to_string();
    if !next.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(&block);
    if let Err(error) = fs::write(&rc_path, next) {
        eprintln!(
            "Codexplain warning: shell auto-activation skipped for {}: {error}",
            rc_path.display()
        );
        return Ok(());
    }
    println!(
        "Installed Codexplain zsh auto-activation: {}",
        rc_path.display()
    );
    Ok(())
}

fn remove_shell_autoload_for_project(root: &Path) -> io::Result<()> {
    let Some(rc_path) = shell_autoload_rc_path() else {
        return Ok(());
    };
    let Ok(current) = fs::read_to_string(&rc_path) else {
        return Ok(());
    };
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let next = remove_shell_autoload_block_for_root(&current, &root);
    if next != current {
        if let Err(error) = fs::write(&rc_path, next) {
            eprintln!(
                "Codexplain warning: shell auto-activation cleanup skipped for {}: {error}",
                rc_path.display()
            );
            return Ok(());
        }
        println!(
            "Removed Codexplain zsh auto-activation: {}",
            rc_path.display()
        );
    }
    Ok(())
}

fn remove_shell_autoload_block_for_root(current: &str, root: &Path) -> String {
    let start_marker = format!("{SHELL_AUTO_START} {}", root.display());
    let end_marker = format!("{SHELL_AUTO_END} {}", root.display());
    let Some(start) = current.find(&start_marker) else {
        return current.to_string();
    };
    let Some(end_offset) = current[start..].find(&end_marker) else {
        return current.to_string();
    };
    let end = start + end_offset + end_marker.len();
    let mut next = String::new();
    next.push_str(current[..start].trim_end());
    let tail = current[end..].trim_start();
    if !next.is_empty() && !tail.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(tail);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next
}

fn install_global_codex_guidance() -> io::Result<()> {
    let codex_home = codex_home_dir();
    fs::create_dir_all(&codex_home)?;
    let agents_path = codex_home.join("AGENTS.md");
    let next = if let Ok(current) = fs::read_to_string(&agents_path) {
        replace_guidance_block(&current, GLOBAL_CODEX_GUIDANCE)
    } else {
        format!(
            "{GLOBAL_CODEX_GUIDANCE}
"
        )
    };
    fs::write(&agents_path, next)?;
    println!(
        "Installed global Codexplain guidance: {}",
        agents_path.display()
    );
    Ok(())
}

fn uninstall_codex_project(args: &[String]) -> io::Result<()> {
    if args.iter().any(|arg| arg == "--session") {
        println!(
            "Codexplain session deactivation:\n1. close this shell, or\n2. remove .codexplain/bin from PATH in this shell\n3. no project/global files are changed"
        );
        return Ok(());
    }

    let uninstall_local = args.iter().any(|arg| arg == "--local")
        || args.iter().any(|arg| arg == "--project")
        || !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--global" | "--session"));
    let uninstall_global = args.iter().any(|arg| arg == "--global");
    let remove_profile = args.iter().any(|arg| arg == "--remove-profile");
    let strict_remove = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--strict" | "--purge"));

    if uninstall_local {
        if strict_remove {
            uninstall_local_codex_project_strict()?;
        } else {
            uninstall_local_codex_project(remove_profile)?;
        }
    }
    if uninstall_global {
        uninstall_global_codex_guidance()?;
    }
    Ok(())
}

fn uninstall_local_codex_project(remove_profile: bool) -> io::Result<()> {
    let root = project_path(".");
    remove_shell_autoload_for_project(&root)?;
    uninstall_local_codex_project_at(&root, remove_profile)
}

fn uninstall_local_codex_project_strict() -> io::Result<()> {
    let root = project_path(".");
    remove_shell_autoload_for_project(&root)?;
    uninstall_local_codex_project_strict_at(&root)
}

fn uninstall_local_codex_project_at(root: &Path, remove_profile: bool) -> io::Result<()> {
    let agents_path = root.join("AGENTS.md");
    remove_guidance_file_block(&agents_path)?;

    let codexplain_dir = root.join(".codexplain");
    remove_file_if_exists(&codexplain_dir.join("bin/codex"))?;
    remove_dir_if_empty(&codexplain_dir.join("bin"))?;
    remove_file_if_exists(&codexplain_dir.join("activate"))?;
    remove_file_if_exists(&codexplain_dir.join("post-response"))?;
    remove_file_if_exists(&codexplain_dir.join("README.md"))?;
    remove_file_if_exists(&codexplain_dir.join("config.json"))?;
    if remove_profile {
        remove_file_if_exists(&codexplain_dir.join("ux-profile.json"))?;
    }
    remove_dir_if_empty(&codexplain_dir)?;
    println!("Uninstalled project-local Codexplain UX");
    Ok(())
}

fn uninstall_local_codex_project_strict_at(root: &Path) -> io::Result<()> {
    let agents_path = root.join("AGENTS.md");
    remove_guidance_file_block(&agents_path)?;
    let codexplain_dir = root.join(".codexplain");
    remove_codexplain_dir_strict(&codexplain_dir)?;
    println!("Strictly removed project-local Codexplain UX");
    Ok(())
}

fn uninstall_global_codex_guidance() -> io::Result<()> {
    let agents_path = codex_home_dir().join("AGENTS.md");
    remove_guidance_file_block(&agents_path)?;
    println!(
        "Uninstalled global Codexplain guidance: {}",
        agents_path.display()
    );
    Ok(())
}

fn managed_project_files() -> &'static [&'static str] {
    &[
        "AGENTS.md managed CODEXPLAIN block",
        ".codexplain/bin/codex",
        ".codexplain/activate",
        ".codexplain/post-response",
        ".codexplain/README.md",
        ".codexplain/config.json",
    ]
}

fn exports_or_forwards_local_shape(script: &str) -> bool {
    script.lines().map(str::trim).any(|line| {
        line == "export CODEXPLAIN_LOCAL_SHAPE=1"
            || line.starts_with("export CODEXPLAIN_LOCAL_SHAPE=1 ")
            || line.starts_with("CODEXPLAIN_LOCAL_SHAPE=1 ")
    })
}

fn compat_check() {
    let root = project_path(".");
    let gitignore = fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    let strict_json = r#"{"ok":true,"mode":"strict"}"#;
    let profile = Profile {
        theme: Theme::None,
        ..Profile::default()
    };
    let strict_preserved = shape("valid JSON만 출력", strict_json, &profile, 88) == strict_json;
    let quality = quality_report(88);
    let ignored_state = gitignore.contains(".codexplain/state/");
    let ignored_harness = gitignore.contains("harness/")
        && gitignore.contains("oh-my-codex/")
        && gitignore.contains("omx/");
    let local_assets = managed_project_files().join(", ");
    let session_hint = session_activation_hint();
    let global_block_is_managed = GLOBAL_CODEX_GUIDANCE.contains("CODEXPLAIN:START")
        && GLOBAL_CODEX_GUIDANCE.contains("CODEXPLAIN:END");
    let local_block_is_managed =
        CODEX_GUIDANCE.contains("CODEXPLAIN:START") && CODEX_GUIDANCE.contains("CODEXPLAIN:END");
    let shim_project_dir_forwarded = CODEX_SHIM_SH
        .contains(r#"ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"#)
        && CODEX_SHIM_SH.contains(r#"export CODEXPLAIN_PROJECT_DIR="$ROOT""#);
    let activate_project_dir_exported = ACTIVATE_SH.contains(
        r#"CODEXPLAIN_PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE:-$0}")/.." && pwd)"#,
    ) && ACTIVATE_SH.contains("export CODEXPLAIN_PROJECT_DIR");
    let shim_local_shape_forwarded = exports_or_forwards_local_shape(CODEX_SHIM_SH);
    let shim_project_local = shim_project_dir_forwarded
        && activate_project_dir_exported
        && shim_local_shape_forwarded
        && CODEX_SHIM_SH.contains("codex --local-shape");
    let session_is_non_mutating = session_hint.contains("source ")
        && !session_hint.contains("Installed")
        && !session_hint.contains("Uninstalled");

    println!("contract=codexplain.compat-check.v1");
    println!("scope=project-local-first");
    println!("managed_project_files={local_assets}");
    println!(
        "project_dir_forwarded={}",
        pass_fail(shim_project_dir_forwarded && activate_project_dir_exported)
    );
    println!(
        "local_shape_forwarded={}",
        pass_fail(shim_local_shape_forwarded)
    );
    println!("project_local_shim={}", pass_fail(shim_project_local));
    println!(
        "session_hint_non_mutating={}",
        pass_fail(session_is_non_mutating)
    );
    println!(
        "global_block_managed_only={}",
        pass_fail(global_block_is_managed)
    );
    println!(
        "local_block_managed_only={}",
        pass_fail(local_block_is_managed)
    );
    println!("strict_json_preserved={}", pass_fail(strict_preserved));
    println!("width_safe_quality={}", pass_fail(quality.passed()));
    println!("gitignore_state={}", pass_fail(ignored_state));
    println!("gitignore_harness={}", pass_fail(ignored_harness));
    println!(
        "result={}",
        pass_fail(
            shim_project_local
                && shim_project_dir_forwarded
                && activate_project_dir_exported
                && session_is_non_mutating
                && global_block_is_managed
                && local_block_is_managed
                && strict_preserved
                && quality.passed()
                && ignored_state
                && ignored_harness
        )
    );
}

fn pass_fail(value: bool) -> &'static str {
    if value {
        "pass"
    } else {
        "fail"
    }
}

fn remove_guidance_file_block(path: &Path) -> io::Result<()> {
    let Ok(current) = fs::read_to_string(path) else {
        return Ok(());
    };
    let next = remove_guidance_block(&current);
    if next.trim().is_empty() {
        remove_file_if_exists(path)
    } else if next != current {
        fs::write(path, next)
    } else {
        Ok(())
    }
}

fn remove_guidance_block(current: &str) -> String {
    let Some(start) = current.find(CODEX_GUIDANCE_START) else {
        return current.to_string();
    };
    let Some(end_offset) = current[start..].find(CODEX_GUIDANCE_END) else {
        return current.to_string();
    };
    let end = start + end_offset + CODEX_GUIDANCE_END.len();
    let mut next = String::new();
    next.push_str(current[..start].trim_end());
    let tail = current[end..].trim_start();
    if !next.is_empty() && !tail.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(tail);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next
}
fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_dir_if_empty(path: &Path) -> io::Result<()> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_codexplain_dir_strict(path: &Path) -> io::Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        return remove_file_if_exists(path);
    }
    fs::remove_dir_all(path)
}

fn slash_control(args: &[String]) -> io::Result<()> {
    match args.get(1).map(String::as_str).unwrap_or("toggle") {
        "toggle" | "" => {
            if project_local_codexplain_enabled() {
                disable_project_local_codexplain_preserve_slash()?;
                println!("Codexplain disabled");
            } else {
                install_local_codex_project()?;
                write_color_config("ansi", "ansi", "semantic")?;
                println!("Codexplain enabled");
                println!("{}", slash_enabled_guide());
            }
        }
        "on" | "enable" => {
            install_local_codex_project()?;
            write_color_config("ansi", "ansi", "semantic")?;
            println!("Codexplain enabled");
            println!("{}", slash_enabled_guide());
        }
        "off" | "disable" => {
            disable_project_local_codexplain_preserve_slash()?;
            println!("Codexplain disabled");
        }
        "status" => {
            if project_local_codexplain_enabled() {
                println!("Codexplain enabled");
            } else {
                println!("Codexplain disabled");
            }
        }
        "help" | "-h" | "--help" => {
            println!("{}", slash_help());
        }
        "settings" | "setting" | "config" | "configure" => {
            println!("{}", slash_settings_guide());
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown /codexplain action: {other}\n{}", slash_help()),
            ));
        }
    }
    Ok(())
}

fn slash_help() -> &'static str {
    "/codexplain toggles UX on/off; /codexplain on|off|status|settings|help"
}

fn slash_settings_guide() -> &'static str {
    "Codexplain settings\n- open UI: codexplain or codexplain settings-ui\n- adjust: explanation depth, architecture view, abstraction level, theme, UX density, emoji cues\n- scope: project-local only; off/uninstall removes managed Codexplain state"
}

fn slash_enabled_guide() -> &'static str {
    "Quick guide\n- settings UI: codexplain\n- explicit UI: codexplain settings-ui\n- status: /codexplain status\n- disable: /codexplain off\n- exact artifacts stay unchanged; explanations get Codexplain UX"
}

fn project_local_codexplain_enabled() -> bool {
    if !project_local_adapter_present_at(&project_path(".")) {
        return false;
    }
    let guidance_enabled = fs::read_to_string(project_path("AGENTS.md"))
        .map(|content| content.contains(CODEX_GUIDANCE_START))
        .unwrap_or(false);
    guidance_enabled && color_feature_enabled()
}

fn disable_project_local_codexplain_preserve_slash() -> io::Result<()> {
    let root = project_path(".");
    disable_project_local_codexplain_preserve_slash_at(&root)
}

fn disable_project_local_codexplain_preserve_slash_at(root: &Path) -> io::Result<()> {
    if !project_local_adapter_present_at(&root) {
        install_local_codex_project_at(&root)?;
    }
    remove_guidance_file_block(&root.join("AGENTS.md"))?;
    write_color_config_at(root, "plain", "plain", "off")
}
fn replace_guidance_block(current: &str, block: &str) -> String {
    let Some(start) = current.find(CODEX_GUIDANCE_START) else {
        return format!("{}\n\n{}\n", current.trim_end(), block);
    };
    let Some(end_offset) = current[start..].find(CODEX_GUIDANCE_END) else {
        return format!("{}\n\n{}\n", current.trim_end(), block);
    };
    let end = start + end_offset + CODEX_GUIDANCE_END.len();
    format!(
        "{}{}{}",
        current[..start].trim_end(),
        format!("\n\n{block}\n"),
        current[end..].trim_start()
    )
}

fn feedback(args: &[String], rlhf: bool) -> io::Result<()> {
    let mut profile = load_profile();
    let rating = arg_value(args, "--rating").and_then(|value| value.parse::<i32>().ok());
    let comment = arg_value(args, "--comment").unwrap_or("");
    if let Some(detail) = arg_value(args, "--detail") {
        profile.detail = detail.to_string();
        profile.explanation_depth = normalize_explanation_depth(detail, &profile.explanation_depth);
    }
    if let Some(style) = arg_value(args, "--style").or_else(|| arg_value(args, "--set-style")) {
        profile.style = style.to_string();
    }
    let lower = comment.to_ascii_lowercase();
    if lower.contains("more detail") || comment.contains("자세") || comment.contains("부족") {
        profile.explanation_depth = "deep".to_string();
    }
    if lower.contains("short") || comment.contains("짧") || comment.contains("간단") {
        profile.explanation_depth = "light".to_string();
    }
    if lower.contains("architecture") || comment.contains("아키텍처") || comment.contains("구조")
    {
        profile.architecture_depth = "internals".to_string();
    }
    save_profile(&profile)?;
    if rlhf {
        println!("Updated .codexplain/ux-profile.json\n- rating: {}\n- explanationDepth: {}\n- architectureDepth: {}\n- abstractionLevel: {}", rating.map(|v| v.to_string()).unwrap_or_else(|| "none".to_string()), profile.explanation_depth, profile.architecture_depth, profile.abstraction_level);
    } else {
        println!(
            "Updated .codexplain/ux-profile.json: explanationDepth={}, style={}",
            profile.explanation_depth, profile.style
        );
    }
    Ok(())
}

fn guide(args: &[String]) {
    let prompt = arg_value(args, "--prompt").unwrap_or("");
    let profile = load_profile_for_args(args);
    println!(
        "Codexplain guidance\n- prompt: {}\n- explanationDepth: {}\n- architectureDepth: {}\n- abstractionLevel: {}\n- theme: {}\n- frame: {}\n- rule: preserve strict artifacts, then choose the smallest useful renderer set",
        prompt,
        profile.explanation_depth,
        profile.architecture_depth,
        profile.abstraction_level,
        profile.theme.name(),
        if profile.frame == Frame::Ascii { "ascii" } else { "unicode" }
    );
}

fn run_codex(args: &[String]) -> i32 {
    let mut codex_args = Vec::new();
    let mut prompt = String::new();
    let mut local_shape = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--prompt" => {
                if let Some(value) = args.get(index + 1) {
                    prompt = value.clone();
                }
                index += 2;
            }
            "--local-shape" => {
                local_shape = true;
                index += 1;
            }
            other => {
                codex_args.push(other.to_string());
                index += 1;
            }
        }
    }
    codex_args = prepare_codex_args(codex_args, &prompt);
    let codex_bin = resolve_real_codex_binary();
    if !should_capture_codex_output(&codex_args) {
        let mut command = Command::new(&codex_bin);
        command.args(&codex_args);
        apply_codex_color_env(&mut command, color_feature_enabled());
        let status = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        return match status {
            Ok(status) => status.code().unwrap_or(1),
            Err(error) => {
                eprintln!(
                    "failed to run codex at {}; ensure Codex CLI is installed and on PATH: {error}",
                    codex_bin.display()
                );
                127
            }
        };
    }

    let mut command = Command::new(&codex_bin);
    command.args(&codex_args);
    apply_codex_color_env(&mut command, color_feature_enabled());
    let output = command.output();
    let Ok(output) = output else {
        eprintln!(
            "failed to run codex at {}; ensure Codex CLI is installed and on PATH",
            codex_bin.display()
        );
        return 127;
    };
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if local_shape || env_flag_enabled(env::var("CODEXPLAIN_LOCAL_SHAPE").ok()) {
        let profile = load_profile_for_args(args);
        let mode = color_output_mode(args);
        let effective_prompt = if prompt.is_empty() {
            codex_args.join(" ")
        } else {
            prompt
        };
        print!(
            "{}",
            shape_for_output(&effective_prompt, &stdout, &profile, terminal_width(), mode)
        );
    } else {
        print!("{stdout}");
    }
    output.status.code().unwrap_or(1)
}

fn prepare_codex_args(mut codex_args: Vec<String>, prompt: &str) -> Vec<String> {
    if codex_args.is_empty() && !prompt.is_empty() {
        codex_args.push("exec".to_string());
        codex_args.push(prompt.to_string());
    }
    codex_args
}

fn should_capture_codex_output(codex_args: &[String]) -> bool {
    codex_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "exec" | "e" | "review"))
}

fn apply_codex_color_env(command: &mut Command, enabled: bool) {
    if enabled {
        command.env_remove("NO_COLOR");
        command.env("CODEXPLAIN_COLOR", "always");
        command.env("CODEXPLAIN_COLOR_OUTPUT", "ansi");
        command.env("CODEXPLAIN_TUI_COLOR", tui_color_env_value());
        command.env("CLICOLOR_FORCE", "1");
        command.env("FORCE_COLOR", "3");
        command.env("COLORTERM", "truecolor");
        if env::var("TERM")
            .map(|value| value == "dumb" || value.trim().is_empty())
            .unwrap_or(true)
        {
            command.env("TERM", "xterm-256color");
        }
    } else {
        command.env("CODEXPLAIN_COLOR", "never");
        command.env("CODEXPLAIN_COLOR_OUTPUT", "plain");
        command.env("CODEXPLAIN_TUI_COLOR", "off");
        command.env("NO_COLOR", "1");
        command.env_remove("CLICOLOR_FORCE");
        command.env_remove("FORCE_COLOR");
    }
}

fn resolve_real_codex_binary() -> PathBuf {
    if let Ok(path) = env::var("CODEXPLAIN_REAL_CODEX") {
        let path = PathBuf::from(path);
        if is_executable_file(&path) {
            return path;
        }
    }

    if tui_color_feature_enabled() || native_slash_feature_enabled() {
        if let Some(path) = local_patched_codex_binary().filter(|path| is_executable_file(path)) {
            return path;
        }
    }

    let shim = env::var("CODEXPLAIN_SHIM_PATH")
        .ok()
        .map(PathBuf::from)
        .and_then(|path| path.canonicalize().ok());
    let path_var = env::var_os("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join("codex");
        if !is_executable_file(&candidate) {
            continue;
        }
        let canonical = candidate.canonicalize().unwrap_or(candidate.clone());
        if shim
            .as_ref()
            .is_some_and(|shim_path| *shim_path == canonical)
        {
            continue;
        }
        if canonical
            .to_string_lossy()
            .contains("/.codexplain/bin/codex")
        {
            continue;
        }
        return candidate;
    }
    PathBuf::from("codex")
}

fn native_slash_feature_enabled() -> bool {
    !matches!(
        env::var("CODEXPLAIN_NATIVE_SLASH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "0" | "false" | "off" | "none" | "never"
    ) && project_local_adapter_present_at(&project_path("."))
}

fn local_patched_codex_binary() -> Option<PathBuf> {
    let mut candidates = std::iter::once(persisted_patched_codex_binary())
        .chain(built_patched_codex_binary_candidates())
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    candidates.sort_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());
    candidates.pop()
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn terminal_width() -> usize {
    env::var("CODEXPLAIN_WIDTH")
        .or_else(|_| env::var("COLUMNS"))
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(100)
}

fn build_size() {
    let root = project_path(".");
    let binary = root.join("target/release/codexplain");
    let target = root.join("target");
    let binary_mb = fs::metadata(binary)
        .map(|meta| meta.len() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0);
    let target_mb = dir_size(&target) as f64 / 1024.0 / 1024.0;
    println!("contract=codexplain.build-size.v1\nbinary_mb={binary_mb:.2}\ntarget_mb={target_mb:.1}\nstatus=ok");
}

fn build_clean(args: &[String]) -> io::Result<()> {
    let root = project_path(".");
    if args.iter().any(|arg| arg == "--patched-codex") {
        match cleanup_patched_codex_target(&root)? {
            TargetCleanup::Removed => println!("cleaned=patched_codex_target"),
            TargetCleanup::AlreadyAbsent => println!("cleaned=patched_codex_target_already_absent"),
        }
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--target") || args.iter().any(|arg| arg == "--all") {
        match cleanup_project_storage_dir(&root, "target")? {
            TargetCleanup::Removed => println!("cleaned=target"),
            TargetCleanup::AlreadyAbsent => println!("cleaned=target_already_absent"),
        }
    } else {
        println!("nothing_cleaned=pass --target to remove Cargo target artifacts");
    }
    Ok(())
}

fn cleanup_patched_codex_target(root: &Path) -> io::Result<TargetCleanup> {
    let root = root.canonicalize()?;
    let target = root.join(".codexplain/state/codex-upstream/codex-rs/target");
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
            "refusing to clean patched Codex target because it is a symlink",
        ));
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean patched Codex target because it is not a directory",
        ));
    }

    let resolved_target = target.canonicalize()?;
    if !resolved_target.starts_with(root.join(".codexplain/state/codex-upstream/codex-rs"))
        || resolved_target.file_name().and_then(|name| name.to_str()) != Some("target")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "refusing to clean any path outside the project-local patched Codex target directory",
        ));
    }

    fs::remove_dir_all(&target)?;
    Ok(TargetCleanup::Removed)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QualityReport {
    width: usize,
    overflow_lines: usize,
    row_dividers: usize,
    architecture_boxes: usize,
    architecture_panel_overflows: usize,
    flow_arrows: usize,
    flow_box_overflows: usize,
    flow_connector_breaks: usize,
    expansion_overflows: usize,
    numbered_sections: usize,
    score: u8,
}

impl QualityReport {
    fn passed(&self) -> bool {
        self.overflow_lines == 0
            && self.row_dividers >= 3
            && self.architecture_boxes >= 6
            && self.architecture_panel_overflows == 0
            && self.flow_arrows >= 4
            && self.flow_box_overflows == 0
            && self.flow_connector_breaks == 0
            && self.expansion_overflows == 0
            && self.numbered_sections >= 2
            && self.score >= 90
    }
}

fn quality_report(width: usize) -> QualityReport {
    let width = width.max(50);
    let profile = Profile {
        theme: Theme::None,
        frame: Frame::Unicode,
        ..Profile::default()
    };
    let architecture = shape(
        "이 프로젝트 아키텍처를 표와 흐름도로 설명해줘",
        "Codexplain은 Rust CLI core가 Codex 출력과 사용자 응답을 받아 strict-output policy, profile resolver, renderer selector, terminal renderer, lifecycle installer, storage controls를 통과시킵니다.",
        &profile,
        width,
    );
    let long_table = table(
        &["대상", "상태", "근거"],
        &[
            vec![
                "프로젝트 설정".to_string(),
                "켜짐".to_string(),
                "AGENTS.md의 관리 블록과 .codexplain/config.json으로 프로젝트 로컬 UX를 적용합니다."
                    .to_string(),
            ],
            vec![
                "현재 셸 PATH".to_string(),
                "검증 필요".to_string(),
                "source .codexplain/activate 이후 which codex가 프로젝트 shim을 가리켜야 합니다."
                    .to_string(),
            ],
            vec![
                "strict 출력".to_string(),
                "보존".to_string(),
                "JSON/code/diff/log/test output은 설명 UX를 적용하지 않고 원문을 유지합니다."
                    .to_string(),
            ],
        ],
        profile.frame,
        profile.theme,
        true,
        width,
    );
    let numbered = shape(
        "실행 흐름은 크게 두 가지로 설명해줘",
        "실행 흐름은 크게 두 가지입니다. codexplain shape/post-response는 입력 텍스트를 바로 후처리합니다. codexplain codex --local-shape는 실제 codex CLI를 실행한 뒤 stdout을 캡처해서 렌더러에 통과시킵니다.",
        &profile,
        width,
    );
    let flow_contract = render_flow_diagram(
        &FlowDiagram::new(
            [
                FlowStep::new(
                    "Very long architecture gateway label that must wrap inside its own flow box",
                ),
                FlowStep::with_branches(
                    "Decision Router",
                    [
                        "strict artifacts are passed through without rewriting".to_string(),
                        "explanations are shaped with Unicode diagrams and quality gates"
                            .to_string(),
                    ],
                ),
                FlowStep::new("Terminal Renderer"),
            ],
            width,
        ),
        profile.frame,
        profile.theme,
    );
    let expansion_contract = render_expansion_diagram(
        &[
            "Request",
            "Policy",
            "Profile",
            "Selector",
            "Renderer",
            "Quality Gate",
        ],
        profile.frame,
        profile.theme,
        width,
    );
    let combined =
        format!("{architecture}\n{long_table}\n{numbered}\n{flow_contract}\n{expansion_contract}");
    let overflow_lines = combined
        .lines()
        .filter(|line| visible_width(line) > width)
        .count();
    let row_dividers = long_table
        .lines()
        .filter(|line| line.starts_with('├') && line.ends_with('┤'))
        .count();
    let architecture_boxes = architecture.matches('┌').count();
    let architecture_panel_overflows = architecture
        .lines()
        .filter(|line| visible_width(line) > width)
        .count();
    let flow_arrows = architecture.matches('▼').count();
    let flow_box_overflows = flow_contract
        .lines()
        .filter(|line| visible_width(line) > width)
        .count();
    let flow_connector_breaks = flow_contract
        .lines()
        .filter(|line| line.contains("----") || line.contains("====") || line.contains("ㅡㅡ"))
        .count();
    let expansion_overflows = expansion_contract
        .lines()
        .filter(|line| visible_width(line) > width)
        .count();
    let numbered_sections = ["1. ", "2. ", "01. ", "02. ", "a. ", "b. "]
        .iter()
        .filter(|needle| numbered.contains(**needle))
        .count();
    let mut score: i32 = 100;
    score -= (overflow_lines as i32) * 20;
    if row_dividers < 3 {
        score -= ((3 - row_dividers) as i32) * 10;
    }
    if architecture_boxes < 6 {
        score -= ((6 - architecture_boxes) as i32) * 8;
    }
    score -= (architecture_panel_overflows as i32) * 20;
    if flow_arrows < 4 {
        score -= ((4 - flow_arrows) as i32) * 8;
    }
    score -= (flow_box_overflows as i32) * 20;
    score -= (flow_connector_breaks as i32) * 20;
    score -= (expansion_overflows as i32) * 20;
    if numbered_sections < 2 {
        score -= ((2 - numbered_sections) as i32) * 10;
    }

    QualityReport {
        width,
        overflow_lines,
        row_dividers,
        architecture_boxes,
        architecture_panel_overflows,
        flow_arrows,
        flow_box_overflows,
        flow_connector_breaks,
        expansion_overflows,
        numbered_sections,
        score: score.clamp(0, 100) as u8,
    }
}

fn print_quality_report(report: &QualityReport) {
    println!("contract=codexplain.quality-check.v1");
    println!("width={}", report.width);
    println!("overflow_lines={}", report.overflow_lines);
    println!("row_dividers={}", report.row_dividers);
    println!("architecture_boxes={}", report.architecture_boxes);
    println!(
        "architecture_panel_overflows={}",
        report.architecture_panel_overflows
    );
    println!("flow_arrows={}", report.flow_arrows);
    println!("flow_box_overflows={}", report.flow_box_overflows);
    println!("flow_connector_breaks={}", report.flow_connector_breaks);
    println!("expansion_overflows={}", report.expansion_overflows);
    println!("numbered_sections={}", report.numbered_sections);
    println!("score={}", report.score);
    println!("result={}", if report.passed() { "pass" } else { "fail" });
}

fn settings_ui() -> io::Result<()> {
    let mut profile = load_profile();
    if !io::stdin().is_terminal() {
        println!("{}", settings_dashboard(&profile, 88));
        println!("non_interactive=run `codexplain settings-ui` in a terminal to edit");
        return Ok(());
    }

    loop {
        print!("\x1b[2J\x1b[H");
        println!("{}", settings_dashboard(&profile, 88));
        println!();
        println!("1 explanation depth   2 architecture view   3 abstraction level");
        println!("4 theme               5 UX density          6 emoji cues");
        println!("7 frame               8 color mode          p preview");
        println!("s save and exit       q quit without saving");
        print!("select > ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => choose_explanation_depth(&mut profile)?,
            "2" => choose_architecture_depth(&mut profile)?,
            "3" => choose_abstraction_level(&mut profile)?,
            "4" => choose_theme(&mut profile)?,
            "5" => choose_ux_density(&mut profile)?,
            "6" => choose_emoji_cues(&mut profile)?,
            "7" => choose_frame(&mut profile)?,
            "8" => choose_color_mode()?,
            "p" | "preview" => settings_preview(&profile)?,
            "s" | "save" | "" => {
                save_profile(&profile)?;
                println!("saved=.codexplain/ux-profile.json");
                return Ok(());
            }
            "q" | "quit" | "exit" => {
                println!("not_saved=.codexplain/ux-profile.json");
                return Ok(());
            }
            other => {
                println!("ignored=unknown menu item '{other}'");
                wait_for_enter()?;
            }
        }
    }
}

fn settings_dashboard(profile: &Profile, width: usize) -> String {
    let title = "🎛️ Codexplain Settings";
    let subtitle =
        "설명 깊이, 추상화 레벨, 색상, 이모지, 출력 밀도를 현재 프로젝트 기준으로 조정합니다.";
    let rows = vec![
        vec![
            "1".to_string(),
            "🧠 Explanation".to_string(),
            profile.explanation_depth.clone(),
            "light / standard / deep".to_string(),
        ],
        vec![
            "2".to_string(),
            "🧭 Architecture".to_string(),
            profile.architecture_depth.clone(),
            "overview / system / internals".to_string(),
        ],
        vec![
            "3".to_string(),
            "🔭 Abstraction".to_string(),
            profile.abstraction_level.clone(),
            "concrete / architecture / strategy".to_string(),
        ],
        vec![
            "4".to_string(),
            "🎨 Theme".to_string(),
            profile.theme.name().to_string(),
            "semantic, restrained color".to_string(),
        ],
        vec![
            "5".to_string(),
            "📐 UX density".to_string(),
            profile.ux_density.to_string(),
            "35 minimal / 65 balanced / 90 rich".to_string(),
        ],
        vec![
            "6".to_string(),
            "✨ Emoji cues".to_string(),
            if profile.emoji_cues { "on" } else { "off" }.to_string(),
            "semantic section and status markers".to_string(),
        ],
        vec![
            "7".to_string(),
            "▣ Frame".to_string(),
            if profile.frame == Frame::Ascii {
                "ascii"
            } else {
                "unicode"
            }
            .to_string(),
            "cross-platform terminal borders".to_string(),
        ],
        vec![
            "8".to_string(),
            "🌈 Color mode".to_string(),
            color_output_name(configured_color_output().unwrap_or(ColorOutput::Terminal))
                .to_string(),
            "ansi / plain / html".to_string(),
        ],
    ];
    format!(
        "{title}\n{subtitle}\n\n{}",
        table(
            &["#", "Control", "Current", "Range"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            width,
        )
    )
}

fn choose_explanation_depth(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "explanationDepth",
        &profile.explanation_depth,
        &["light", "standard", "deep"],
    )? {
        profile.explanation_depth = normalize_explanation_depth(&value, &profile.explanation_depth);
    }
    Ok(())
}

fn choose_architecture_depth(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "architectureDepth",
        &profile.architecture_depth,
        &["overview", "system", "internals"],
    )? {
        profile.architecture_depth =
            normalize_architecture_depth(&value, &profile.architecture_depth);
    }
    Ok(())
}

fn choose_abstraction_level(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "abstractionLevel",
        &profile.abstraction_level,
        &["concrete", "architecture", "strategy"],
    )? {
        profile.abstraction_level = normalize_abstraction_level(&value, &profile.abstraction_level);
    }
    Ok(())
}

fn choose_theme(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "theme",
        profile.theme.name(),
        &[
            "ocean", "forest", "warm", "sunset", "grape", "slate", "rose", "mono", "none",
        ],
    )? {
        profile.theme = Theme::parse(Some(&value));
    }
    Ok(())
}

fn choose_ux_density(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "uxDensity",
        &profile.ux_density.to_string(),
        &["35", "65", "90"],
    )? {
        if let Some(parsed) = parse_control_value(&value) {
            profile.ux_density = parsed;
        }
    }
    Ok(())
}

fn choose_emoji_cues(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "emojiCues",
        if profile.emoji_cues { "on" } else { "off" },
        &["on", "off"],
    )? {
        profile.emoji_cues = !matches!(value.as_str(), "off" | "false" | "0" | "no");
    }
    Ok(())
}

fn choose_frame(profile: &mut Profile) -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "frame",
        if profile.frame == Frame::Ascii {
            "ascii"
        } else {
            "unicode"
        },
        &["unicode", "ascii"],
    )? {
        profile.frame = Frame::parse(Some(&value));
    }
    Ok(())
}

fn choose_color_mode() -> io::Result<()> {
    if let Some(value) = prompt_choice(
        "colorMode",
        color_output_name(configured_color_output().unwrap_or(ColorOutput::Terminal)),
        &["ansi", "plain", "html"],
    )? {
        match value.as_str() {
            "plain" => write_color_config("plain", "plain", "off")?,
            "html" => write_color_config("html", "html", "full")?,
            _ => write_color_config("ansi", "ansi", "semantic")?,
        }
    }
    Ok(())
}

fn settings_preview(profile: &Profile) -> io::Result<()> {
    println!();
    println!(
        "{}",
        shape(
            "이 프로젝트의 아키텍처를 기능 기준으로 설명해줘",
            "Codexplain은 Codex 응답을 받아 strict artifact는 보존하고, 일반 설명은 프로필과 렌더러를 거쳐 터미널 친화적인 설명 UX로 바꿉니다.",
            profile,
            88,
        )
    );
    wait_for_enter()
}

fn wait_for_enter() -> io::Result<()> {
    print!("press Enter > ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

fn statusbar_control(args: &[String]) -> io::Result<()> {
    let action = args.get(1).map(String::as_str).unwrap_or("status");
    match action {
        "status" | "--show" | "show" => {
            print_statusbar_state();
        }
        "on" | "enable" => {
            let root = trusted_statusbar_project_root();
            install_local_codex_project_at(&root)?;
            write_color_config_at(&root, "ansi", "ansi", "semantic")?;
            println!("statusbar=on");
            print_statusbar_state_at(&root);
        }
        "off" | "disable" | "restore" => {
            let root = trusted_statusbar_project_root();
            uninstall_local_codex_project_at(&root, false)?;
            println!("statusbar=off");
            print_statusbar_state_at(&root);
        }
        "set" | "profile" => {
            let root = trusted_statusbar_project_root();
            let mut profile = load_profile_at(&root);
            apply_statusbar_profile_args(args, &mut profile)?;
            save_profile_at(&root, &profile)?;
            if let Some(color) = arg_value(args, "--color-output") {
                apply_statusbar_color_output_at(&root, color)?;
            }
            println!("statusbar=updated");
            print_statusbar_state_at(&root);
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown statusbar action: {other}"),
            ));
        }
    }
    Ok(())
}

fn trusted_statusbar_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn print_statusbar_state() {
    let root = project_path(".");
    print_statusbar_state_at(&root);
}

fn print_statusbar_state_at(root: &Path) {
    let profile = load_profile_at(root);
    let rows = statusbar_state_rows_at(&profile, root);
    println!(
        "{}",
        table(
            &["Control", "Value", "Scope"],
            &rows,
            profile.frame,
            profile.theme,
            true,
            88,
        )
    );
    println!("actions=codexplain statusbar on|off|set");
}

fn statusbar_state_rows_at(profile: &Profile, root: &Path) -> Vec<Vec<String>> {
    vec![
        vec![
            "Power".to_string(),
            if project_local_adapter_present_at(root) {
                "on".to_string()
            } else {
                "off".to_string()
            },
            "project-local".to_string(),
        ],
        vec![
            "Explanation".to_string(),
            profile.explanation_depth.clone(),
            "light/standard/deep".to_string(),
        ],
        vec![
            "Architecture".to_string(),
            profile.architecture_depth.clone(),
            "overview/system/internals".to_string(),
        ],
        vec![
            "Abstraction".to_string(),
            profile.abstraction_level.clone(),
            "concrete/architecture/strategy".to_string(),
        ],
        vec![
            "Expression".to_string(),
            expression_mode(profile).to_string(),
            "code/concept/metaphor".to_string(),
        ],
        vec![
            "Theme".to_string(),
            profile.theme.name().to_string(),
            "profile".to_string(),
        ],
    ]
}

fn project_local_adapter_present_at(root: &Path) -> bool {
    root.join(".codexplain/bin/codex").exists() && root.join(".codexplain/activate").exists()
}

fn expression_mode(profile: &Profile) -> &'static str {
    match profile.style.as_str() {
        "metaphorical" | "metaphor" | "비유" => "metaphor",
        "conceptual" | "concept" | "개념" => "concept",
        _ => "code",
    }
}

fn apply_statusbar_profile_args(args: &[String], profile: &mut Profile) -> io::Result<()> {
    if let Some(theme) = arg_value(args, "--theme") {
        profile.theme = Theme::parse(Some(theme));
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
    if let Some(value) = arg_value(args, "--expression-mode") {
        apply_expression_mode(profile, value)?;
    }
    Ok(())
}

fn apply_expression_mode(profile: &mut Profile, value: &str) -> io::Result<()> {
    match value.trim().to_ascii_lowercase().as_str() {
        "code" | "code-based" | "technical" | "코드" | "기술" => {
            profile.style = "technical".to_string();
            profile.abstraction_level = "concrete".to_string();
        }
        "concept" | "conceptual" | "개념" => {
            profile.style = "conceptual".to_string();
            profile.abstraction_level = "architecture".to_string();
        }
        "metaphor" | "metaphorical" | "비유" => {
            profile.style = "metaphorical".to_string();
            profile.abstraction_level = "strategy".to_string();
        }
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown expression mode: {other}"),
            ));
        }
    }
    Ok(())
}

fn apply_statusbar_color_output_at(root: &Path, value: &str) -> io::Result<()> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ansi" | "terminal" | "color" => write_color_config_at(root, "ansi", "ansi", "semantic"),
        "plain" | "none" | "off" => write_color_config_at(root, "plain", "plain", "off"),
        "html" | "html-chat" | "chat" => write_color_config_at(root, "html", "html", "full"),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown color output: {other}"),
        )),
    }
}

fn prompt_choice(label: &str, current: &str, options: &[&str]) -> io::Result<Option<String>> {
    print!("{label} [{current}] options={} > ", options.join("/"));
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if options.iter().any(|option| *option == value) {
        Ok(Some(value.to_string()))
    } else {
        println!("ignored={label}: unsupported value '{value}'");
        Ok(None)
    }
}

fn install_app_launchers() -> io::Result<()> {
    let app_dir = project_path(".codexplain/app");
    fs::create_dir_all(&app_dir)?;
    let root = env::current_dir()?;
    let bin = root.join("bin/codexplain");
    let mac = app_dir.join("Codexplain Settings.command");
    let mac_statusbar = app_dir.join("Codexplain Status Bar.command");
    let linux = app_dir.join("codexplain-settings.desktop");
    let windows = app_dir.join("codexplain-settings.cmd");

    fs::write(
        &mac,
        format!(
            "#!/usr/bin/env sh\ncd '{}'\nexec '{}' settings-ui\n",
            shell_single_quote(&root.display().to_string()),
            shell_single_quote(&bin.display().to_string())
        ),
    )?;
    set_executable(&mac)?;
    fs::write(
        &mac_statusbar,
        format!(
            "#!/usr/bin/env sh\ncd '{}'\nexec '{}' statusbar status\n",
            shell_single_quote(&root.display().to_string()),
            shell_single_quote(&bin.display().to_string())
        ),
    )?;
    set_executable(&mac_statusbar)?;
    fs::write(
        &linux,
        format!(
            "[Desktop Entry]\nType=Application\nName=Codexplain Settings\nExec={} settings-ui\nTerminal=true\nCategories=Development;\n",
            bin.display()
        ),
    )?;
    fs::write(
        &windows,
        format!(
            "@echo off\r\ncd /d \"{}\"\r\n\"{}\" settings-ui\r\n",
            root.display(),
            bin.display()
        ),
    )?;
    println!("installed={}", app_dir.display());
    println!("mac={}", mac.display());
    println!("mac_statusbar={}", mac_statusbar.display());
    println!("linux={}", linux.display());
    println!("windows={}", windows.display());
    Ok(())
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn usage() -> &'static str {
    "Usage:
  codexplain shape --prompt <text> [--response <text>] [--width <n>] [--emoji-cues|--no-emoji-cues] [--chat-color|--color-output markdown|html|ansi|plain]
  codexplain post-response --prompt <text> [--width <n>] [--emoji-cues|--no-emoji-cues] [--chat-color|--color-output markdown|html|ansi|plain]
  codexplain codex --prompt <text> [--local-shape] [codex exec args...]
  codexplain on|install-codex [--project|--local] [--global] [--session] [--force]
  codexplain off|uninstall-codex [--project|--local] [--global] [--session] [--remove-profile]
  codexplain slash [toggle|on|off|status|settings|help]
  codexplain color on|off|status|rules
  codexplain tui-color on|full|off|status
  codexplain tui-adapter on|full|off|status|apply|build
  codexplain style add <name> --trigger <text> --renderers <tldr,table,flow,pros-cons,formula,cause-effect,problem-diagnosis,indexed,progress> --description <text> [--tone <tone>] [--example <text>]
  codexplain style list|show <name>|preview <name>|remove <name>
  codexplain feedback|rlhf --rating <1-5> --comment <text>
  codexplain profile --show|--theme <name>|--frame <unicode|ascii|fallback|auto>|--index-style <style>|--detail <level>
  codexplain profile --explanation-depth <light|standard|deep>|--architecture-depth <overview|system|internals>|--abstraction-level <concrete|architecture|strategy>
  codexplain profile --detail-scale <0-100>|--ux-density <0-100>|--risk-sensitivity <0-100>
  codexplain statusbar status|on|off|set [--explanation-depth <level>] [--architecture-depth <level>] [--abstraction-level <level>] [--expression-mode <code|concept|metaphor>] [--theme <name>] [--color-output <ansi|plain|html-chat>]
  codexplain
  codexplain settings|settings-ui
  codexplain install-app
  codexplain compat-check
  codexplain quality-check [--width <n>]
  codexplain demo
  codexplain build-size
  codexplain build-clean --target|--patched-codex
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
Color outputs: terminal, ansi, markdown, html, plain. Use --chat-color as an alias for --color-output ansi in Codex CLI.
Scopes: --project/--local writes only this repository's managed Codexplain files and a managed zsh auto-activation block for this exact project root; --global writes only managed guidance under CODEX_HOME; --session prints the current-shell activation command because a child process cannot mutate its parent shell.
Emoji cues: enabled by default as active semantic section/status markers such as 🧭 overview, ✅ success, ⚠️ warning, 🚨 danger, 🔎 evidence, 🛠️ fix, and 🚀 next step. Use --no-emoji-cues or settings-ui to turn them off.
Color toggle: `codexplain color on` forces ANSI text color for Codexplain-shaped exec/review output and best-effort Codex TUI color env; `codexplain color off` restores plain output. `codexplain color rules` shows the semantic-sparse role map so colors do not become decorative noise.
TUI assistant color: `codexplain tui-color on` enables project-local full assistant-message color when a patched Codex binary exists under .codexplain/patched-codex/bin/codex, .codexplain/state/codex-upstream/codex-rs/target/release/codex, or target/debug/codex; `off` disables only that hook.
TUI adapter: `codexplain tui-adapter status` reports project-local shim path, mode, active binary/fallback, patched binary status, rollback, and cleanup instructions. `codexplain tui-adapter build` applies the tracked Codex TUI assistant-color and native `/codexplain` slash patches, then builds only the project-local patched Codex binary.
Slash control: bare `/codexplain` toggles project-local Codexplain UX on/off and is bridged to `codexplain slash toggle`; `/codexplain on|off|status|settings` remain explicit controls. `off` disables the managed AGENTS guidance and color UX while preserving the local shim/native slash bridge; `codexplain off --local` remains the strict uninstall path.
Status bar control: `codexplain statusbar` is the Rust control surface used by local app launchers. It toggles only project-local Codexplain files, updates profile/config controls, and leaves unrelated global Codex settings untouched.
Settings UI: bare `codexplain`, `codexplain settings`, and `codexplain settings-ui` open a dependency-free Rust terminal UI for theme, frame, depth, abstraction, UX density, emoji cues, and color mode; `codexplain install-app` writes lightweight macOS/Linux/Windows launchers under .codexplain/app.
Compatibility gate: `codexplain compat-check` validates project-local OMX/harness safety, managed on/off scopes, strict artifact preservation, ignored harness state, and width-safe renderer contracts.
Quality gate: `codexplain quality-check --width 88` fails if generated output overflows, table body row dividers disappear, architecture boxes overflow or are too sparse, flow arrows/connectors break, expansion diagrams overflow, or two-path explanations are not numbered.
Build cleanup: `codexplain build-clean --patched-codex` removes only the ignored project-local patched Codex Cargo target directory. The persisted `.codexplain/patched-codex/bin/codex` binary remains available so native `/codexplain` and TUI color do not require keeping a multi-GB Cargo cache. The shim runs this cleanup opportunistically on launch and does not auto-build the patched TUI unless `CODEXPLAIN_TUI_AUTO_BUILD=1`.
Index styles: decimal, zero-padded, alpha-lower, alpha-upper, roman-lower, roman-upper"
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("settings-ui");
    match command {
        "shape" => {
            let profile = load_profile_for_args(&args);
            let mode = color_output_mode(&args);
            let prompt = arg_value(&args, "--prompt").unwrap_or("");
            let response = arg_value(&args, "--response")
                .map(str::to_string)
                .unwrap_or_else(read_stdin_if_needed);
            let width = arg_value(&args, "--width")
                .and_then(|v| v.parse().ok())
                .unwrap_or(100);
            println!(
                "{}",
                shape_for_output(prompt, &response, &profile, width, mode)
            );
        }
        "post-response" => post_response(&args),
        "codex" => std::process::exit(run_codex(&args[1..])),
        "on" | "install-codex" | "init" => {
            if let Err(error) = install_codex_project(&args) {
                eprintln!("failed to install Codexplain files: {error}");
                std::process::exit(1);
            }
        }
        "slash" => {
            if let Err(error) = slash_control(&args) {
                eprintln!("failed to run Codexplain slash control: {error}");
                std::process::exit(1);
            }
        }
        "uninstall-codex" | "off" => {
            if let Err(error) = uninstall_codex_project(&args) {
                eprintln!("failed to uninstall Codexplain files: {error}");
                std::process::exit(1);
            }
        }
        "color" => {
            if let Err(error) = color_command(&args) {
                eprintln!("failed to update Codexplain color: {error}");
                std::process::exit(1);
            }
        }
        "tui-color" => {
            if let Err(error) = tui_color_command(&args) {
                eprintln!("failed to update Codexplain TUI color: {error}");
                std::process::exit(1);
            }
        }
        "tui-adapter" => {
            if let Err(error) = tui_adapter_command(&args) {
                eprintln!("failed to update Codexplain TUI adapter: {error}");
                std::process::exit(1);
            }
        }
        "guide" => guide(&args),
        "style" => {
            if let Err(error) = style_command(&args) {
                eprintln!("failed to manage Codexplain style: {error}");
                std::process::exit(1);
            }
        }
        "feedback" => {
            if let Err(error) = feedback(&args, false) {
                eprintln!("failed to save feedback: {error}");
                std::process::exit(1);
            }
        }
        "rlhf" => {
            if let Err(error) = feedback(&args, true) {
                eprintln!("failed to save rlhf feedback: {error}");
                std::process::exit(1);
            }
        }
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
        "statusbar" => {
            if let Err(error) = statusbar_control(&args) {
                eprintln!("failed to run Codexplain statusbar control: {error}");
                std::process::exit(1);
            }
        }
        "settings" | "settings-ui" => {
            if let Err(error) = settings_ui() {
                eprintln!("failed to run Codexplain settings UI: {error}");
                std::process::exit(1);
            }
        }
        "install-app" => {
            if let Err(error) = install_app_launchers() {
                eprintln!("failed to install Codexplain app launchers: {error}");
                std::process::exit(1);
            }
        }
        "compat-check" => compat_check(),
        "quality-check" => {
            let width = arg_value(&args, "--width")
                .and_then(|value| value.parse().ok())
                .unwrap_or(88);
            let report = quality_report(width);
            print_quality_report(&report);
            if !report.passed() {
                std::process::exit(1);
            }
        }
        "storage-check" => storage_check(&args),
        "build-size" => build_size(),
        "build-clean" => {
            if let Err(error) = build_clean(&args) {
                eprintln!("failed to clean build artifacts: {error}");
                std::process::exit(1);
            }
        }
        "pros-cons" => {
            let profile = load_profile_for_args(&args);
            let width = arg_value(&args, "--width")
                .and_then(|value| value.parse().ok())
                .unwrap_or(100);
            println!("{}", pros_cons_for_width(&profile, width));
        }
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
                pros_cons_for_width(&profile, 100),
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
            "  \"riskSensitivity\": {},\n",
            "  \"emojiCues\": {}\n",
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
        profile.risk_sensitivity,
        profile.emoji_cues
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
        let plain = strip_ansi(&output);
        assert!(plain.contains("JS / Node"));
        assert!(plain.contains("Rust"));
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
            "┌───────────┬───────────────┬────────────────────┬───────────────────────┐"
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
                "┌───────────┬───────────────┬────────────────────┬───────────────────────┐",
                "│ 선택지    │ 장점          │ 단점               │ 적합한 때             │",
                "├───────────┼───────────────┼────────────────────┼───────────────────────┤",
                "│ JS / Node │ 빠른 수정     │ 런타임 의존성      │ UX 실험과 피드백 루프 │",
                "│           │ provider 연동 │ 단일 바이너리 약함 │                       │",
                "│           │ JSON 처리     │                    │                       │",
                "├───────────┼───────────────┼────────────────────┼───────────────────────┤",
                "│ Rust      │ 단일 바이너리 │ 초기 구현 비용     │ 안정화된 CLI core     │",
                "│           │ 빠른 시작     │ provider 실험 비용 │                       │",
                "│           │ 낮은 메모리   │                    │                       │",
                "└───────────┴───────────────┴────────────────────┴───────────────────────┘",
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
                "│ 설명   : 초기에는 반복속도, 제품화에는 배포/안정성 가중치가         │",
                "│          커집니다.                                                  │",
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
        assert!(output.contains("distribution, safety)"));
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
        assert!(output.contains("1. 첫째입니다"));
        assert!(output.contains("2. 둘째입니다"));
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
            ["1. 첫 번째 설명", "2. 두 번째 설명", "3. 세 번째 설명",].join("\n")
        );
    }

    #[test]
    fn indexed_list_keeps_numbered_items_compact_and_uses_wrapped_continuations() {
        let output = indexed(
            &[
                "첫 번째 원인과 결과".to_string(),
                "두 번째 대안과 한계".to_string(),
            ],
            Frame::Unicode,
            Theme::None,
            60,
            IndexStyle::Decimal,
        );

        assert!(
            output.contains("첫 번째 원인과 결과\n2. 두 번째 대안과 한계"),
            "{output}"
        );
        assert!(!output.contains("\n\n2. "), "{output}");
    }

    #[test]
    fn notion_style_components_parse_and_select_from_english_and_korean_hints() {
        assert_eq!(parse_ux_component("toggle"), Some(UxComponent::Toggle));
        assert_eq!(parse_ux_component("quote"), Some(UxComponent::Quote));
        assert_eq!(parse_ux_component("divider"), Some(UxComponent::Divider));

        let profile = Profile {
            ux_density: 50,
            ..Profile::default()
        };
        let components = requested_ux_components(
            "노션처럼 토글, 인용, 구분선으로 설명해줘",
            "핵심은 renderer-owned block입니다.",
            &profile,
        );

        assert!(components.contains(&UxComponent::Toggle), "{components:?}");
        assert!(components.contains(&UxComponent::Quote), "{components:?}");
        assert!(components.contains(&UxComponent::Divider), "{components:?}");
    }

    #[test]
    fn notion_style_blocks_render_width_safe_static_terminal_output() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let summary = "Codexplain은 Notion식 블록을 정적 터미널 출력으로 바꿔 긴 설명을 접힌 요약, 인용, 구분선으로 나눕니다.";

        let toggle = notion_toggle(&profile, summary, 54);
        let quote = notion_quote(&profile, summary, 54);
        let divider = notion_divider(&profile, 12);
        let narrow_quote = notion_quote(&profile, "abcdefghijklmnopqrstuvwxyz", 12);

        assert!(toggle.contains("▸ 핵심 접기"), "{toggle}");
        assert!(quote.lines().all(|line| line.starts_with("│ ")), "{quote}");
        assert_eq!(visible_width(&divider), 12);
        assert_visible_lines_fit(&toggle, 54);
        assert_visible_lines_fit(&quote, 54);
        assert_visible_lines_fit(&divider, 12);
        assert_visible_lines_fit(&narrow_quote, 12);
    }

    #[test]
    fn rich_ux_prompt_can_compose_notion_blocks_without_replacing_main_renderer() {
        let profile = Profile {
            theme: Theme::None,
            ux_density: 80,
            ..Profile::default()
        };
        let output = shape(
            "아키텍처를 표와 흐름도로 설명하고 notion UX도 모두 넣어줘",
            "Codexplain은 Rust renderer와 project-local adapter로 설명 UX를 개선합니다.",
            &profile,
            88,
        );

        assert!(output.contains("• TLDR"), "{output}");
        assert!(output.contains("━━━━━━━━"), "{output}");
        assert!(output.contains("Prompt Input"), "{output}");
        assert!(output.contains("▸ 핵심 접기"), "{output}");
        assert!(output.contains("│ Codexplain은 Rust renderer"), "{output}");
        assert!(output.contains("────"), "{output}");
        assert_visible_lines_fit(&output, 88);
    }

    #[test]
    fn emoji_cues_are_active_semantic_section_markers() {
        let profile = Profile {
            theme: Theme::None,
            ux_density: 80,
            ..Profile::default()
        };

        let badge = status_badge(&profile, "완료 100%");
        let callout = attention_callout(&profile, "실패 원인을 확인해야 합니다.", 64);
        let next = next_action_footer(&profile, 64);
        let confidence = confidence_meter(&profile, "통과", 64);
        let combined = format!("{badge}\n{callout}\n{next}\n{confidence}");

        assert!(badge.starts_with("✅ [PASS]"), "{badge}");
        assert!(callout.contains("⚠️ 중요"), "{callout}");
        assert!(next.contains("🚀 다음 행동:"), "{next}");
        assert!(confidence.contains("🔎 확신도"), "{confidence}");
        assert_eq!(ux_emoji_for_role(&profile, "danger"), "🚨");
        assert_eq!(ux_emoji_for_role(&profile, "fix"), "🛠️");
        assert_eq!(ux_emoji_for_role(&profile, "architecture"), "🧭");
        assert_visible_lines_fit(&combined, 64);
    }

    #[test]
    fn emoji_cues_can_be_disabled_without_losing_text_labels() {
        let profile = Profile {
            theme: Theme::None,
            emoji_cues: false,
            ..Profile::default()
        };

        let next = next_action_footer(&profile, 64);
        let confidence = confidence_meter(&profile, "통과", 64);

        assert!(next.contains("• 다음 행동:"), "{next}");
        assert!(confidence.contains("• 확신도"), "{confidence}");
        assert!(!next.contains("🚀"), "{next}");
        assert!(!confidence.contains("🔎"), "{confidence}");
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
        assert!(lines[0].starts_with("1. "));
        assert!(lines[1].starts_with("   "));
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

        assert!(zero_padded.contains("01. item1"));
        assert!(zero_padded.contains("12. item12"));
        assert!(alpha.contains(" A. item1"));
        assert!(alpha.contains(" Z. item26"));
        assert!(alpha.contains("AA. item27"));
        assert!(roman.contains("  i. one"));
        assert!(roman.contains(" iv. four"));
    }

    #[test]
    fn indexed_list_uses_plain_numbering_even_with_ascii_frame() {
        let output = indexed(
            &["first".to_string(), "second".to_string()],
            Frame::Ascii,
            Theme::None,
            40,
            IndexStyle::Decimal,
        );

        assert!(output.contains("1. first"));
        assert!(output.contains("2. second"));
        assert!(!output.contains('|'));
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
    fn forced_color_overrides_non_interactive_terminal_defaults() {
        assert_eq!(
            Theme::Ocean.apply_terminal_policy(|key| match key {
                "TERM" => Some("dumb".to_string()),
                "CODEXPLAIN_COLOR" => Some("always".to_string()),
                _ => None,
            }),
            Theme::Ocean
        );
        assert_eq!(
            Theme::Grape.apply_terminal_policy(|key| match key {
                "TERM" => Some("dumb".to_string()),
                "CLICOLOR_FORCE" => Some("1".to_string()),
                _ => None,
            }),
            Theme::Grape
        );
        assert_eq!(
            Theme::Sunset.apply_terminal_policy(|key| match key {
                "NO_COLOR" => Some("1".to_string()),
                "CODEXPLAIN_COLOR" => Some("always".to_string()),
                _ => None,
            }),
            Theme::Sunset
        );
        assert_eq!(
            Theme::Sunset.apply_terminal_policy(|key| match key {
                "CODEXPLAIN_NO_COLOR" => Some("true".to_string()),
                "CODEXPLAIN_COLOR" => Some("always".to_string()),
                _ => None,
            }),
            Theme::None
        );
    }

    #[test]
    fn remove_guidance_block_removes_only_codexplain_section() {
        let input = "before

<!-- CODEXPLAIN:START -->
managed
<!-- CODEXPLAIN:END -->

after
";
        assert_eq!(
            remove_guidance_block(input),
            "before

after
"
        );
    }

    #[test]
    fn remove_guidance_block_leaves_unmanaged_content_unchanged() {
        let input = "before
after
";
        assert_eq!(remove_guidance_block(input), input);
    }

    #[test]
    fn replace_guidance_block_preserves_user_agents_content() {
        let input = "# Team rules

- keep this repo-specific rule

<!-- CODEXPLAIN:START -->
old managed block
<!-- CODEXPLAIN:END -->

## More project policy

Do not remove this.
";
        let output = replace_guidance_block(input, CODEX_GUIDANCE);

        assert!(output.contains("# Team rules"));
        assert!(output.contains("- keep this repo-specific rule"));
        assert!(output.contains("## More project policy"));
        assert!(output.contains("Do not remove this."));
        assert!(!output.contains("old managed block"));
        assert_eq!(output.matches("CODEXPLAIN:START").count(), 1);
        assert_eq!(output.matches("CODEXPLAIN:END").count(), 1);
    }

    #[test]
    fn custom_style_parser_sanitizes_and_loads_renderer_plan() {
        let style = parse_custom_style(
            "name: research-card!\ntrigger: 연구 카드\nrenderers: tldr,table,formula,cause-effect\nbody:\n배경, 근거, 한계, 다음 행동을 분리한다.\n",
        )
        .unwrap();

        assert_eq!(style.name, "research-card");
        assert_eq!(style.trigger, "연구 카드");
        assert_eq!(
            style.renderers,
            vec![
                RendererKind::TldrProse,
                RendererKind::Table,
                RendererKind::Formula,
                RendererKind::CauseEffect
            ]
        );
        assert!(style.body.contains("근거"));
        assert_eq!(style.tone, "technical");
    }

    #[test]
    fn custom_style_section_uses_table_interface_without_losing_rule_text() {
        let style = CustomStyle {
            name: "research-card".to_string(),
            trigger: "연구 카드".to_string(),
            renderers: vec![RendererKind::TldrProse, RendererKind::Table],
            body: "배경, 근거, 한계, 다음 행동을 분리한다.".to_string(),
            tone: "research".to_string(),
            example: "연구 카드로 설명".to_string(),
        };

        let output = render_custom_style_section(
            &[style],
            &Profile {
                theme: Theme::None,
                ..Profile::default()
            },
            120,
        );

        assert!(output.contains("┌"));
        assert!(output.contains("설명방식"));
        assert!(output.contains("research"));
        assert!(output.contains("연구 카드"));
        assert!(output.contains("tldr,table"));
        assert!(output.contains("research"));
        assert!(output.contains("다음 행동"));
        assert_visible_lines_fit(&output, 120);
    }

    #[test]
    fn codex_shim_assets_are_project_local_and_reversible() {
        assert!(CODEX_SHIM_SH.contains("CODEXPLAIN_PROJECT_DIR"));
        assert!(
            CODEX_SHIM_SH.contains(r#"ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"#)
        );
        assert!(CODEX_SHIM_SH.contains(r#"export CODEXPLAIN_PROJECT_DIR="$ROOT""#));
        assert!(exports_or_forwards_local_shape(CODEX_SHIM_SH));
        assert!(CODEX_SHIM_SH.contains("codex --local-shape"));
        assert!(CODEX_SHIM_SH.contains("codexplain_auto_update"));
        assert!(CODEX_SHIM_SH.contains("CODEXPLAIN_AUTO_UPDATE"));
        assert!(CODEX_SHIM_SH.contains("dirty_paths"));
        assert!(CODEX_SHIM_SH.contains(".codexplain/config.json"));
        assert!(CODEX_SHIM_SH.contains("pull --ff-only"));
        assert!(CODEX_SHIM_SH.contains("cargo build --release"));
        assert!(CODEX_SHIM_SH.contains("codexplain_ensure_patched_tui"));
        assert!(CODEX_SHIM_SH.contains("tui-adapter build"));
        assert!(CODEX_SHIM_SH.contains("FORCE_COLOR=3"));
        assert!(CODEX_SHIM_SH.contains("NO_COLOR=1"));
        assert!(ACTIVATE_SH.contains(
            r#"CODEXPLAIN_PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE:-$0}")/.." && pwd)"#
        ));
        assert!(ACTIVATE_SH.contains("export CODEXPLAIN_PROJECT_DIR"));
        assert!(ACTIVATE_SH.contains(".codexplain/bin:$PATH"));
        assert!(ACTIVATE_SH.contains("CODEXPLAIN_COLOR_OUTPUT=ansi"));
        assert!(LOCAL_README.contains("source .codexplain/activate"));
        assert!(LOCAL_README.contains("codexplain color on"));
        assert!(LOCAL_README.contains("codexplain style add"));
    }

    #[test]
    fn compat_matrix_documents_harness_safe_scopes_without_committed_state() {
        assert!(usage().contains("codexplain compat-check"));
        assert!(LOCAL_README.contains("codexplain compat-check"));
        assert!(managed_project_files().contains(&".codexplain/bin/codex"));
        assert!(managed_project_files().contains(&".codexplain/activate"));
        assert!(managed_project_files().contains(&".codexplain/post-response"));
        assert!(!managed_project_files()
            .iter()
            .any(|path| path.contains("state")));
        assert!(!managed_project_files()
            .iter()
            .any(|path| path.contains("harness")));
        assert!(!managed_project_files()
            .iter()
            .any(|path| path.contains("oh-my-codex")));
        assert!(CODEX_SHIM_SH.contains(r#"export CODEXPLAIN_PROJECT_DIR="$ROOT""#));
        assert!(exports_or_forwards_local_shape(CODEX_SHIM_SH));
        assert!(session_activation_hint().contains("source ./.codexplain/activate"));
        assert!(!session_activation_hint().contains("Installed"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("CODEXPLAIN:START"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("CODEXPLAIN:END"));
        assert!(CODEX_GUIDANCE.contains("capability boundary"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("capability boundaries"));
        assert!(CODEX_GUIDANCE.contains("Use English by default"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("use English by default"));
    }

    #[test]
    fn compat_gate_preserves_strict_artifacts_and_renderer_quality() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let json = r#"{"ok":true,"mode":"strict"}"#;

        assert_eq!(shape("valid JSON만 출력", json, &profile, 88), json);
        assert!(quality_report(88).passed());
    }

    #[test]
    fn statusbar_control_surface_exposes_project_local_depth_and_expression_controls() {
        assert!(usage().contains("codexplain statusbar status|on|off|set"));
        assert!(usage().contains("--expression-mode <code|concept|metaphor>"));
        assert!(LOCAL_README.contains("codexplain statusbar status"));
        assert!(LOCAL_README.contains("project-local status control surface"));

        let mut profile = Profile::default();
        apply_expression_mode(&mut profile, "metaphor").unwrap();
        assert_eq!(profile.style, "metaphorical");
        assert_eq!(profile.abstraction_level, "strategy");
        assert_eq!(expression_mode(&profile), "metaphor");

        apply_statusbar_profile_args(
            &[
                "statusbar".to_string(),
                "set".to_string(),
                "--explanation-depth".to_string(),
                "light".to_string(),
                "--architecture-depth".to_string(),
                "internals".to_string(),
                "--abstraction-level".to_string(),
                "concrete".to_string(),
                "--theme".to_string(),
                "forest".to_string(),
            ],
            &mut profile,
        )
        .unwrap();

        assert_eq!(profile.explanation_depth, "light");
        assert_eq!(profile.architecture_depth, "internals");
        assert_eq!(profile.abstraction_level, "concrete");
        assert_eq!(profile.theme, Theme::Forest);

        let root = env::temp_dir().join(format!(
            "codexplain-statusbar-row-test-{}",
            std::process::id()
        ));
        let rows = statusbar_state_rows_at(&profile, &root);
        let controls: Vec<&str> = rows.iter().map(|row| row[0].as_str()).collect();
        assert_eq!(
            controls,
            vec![
                "Power",
                "Explanation",
                "Architecture",
                "Abstraction",
                "Expression",
                "Theme"
            ]
        );
        assert!(rows
            .iter()
            .any(|row| row[0] == "Explanation" && row[1] == "light"));
        assert!(rows
            .iter()
            .any(|row| row[0] == "Architecture" && row[1] == "internals"));
        assert!(rows
            .iter()
            .any(|row| row[0] == "Abstraction" && row[1] == "concrete"));
        assert!(rows
            .iter()
            .any(|row| row[0] == "Expression" && row[1] == "metaphor"));
        assert!(rows
            .iter()
            .any(|row| row[0] == "Theme" && row[1] == "forest"));
    }

    #[test]
    fn statusbar_write_controls_resolve_to_trusted_manifest_root() {
        let root = trusted_statusbar_project_root();
        assert_eq!(root, PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        assert!(
            root.is_absolute(),
            "statusbar trusted root must be absolute"
        );
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("rust/codexplain.rs").exists());

        let rows = statusbar_state_rows_at(&Profile::default(), &root);
        assert!(rows
            .iter()
            .any(|row| row[0] == "Power" && row[2] == "project-local"));
    }

    #[test]
    fn install_app_launchers_include_mac_statusbar_entry_without_second_renderer() {
        assert!(usage().contains("Settings UI"));
        assert!(usage().contains("Status bar control"));
        assert!(LOCAL_README.contains("codexplain install-app"));
    }

    #[test]
    fn slash_control_guidance_and_usage_are_discoverable() {
        assert!(usage().contains("codexplain slash [toggle|on|off|status|settings|help]"));
        assert!(usage().contains("codexplain settings|settings-ui"));
        assert!(usage().contains("managed zsh auto-activation block"));
        assert!(CODEX_GUIDANCE.contains("`/codexplain`"));
        assert!(CODEX_GUIDANCE.contains("slash toggle"));
        assert!(CODEX_GUIDANCE.contains("/codexplain on"));
        assert!(CODEX_GUIDANCE.contains("./bin/codexplain slash on"));
        assert!(CODEX_GUIDANCE.contains("codexplain settings-ui"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("codexplain slash <action>"));
        assert!(GLOBAL_CODEX_GUIDANCE.contains("Bare `/codexplain` toggles"));
        let patch = fs::read_to_string(project_path("patches/codex-tui-codexplain-slash.patch"))
            .expect("slash patch should be readable");
        assert!(patch.contains("run_codexplain_slash_command(\"toggle\")"));
        assert!(patch.contains("[toggle|on|off|status|settings|help]"));
        assert!(patch.contains("\"settings\" | \"setting\""));
        assert!(slash_help().contains("settings"));
        assert!(slash_enabled_guide().contains("settings UI"));
    }

    #[test]
    fn settings_dashboard_exposes_cross_platform_controls() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let dashboard = settings_dashboard(&profile, 88);

        assert!(dashboard.contains("🎛️ Codexplain Settings"));
        assert!(dashboard.contains("🧠 Explanation"));
        assert!(dashboard.contains("🧭 Architecture"));
        assert!(dashboard.contains("🔭 Abstraction"));
        assert!(dashboard.contains("✨ Emoji cues"));
        assert!(dashboard.contains("🌈 Color mode"));
        assert_visible_lines_fit(&dashboard, 88);
    }

    #[test]
    fn shell_autoload_block_is_managed_per_project_and_removable() {
        let root = PathBuf::from("/tmp/codexplain-demo");
        let block = shell_autoload_block(&root);
        assert!(block.contains(SHELL_AUTO_START));
        assert!(block.contains("add-zsh-hook chpwd"));
        assert!(block.contains("CODEXPLAIN_AUTO_ACTIVATING"));
        assert!(block.contains("_codexplain_root='/tmp/codexplain-demo'"));
        assert!(block.contains(".codexplain/bin/codex"));
        assert!(block.contains("if [ -x \"$_codexplain_root/.codexplain/bin/codex\" ]; then"));
        assert!(!block.contains("[ \"${CODEXPLAIN_PROJECT_DIR:-}\" != \"$_codexplain_root\" ]"));
        assert!(block.contains("CODEXPLAIN_PREV_CODEX_ALIAS=$(alias codex"));
        assert!(block.contains("alias codex=\"$_codexplain_root/.codexplain/bin/codex\""));
        assert!(block.contains("eval \"alias $CODEXPLAIN_PREV_CODEX_ALIAS\""));
        assert!(!block.contains(". .codexplain/activate"));
        assert!(!block.contains(":'/tmp/codexplain-demo'/.codexplain/bin"));
        assert!(block.contains("codexplain off --local"));

        let current = format!("before\n\n{block}\nafter\n");
        let next = remove_shell_autoload_block_for_root(&current, &root);
        assert_eq!(next, "before\n\nafter\n");
    }

    #[test]
    fn slash_control_output_is_minimal_for_tui() {
        assert!(usage().contains("bare `/codexplain` toggles"));
        assert!(usage().contains("Emoji cues:"));
        assert!(!usage().contains("contract=codexplain.slash.v1"));
    }

    #[test]
    fn strict_slash_off_removes_only_codexplain_managed_local_state() {
        let root = env::temp_dir().join(format!(
            "codexplain-slash-strict-off-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".codexplain/styles")).unwrap();
        fs::write(root.join("AGENTS.md"), "user notes\n").unwrap();
        install_local_codex_project_at(&root).unwrap();
        fs::write(root.join(".codexplain/ux-profile.json"), "{}").unwrap();
        fs::write(
            root.join(".codexplain/styles/custom.style"),
            "name: custom\n",
        )
        .unwrap();
        fs::write(root.join("keep.txt"), "keep").unwrap();

        uninstall_local_codex_project_strict_at(&root).unwrap();

        assert!(!root.join(".codexplain").exists());
        assert_eq!(
            fs::read_to_string(root.join("AGENTS.md")).unwrap(),
            "user notes\n"
        );
        assert_eq!(fs::read_to_string(root.join("keep.txt")).unwrap(), "keep");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn slash_off_preserves_native_control_bridge_and_disables_ux() {
        let root = env::temp_dir().join(format!(
            "codexplain-slash-preserve-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("AGENTS.md"), "user notes\n").unwrap();
        install_local_codex_project_at(&root).unwrap();

        disable_project_local_codexplain_preserve_slash_at(&root).unwrap();

        assert!(root.join(".codexplain/bin/codex").exists());
        assert!(root.join(".codexplain/activate").exists());
        assert!(root.join(".codexplain/config.json").exists());
        assert!(!fs::read_to_string(root.join("AGENTS.md"))
            .unwrap()
            .contains(CODEX_GUIDANCE_START));
        let config = fs::read_to_string(root.join(".codexplain/config.json")).unwrap();
        assert!(
            config.contains("\"defaultColorOutput\": \"plain\""),
            "{config}"
        );
        assert!(
            config.contains("\"tuiAssistantColor\": \"off\""),
            "{config}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn install_preserves_existing_codexplain_config() {
        let root = env::temp_dir().join(format!(
            "codexplain-config-preserve-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".codexplain")).unwrap();
        fs::write(
            root.join(".codexplain/config.json"),
            r#"{"schemaVersion":1,"defaultColorOutput":"plain","custom":true}"#,
        )
        .unwrap();

        install_local_codex_project_at(&root).unwrap();

        let config = fs::read_to_string(root.join(".codexplain/config.json")).unwrap();
        assert!(
            config.contains(r#""defaultColorOutput":"plain""#),
            "{config}"
        );
        assert!(config.contains(r#""custom":true"#), "{config}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn codex_capture_policy_preserves_tui_passthrough() {
        assert!(prepare_codex_args(Vec::new(), "").is_empty());
        assert_eq!(
            prepare_codex_args(Vec::new(), "이 프로젝트 설명"),
            vec!["exec".to_string(), "이 프로젝트 설명".to_string()]
        );
        assert!(should_capture_codex_output(&["exec".to_string()]));
        assert!(should_capture_codex_output(&["review".to_string()]));
        assert!(!should_capture_codex_output(&[]));
        assert!(!should_capture_codex_output(&[
            "이 프로젝트 설명".to_string()
        ]));
        assert!(local_config_json("plain", "plain", "off")
            .contains("\"defaultColorOutput\": \"plain\""));
        assert!(
            local_config_json("ansi", "ansi", "full").contains("\"chatHighlightOutput\": \"ansi\"")
        );
        assert!(
            local_config_json("ansi", "ansi", "full").contains("\"tuiAssistantColor\": \"full\"")
        );
    }

    #[test]
    fn tui_adapter_status_report_explains_reversible_project_local_fallback() {
        let output = tui_adapter_status_report("full", "not-built".to_string());

        assert!(output.contains("Codexplain TUI adapter"), "{output}");
        assert!(output.contains("scope: project-local only"), "{output}");
        assert!(output.contains("mode: full"), "{output}");
        assert!(output.contains(".codexplain/bin/codex"), "{output}");
        assert!(
            output.contains("fallback: stock Codex binary via PATH"),
            "{output}"
        );
        assert!(output.contains("patchedCodex: not-built"), "{output}");
        assert!(
            output.contains("exec/review shaping remains available"),
            "{output}"
        );
        assert!(output.contains("codexplain tui-adapter off"), "{output}");
        assert!(output.contains("codexplain off --local"), "{output}");
        assert!(output.contains("build-clean --patched-codex"), "{output}");
    }

    #[test]
    fn tui_adapter_docs_and_help_are_discoverable_without_global_changes() {
        assert!(usage().contains("codexplain tui-adapter on|full|off|status|apply|build"));
        assert!(LOCAL_README.contains("codexplain tui-adapter on"));
        assert!(LOCAL_README.contains("codexplain tui-adapter status"));
        assert!(LOCAL_README.contains("codexplain tui-adapter build"));
        assert!(LOCAL_README.contains("codex-tui-codexplain-slash.patch"));
        assert!(LOCAL_README
            .contains("builds the project-local patched Codex TUI binary only when it is missing"));
        assert!(LOCAL_README.contains("project-local patched Codex"));
        assert!(LOCAL_README.contains("binary"));
    }

    #[test]
    fn tui_adapter_on_aliases_existing_enable_behavior() {
        assert_eq!(
            parse_tui_adapter_action("on").unwrap(),
            parse_tui_adapter_action("enable").unwrap()
        );
        assert_eq!(
            parse_tui_adapter_action("on").unwrap(),
            TuiAdapterAction::EnableFull
        );
    }

    #[test]
    fn tui_adapter_tracks_color_and_native_slash_patches() {
        let paths = codex_tui_patch_paths();
        assert!(paths
            .iter()
            .any(|path| path.ends_with("patches/codex-tui-assistant-color.patch")));
        assert!(paths
            .iter()
            .any(|path| path.ends_with("patches/codex-tui-codexplain-slash.patch")));
        assert!(
            tui_adapter_status_report("semantic", "not-built".to_string())
                .contains("/codexplain slash patches")
        );
        let color_patch =
            fs::read_to_string(project_path("patches/codex-tui-assistant-color.patch"))
                .expect("color patch should be readable");
        assert!(
            color_patch.contains("codexplain_style_hyperlink_lines"),
            "{color_patch}"
        );
        assert!(
            color_patch.contains("codexplain_tui_color_enabled"),
            "{color_patch}"
        );
        assert!(
            color_patch.contains("codexplain_semantic_style"),
            "{color_patch}"
        );
    }

    #[test]
    fn markdown_output_converts_ansi_to_text_badge_highlight() {
        let profile = Profile {
            theme: Theme::Sunset,
            ..Profile::default()
        };
        let output = shape_for_output(
            "아키텍처를 표로 설명해줘",
            "Codexplain은 Renderer와 Policy로 구성되고 JSON/code/diff/log/test output을 보존합니다.",
            &profile,
            80,
            ColorOutput::Markdown,
        );

        assert!(!output.contains("<span"), "{output}");
        assert!(!output.contains("\x1b["), "{output}");
        assert!(output.contains("**[REF]** `CODEXPLAIN`"), "{output}");
        assert!(
            output.contains("**[REF]** `JSON/code/diff/log/test`"),
            "{output}"
        );
    }

    #[test]
    fn explicit_html_output_converts_ansi_to_html_spans() {
        let profile = Profile {
            theme: Theme::Sunset,
            ..Profile::default()
        };
        let output = shape_for_output(
            "간단히 설명해줘",
            "본문에도 채팅 색상이 들어갑니다.",
            &profile,
            80,
            ColorOutput::Html,
        );

        assert!(output.starts_with(r#"<pre class="codexplain-chat-color""#));
        assert!(output.contains(r#"<span style="color: #"#));
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn chat_color_output_preserves_strict_artifacts() {
        let profile = Profile {
            theme: Theme::Sunset,
            ..Profile::default()
        };
        let output = shape_for_output(
            "JSON만",
            r#"{"ok":true}"#,
            &profile,
            80,
            ColorOutput::Markdown,
        );

        assert_eq!(output, r#"{"ok":true}"#);
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
        assert!(combined.contains("1. 첫 번째 설명"));
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
                "──────▼──────",
                "┌─────┴─────┐",
                "│ 정책 검사 │",
                "└─────┬─────┘",
                "──────▼──────",
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
                "──────▼──────",
                "┌─────┴─────┐",
                "│ Policy    │",
                "└─────┬─────┘",
                "──────▼──────",
                "      ├─▶ JSON safe",
                "      └─▶ Explain",
                "──────▼──────",
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
    fn architecture_flow_boxes_wrap_long_labels_and_use_full_width_connectors() {
        let diagram = FlowDiagram::new(
            [
                FlowStep::new("User / Codex Prompt"),
                FlowStep::new("Real Codex Runner\nstdout/stderr/exit 보존"),
                FlowStep::new("Renderer Selector\ntable/flow/risk/progress 선택"),
                FlowStep::new("Terminal Renderer\nANSI + Unicode + wrapping"),
            ],
            25,
        );
        let output = render_flow_diagram(&diagram, Frame::Unicode, Theme::None);

        assert_visible_lines_fit(&output, 25);
        assert!(output.contains("│ stdout/stderr/exit    │"), "{output}");
        assert!(output.contains("보존"), "{output}");
        assert!(output.contains("│ table/flow/risk/progr │"), "{output}");
        assert!(output.contains("│ ess 선택              │"), "{output}");
        assert!(
            output
                .lines()
                .any(|line| line == "────────────▼────────────"),
            "{output}"
        );
        assert!(!output.lines().any(|line| line.trim() == "▼"), "{output}");
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
    fn post_output_repairs_hand_drawn_unicode_tables_without_body_dividers() {
        let raw = [
            "설명",
            "┌──────┬────────┐",
            "│ 영역 │ 목적   │",
            "├──────┼────────┤",
            "│ 평가 │ 승격   │",
            "│ 훈련 │ 학습   │",
            "│ 운영 │ 보고   │",
            "└──────┴────────┘",
        ]
        .join("\n");
        let output = enforce_unicode_table_row_dividers(&raw);

        assert_eq!(output.matches("├──────┼────────┤").count(), 3);
        assert!(output.contains("│ 평가 │ 승격   │\n├──────┼────────┤\n│ 훈련 │ 학습   │"));
        assert!(output.contains("│ 훈련 │ 학습   │\n├──────┼────────┤\n│ 운영 │ 보고   │"));
    }

    #[test]
    fn post_output_keeps_existing_row_dividers_unchanged() {
        let rendered = table(
            &["영역", "목적"],
            &[
                vec!["평가".to_string(), "승격".to_string()],
                vec!["훈련".to_string(), "학습".to_string()],
            ],
            Frame::Unicode,
            Theme::None,
            true,
            40,
        );

        assert_eq!(enforce_unicode_table_row_dividers(&rendered), rendered);
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
                "│ 다음 행동 │ 필요하면 abstraction     │",
                "│           │ range와 detail layers를  │",
                "│           │ 조절합니다.              │",
                "└───────────┴──────────────────────────┘",
            ]
            .join("\n")
        );
    }

    #[test]
    fn table_cells_normalize_html_breaks_before_wrapping() {
        let output = table(
            &["주제", "답"],
            &[vec![
                "GPU 쓰는 정당한 학습연산".to_string(),
                "필요하면 강화합니다.<br>예: distillation, offline eval.<br />fake allocation은 제외합니다.<br/>eval로 검증합니다.".to_string(),
            ]],
            Frame::Unicode,
            Theme::None,
            true,
            72,
        );

        assert!(!output.contains("<br"), "{output}");
        assert!(output.contains("필요하면 강화합니다."), "{output}");
        assert!(output.contains("예: distillation,"), "{output}");
        assert!(output.contains("fake allocation은"), "{output}");
        assert!(output.contains("eval로 검증합니다."), "{output}");
        assert_visible_lines_fit(&output, 72);
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
        assert!(output.contains("단일"));
        assert!(output.contains("바이너리"));
        assert!(output.contains("provider"));
        assert!(output.contains("실험"));
    }

    #[test]
    fn cause_effect_report_wraps_rows_and_maps_causal_language() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "원인-결과 리포트로 설명해줘",
            "표 셀이 길어서 박스를 벗어납니다. 그래서 사용자가 구조를 신뢰하기 어렵습니다. 색상이 없어서 attention 신호가 약합니다.",
            &profile,
            70,
        );

        assert_visible_lines_fit(&output, 70);
        assert!(output.contains("│ 원인"), "{output}");
        assert!(output.contains("│ 결과"), "{output}");
        assert!(output.contains("│ 대응"), "{output}");
        assert!(output.contains("표 셀이"), "{output}");
        assert!(output.contains("사용자가"), "{output}");
    }

    #[test]
    fn pros_cons_shape_uses_requested_width_instead_of_fixed_snapshot_width() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "JS와 Rust 장단점을 pros and cons 표로 비교해줘",
            "JS는 실험이 빠르고 Rust는 배포가 단순합니다.",
            &profile,
            58,
        );

        assert_visible_lines_fit(&output, 58);
        assert!(output.contains("JS / Node"), "{output}");
        assert!(output.contains("Rust"), "{output}");
    }

    #[test]
    fn quality_report_enforces_width_row_divider_and_architecture_contracts() {
        let report = quality_report(88);

        assert_eq!(report.overflow_lines, 0, "{report:?}");
        assert!(report.row_dividers >= 3, "{report:?}");
        assert!(report.architecture_boxes >= 6, "{report:?}");
        assert_eq!(report.architecture_panel_overflows, 0, "{report:?}");
        assert!(report.flow_arrows >= 4, "{report:?}");
        assert_eq!(report.flow_box_overflows, 0, "{report:?}");
        assert_eq!(report.flow_connector_breaks, 0, "{report:?}");
        assert_eq!(report.expansion_overflows, 0, "{report:?}");
        assert!(report.numbered_sections >= 2, "{report:?}");
        assert!(report.passed(), "{report:?}");
    }

    #[test]
    fn flow_and_expansion_diagrams_keep_wrapped_labels_inside_width() {
        let diagram = FlowDiagram::new(
            [
                FlowStep::new("Very long architecture gateway label that must wrap inside a box"),
                FlowStep::with_branches(
                    "Decision Router",
                    [
                        "strict artifacts are passed through without rewriting".to_string(),
                        "explanations are shaped with Unicode diagrams".to_string(),
                    ],
                ),
                FlowStep::new("Terminal Renderer"),
            ],
            46,
        );
        let flow = render_flow_diagram(&diagram, Frame::Unicode, Theme::None);
        let expansion = render_expansion_diagram(
            &[
                "Request",
                "Policy",
                "Profile",
                "Selector",
                "Renderer",
                "Quality Gate",
            ],
            Frame::Unicode,
            Theme::None,
            46,
        );

        assert_visible_lines_fit(&flow, 46);
        assert_visible_lines_fit(&expansion, 46);
        assert!(flow.contains('▼'), "{flow}");
        assert!(flow.contains("├─▶"), "{flow}");
        assert!(flow.contains("└─▶"), "{flow}");
        assert!(!flow.contains("----"), "{flow}");
        assert!(!expansion.contains("===="), "{expansion}");
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
    fn wide_divider_table_renders_showcase_style_without_box_overflow() {
        let output = wide_divider_table(
            &["영역", "역할"],
            &[
                vec![
                    "rust/codexplain.rs:307".to_string(),
                    "프로필, 렌더러, 설치/해제, compat-check까지 포함한 Rust core".to_string(),
                ],
                vec![
                    "patches/codex-tui-assistant-color.patch".to_string(),
                    "Codex TUI assistant-message color hook용 패치".to_string(),
                ],
            ],
            Theme::None,
            88,
        );

        assert!(output.contains("영역"), "{output}");
        assert!(output.contains("역할"), "{output}");
        assert!(output.contains("━━━━━━━━"), "{output}");
        assert!(output.contains("────────"), "{output}");
        assert!(!output.contains('│'), "{output}");
        assert_visible_lines_fit(&output, 88);
    }

    #[test]
    fn narrow_table_wraps_long_install_explanations_without_overflow() {
        let output = table(
            &["명령", "동작"],
            &[
                vec![
                    "on".to_string(),
                    "AGENTS.md와 ~/.codex/AGENTS.md에 관리 블록 추가".to_string(),
                ],
                vec![
                    "off".to_string(),
                    "Codexplain이 넣은 관리 블록만 제거".to_string(),
                ],
                vec![
                    "로컬 파일".to_string(),
                    ".codexplain/post-response, README, config 제거".to_string(),
                ],
                vec![
                    "프로필".to_string(),
                    "ux-profile.json은 기본 보존".to_string(),
                ],
            ],
            Frame::Unicode,
            Theme::None,
            true,
            58,
        );

        assert_visible_lines_fit(&output, 58);
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with('├') && line.ends_with('┤'))
                .count(),
            4
        );
        assert!(output.contains("AGENTS.md와 ~/.codex/"));
        assert!(output.contains(".codexplain/post-r"));
        assert!(output.contains("│ 프로필"));
    }

    #[test]
    fn semantic_highlight_keeps_table_widths_and_colors_only_significant_tokens() {
        let rows = [vec![
            "보존".to_string(),
            "JSON/code/diff/log는 strict 출력이고 AGENTS.md와 .codexplain/config.json은 설정입니다."
                .to_string(),
        ]];
        let plain = table(
            &["명령", "동작"],
            &rows,
            Frame::Unicode,
            Theme::None,
            true,
            64,
        );
        let colored = table(
            &["명령", "동작"],
            &rows,
            Frame::Unicode,
            Theme::Ocean,
            true,
            64,
        );

        assert_visible_lines_fit(&colored, 64);
        assert_eq!(visible_line_widths(&colored), visible_line_widths(&plain));
        assert!(colored.contains("\x1b[1;33mJSON/code/diff/log는"));
        assert!(colored.contains("\x1b[1;36mAGENTS.md와"));
        assert!(semantic_highlight(
            Theme::Ocean,
            "on은 off는 AGENTS.md와 JSON/code/diff/log는",
            "accent"
        )
        .contains("\x1b[1;35mon은"));
        assert!(semantic_highlight(
            Theme::Ocean,
            "on은 off는 AGENTS.md와 JSON/code/diff/log는",
            "accent"
        )
        .contains("\x1b[1;36mAGENTS.md와"));
        assert!(semantic_highlight(
            Theme::Ocean,
            "on은 off는 AGENTS.md와 JSON/code/diff/log는",
            "accent"
        )
        .contains("\x1b[1;33mJSON/code/diff/log는"));
        let generic = semantic_highlight(
            Theme::Ocean,
            "CLI Policy Renderer Profile Semantic Highlight Output Mode",
            "accent",
        );
        assert!(!generic.contains("\x1b["), "{generic:?}");
        assert!(semantic_highlight(
            Theme::Ocean,
            "Renderer output has risk when JSON/code/diff is rewritten.",
            "accent"
        )
        .contains("\x1b[1;33mrisk"));
    }

    #[test]
    fn codex_architecture_summary_table_wraps_without_overflow_and_uses_attention_roles() {
        let rows = [
            vec![
                "Codex 자체 색상".to_string(),
                "있음. TUI가 terminal color support를 감지함".to_string(),
            ],
            vec![
                "색상 적용 대상".to_string(),
                "code syntax, diff, status, error 등".to_string(),
            ],
            vec![
                "외부 후처리 hook".to_string(),
                "최종 assistant 응답 렌더링용은 안 보임".to_string(),
            ],
            vec![
                "가능한 우회".to_string(),
                "wrapper/shim으로 stdout 후처리".to_string(),
            ],
            vec![
                "진짜 통합".to_string(),
                "openai/codex TUI renderer 내부 수정 필요".to_string(),
            ],
        ];
        let plain = table(
            &["구분", "결론"],
            &rows,
            Frame::Unicode,
            Theme::None,
            true,
            78,
        );
        let colored = table(
            &["구분", "결론"],
            &rows,
            Frame::Unicode,
            Theme::Ocean,
            true,
            78,
        );

        assert_visible_lines_fit(&plain, 78);
        assert_visible_lines_fit(&colored, 78);
        assert_eq!(visible_line_widths(&plain), visible_line_widths(&colored));
        assert!(plain.contains("최종 assistant 응답"));
        assert!(colored.contains("\x1b[1;31merror"));
        assert!(colored.contains("\x1b[1;33m필요"));
        assert!(!colored.contains("\x1b[1;34mTUI"));
    }

    #[test]
    fn markdown_highlight_panel_does_not_inject_markup_inside_table() {
        let profile = Profile {
            theme: Theme::Ocean,
            ..Profile::default()
        };
        let output = shape_for_output(
            "표로 정리",
            "Codex 자체 색상은 있음, 외부 후처리 hook은 안 보임, 가능한 우회는 wrapper shim입니다.",
            &profile,
            78,
            ColorOutput::Markdown,
        );

        assert!(output.starts_with("**Codexplain highlights**:"), "{output}");
        assert!(output.contains("**[REF]** `CODEX`"), "{output}");
        assert!(
            !output
                .lines()
                .any(|line| line.starts_with('│') && line.contains("**")),
            "{output}"
        );
        assert_visible_lines_fit(
            &output
                .lines()
                .filter(|line| !line.starts_with("**Codexplain highlights**:"))
                .collect::<Vec<_>>()
                .join("\n"),
            78,
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
        assert!(output.contains("\x1b[1;31m위험"));
        assert!(output.contains("color"));
        assert!(output.contains("supplemental"));
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
        assert!(colored.contains("\x1b[1;31m위험"));
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

    #[test]
    fn cleanup_patched_codex_target_deletes_only_project_local_build_cache() {
        let root = env::temp_dir().join(format!(
            "codexplain-patched-codex-clean-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let patched_target = root.join(".codexplain/state/codex-upstream/codex-rs/target/debug");
        fs::create_dir_all(&patched_target).unwrap();
        fs::create_dir_all(root.join(".codexplain/state/codex-upstream/codex-rs/tui")).unwrap();
        fs::write(patched_target.join("codex"), b"binary").unwrap();
        fs::write(
            root.join(".codexplain/state/codex-upstream/codex-rs/tui/source.rs"),
            b"source",
        )
        .unwrap();

        let cleaned = cleanup_patched_codex_target(&root).unwrap();

        assert_eq!(cleaned, TargetCleanup::Removed);
        assert!(!root
            .join(".codexplain/state/codex-upstream/codex-rs/target")
            .exists());
        assert!(root
            .join(".codexplain/state/codex-upstream/codex-rs/tui/source.rs")
            .exists());

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
            RendererKind::CauseEffect,
            RendererKind::ProblemDiagnosis,
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
            .any(|signal| signal.intent == ExplanationIntent::CauseEffectReport));
        assert!(catalog
            .iter()
            .any(|signal| signal.intent == ExplanationIntent::ProblemDiagnosis));
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
                "원인-결과 리포트로 설명해줘",
                RendererKind::CauseEffect,
                ExplanationIntent::CauseEffectReport,
            ),
            (
                "왜 안되고 있는지 문제에 대한 설명과 해결책을 말해줘",
                RendererKind::ProblemDiagnosis,
                ExplanationIntent::ProblemDiagnosis,
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
                "원인-결과 리포트로 설명해줘",
                RendererKind::CauseEffect,
                "원인",
                "결과",
            ),
            (
                "왜 안되고 있는지 문제 설명해줘",
                RendererKind::ProblemDiagnosis,
                "결론",
                "네 질문",
            ),
            (
                "1,2,3 번호 목록으로 설명",
                RendererKind::IndexedList,
                "1. 작업 완료",
                "2. 검증 완료",
            ),
            (
                "처리 흐름을 보여줘",
                RendererKind::Flow,
                "▼",
                "Strict Policy",
            ),
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
    fn architecture_prompts_force_diagram_first_and_table_second() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "이 프로젝트 아키텍처를 설명해줘",
            "Codexplain은 Codex 응답 표현층을 프로젝트 로컬에서 제어합니다.",
            &profile,
            88,
        );
        let selection = select_renderer("이 프로젝트 아키텍처를 설명해줘", &profile);
        let requested = requested_renderers("이 프로젝트 아키텍처를 설명해줘");

        assert_eq!(selection.renderer, RendererKind::Flow);
        assert!(requested.contains(&RendererKind::Flow), "{requested:?}");
        assert!(requested.contains(&RendererKind::Table), "{requested:?}");
        assert!(output.contains("│ Prompt Input"), "{output}");
        assert!(output.contains("│ Capability Boundary"), "{output}");
        assert!(output.contains("━━━━━━━━"), "{output}");
        assert!(output.find("│ Prompt Input").unwrap() < output.find("━━━━━━━━").unwrap());
        assert_visible_lines_fit(&output, 88);
    }

    #[test]
    fn two_paths_prompt_shapes_dense_paragraph_as_numbered_sections() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "실행 흐름은 크게 두 가지로 설명해줘",
            "실행 흐름은 크게 두 가지입니다. codexplain shape/post-response는 입력 텍스트를 바로 후처리합니다. codexplain codex --local-shape는 실제 codex CLI를 실행한 뒤 stdout을 캡처해서 렌더러에 통과시킵니다.",
            &profile,
            80,
        );

        assert!(output.contains("1. codexplain shape/post-response"));
        assert!(output.contains("2. codexplain codex --local-shape"));
        assert!(
            !output.contains("실행 흐름은 크게 두 가지입니다."),
            "intro sentence should not consume a numbered slot: {output}"
        );
    }

    #[test]
    fn problem_diagnosis_uses_answer_first_korean_flow_order() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "왜 안되고 있는지 문제에 대한 설명과 해결책을 말해줘",
            "현재 /codexplain 명령은 Codex TUI가 unknown slash command를 먼저 가로채기 때문에 모델까지 도달하지 않습니다. Evidence: slash status는 CLI fallback으로만 pass입니다. Fix: patched TUI adapter에 slash registry hook을 넣거나 ./bin/codexplain slash status를 직접 실행해야 합니다.",
            &profile,
            88,
        );

        let conclusion = output.find("결론").unwrap();
        let evidence = output.find("왜 그런가").unwrap();
        let fix = output.find("해결 흐름").unwrap();
        let question = output.find("네 질문").unwrap();
        let answer = output.find("답").unwrap();

        assert!(conclusion < evidence && evidence < fix && fix < question && question < answer);
        assert!(output.contains("codexplain"), "{output}");
        assert!(output.contains("slash status"), "{output}");
        assert_visible_lines_fit(&output, 88);
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

        assert!(output.contains("⏱ 진행상황: 진행 중"), "{output}");
        assert!(
            output.contains("[██████████████████████░░░░░░░░░░░░░░]  60%"),
            "{output}"
        );
        assert!(output.contains("│ ⏱ 진척"), "{output}");
        assert!(output.contains("진행 중 · 60%"), "{output}");
        assert!(output.contains("│ 🚀 다음 행동 │"), "{output}");
    }

    #[test]
    fn workflow_progress_component_selects_dev_harness_and_custom_prompts() {
        let profile = Profile {
            theme: Theme::None,
            ux_density: 50,
            ..Profile::default()
        };

        let dev = requested_ux_components(
            "개발 워크플로우 progress block으로 보여줘",
            "구현을 진행 중입니다.",
            &profile,
        );
        let harness = requested_ux_components(
            "하네스 진행상황을 보여줘",
            "harness validation is running",
            &profile,
        );
        let custom = requested_ux_components(
            "사용자 정의 워크플로우 상태 보고",
            "planning stage",
            &profile,
        );

        assert!(dev.contains(&UxComponent::WorkflowProgress), "{dev:?}");
        assert!(
            harness.contains(&UxComponent::WorkflowProgress),
            "{harness:?}"
        );
        assert!(
            custom.contains(&UxComponent::WorkflowProgress),
            "{custom:?}"
        );
    }

    #[test]
    fn workflow_progress_renders_canonical_fields_and_width_safe_bar() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "하네스 workflow progress block으로 보고해줘",
            "현재 3/4 단계입니다. cargo test 통과 후 harness validation이 남았습니다.",
            &profile,
            88,
        );

        assert!(output.contains("⏱ Workflow ["), "{output}");
        assert!(output.contains(" 75%"), "{output}");
        assert!(output.contains("│ 🧭 유형"), "{output}");
        assert!(output.contains("│ ⏱ 단계"), "{output}");
        assert!(output.contains("│ ✅ 완료"), "{output}");
        assert!(output.contains("│ 🔎 현재"), "{output}");
        assert!(output.contains("│ 🚀 다음"), "{output}");
        assert!(output.contains("│ 🔎 근거"), "{output}");
        assert!(output.contains("harness"), "{output}");
        assert!(output.contains("validation"), "{output}");
        assert!(output.contains("validation evidence"), "{output}");
        assert!(
            output.contains("│ 📌 트리거 │ explicit workflow prompt"),
            "{output}"
        );
        assert_visible_lines_fit(&output, 88);
    }

    #[test]
    fn workflow_progress_uses_phase_defaults_when_percent_is_absent() {
        let block = workflow_progress_model(
            "개발 워크플로우 보고",
            "구현을 수정하고 renderer logic을 추가했습니다.",
            "구현 진행",
        );

        assert_eq!(block.workflow_type, "development");
        assert_eq!(block.phase_label, "implementation");
        assert_eq!(block.progress_percent, 50);
        assert!(block.progress_bar.contains("50%"), "{block:?}");
        assert_eq!(block.completed_step, "목표/범위 확인");
        assert!(block.current_step.contains("진행 중"), "{block:?}");
        assert_eq!(block.next_step, "구현 후 검증 증거 확보");
        assert!(block
            .evidence
            .iter()
            .any(|item| item.contains("implementation evidence")));
        assert_eq!(block.trigger_source, "explicit workflow prompt");
        assert!(!block.strict_artifact_bypass);
    }

    #[test]
    fn workflow_progress_reads_shared_fields_before_inference() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let prompt = "\
사용자 정의 워크플로우 progress block
workflow_type: custom
phase_label: review
progress_percent: 42
completed_step: schema wired
current_step: rendering shared progress fields
next_step: run regression suite
next_action: cargo test --bin codexplain
trigger_source: explicit prompt field";
        let response = "\
현재 3/4 단계입니다.
evidence: shared fields rendered|width-safe table output";
        let output = shape(prompt, response, &profile, 96);

        assert!(output.contains("custom"), "{output}");
        assert!(output.contains("review"), "{output}");
        assert!(output.contains("매크로 42%"), "{output}");
        assert!(output.contains("schema wired"), "{output}");
        assert!(
            output.contains("rendering shared progress fields"),
            "{output}"
        );
        assert!(output.contains("run regression suite"), "{output}");
        assert!(output.contains("cargo test --bin codexplain"), "{output}");
        assert!(output.contains("shared fields rendered"), "{output}");
        assert!(output.contains("width-safe table output"), "{output}");
        assert!(output.contains("explicit prompt field"), "{output}");
        assert_visible_lines_fit(&output, 96);
    }

    #[test]
    fn shared_progress_field_parser_accepts_planner_hint_shape() {
        let fields = parse_shared_workflow_progress_fields(
            "workflow_type=harness; phase_label=validation; progress_percent=80; next_action=collect logs; evidence=tests pass|harness trace",
        );

        assert_eq!(fields.workflow_type.as_deref(), Some("harness"));
        assert_eq!(fields.phase_label.as_deref(), Some("validation"));
        assert_eq!(fields.progress_percent, Some(80));
        assert_eq!(fields.next_action.as_deref(), Some("collect logs"));
        assert_eq!(
            fields.evidence,
            vec!["tests pass".to_string(), "harness trace".to_string()]
        );
    }

    #[test]
    fn workflow_progress_preserves_strict_artifacts_by_backing_off() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let json = r#"{"phase":"validation","percent":75}"#;
        let output = shape("valid JSON만 출력", json, &profile, 88);

        assert_eq!(output, json);
    }

    #[test]
    fn macro_progress_collapses_codex_activity_transcript() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let transcript = "\
• Explored
  └ Read codexplain, codexplain-codex
• Ran ls bin dist rust scripts docs 2>/dev/null || true
• Explored
  └ Search *.rs in rust
• Ran find .codexplain -maxdepth 2 -type f -print 2>/dev/null || true
  └ .codexplain/config.json
";
        let output = shape(
            "이 작업 로그를 전체적인 프로세스로 설명해줘",
            transcript,
            &profile,
            88,
        );

        assert!(output.contains("│ 🧭 탐색"), "{output}");
        assert!(output.contains("│ 🔎 검색"), "{output}");
        assert!(output.contains("│ ⚙️ 실행"), "{output}");
        assert!(output.contains("│ 🎛️ 설정"), "{output}");
        assert!(output.contains("macro-progress"), "{output}");
        assert!(output.contains("UX 적용"), "{output}");
        assert!(!output.contains("codexplain-codex"), "{output}");
        assert_visible_lines_fit(&output, 88);
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

        assert!(light_output.contains("Input Gateway"), "{light_output}");
        assert!(light_output.contains("Concrete View"), "{light_output}");
        assert!(
            !light_output.contains("Lifecycle Manager"),
            "{light_output}"
        );
        assert!(!light_output.contains("다음 행동"), "{light_output}");
        assert!(deep_output.contains("Boundary"), "{deep_output}");
        assert!(deep_output.contains("Adaptation"), "{deep_output}");
        assert!(deep_output.contains("Terminal Renderer"), "{deep_output}");
        assert!(deep_output.contains("Level Controls"), "{deep_output}");
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
        assert!(output.contains("│ ⚠️ 위험"), "{output}");
        assert!(output.contains("확신도"), "{output}");
        assert!(output.contains("│ 변경"), "{output}");
        assert!(output.contains("│ 선택"), "{output}");
        assert!(output.contains("🚀 다음 행동:"), "{output}");
        assert!(output.contains("⏱ ETA:"), "{output}");
        assert!(output.contains("│ ⚠️ 중요"), "{output}");
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
        assert!(output.contains("│ ⚠️ 위험"), "{output}");
        assert!(output.contains("🚀 다음 행동:"), "{output}");
        assert!(output.contains("│ ⚠️ 중요"), "{output}");
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

        assert!(output.contains("영역"), "{output}");
        assert!(output.contains("━━━━━━━━"), "{output}");
        assert!(output.contains("│ Prompt Input"), "{output}");
        assert!(output.contains("JS / Node"), "{output}");
        assert!(output.contains("Rust"), "{output}");
        assert!(output.contains("핵심식 : 설명 품질 = f"), "{output}");
        assert_visible_lines_fit(&output, 132);
    }

    #[test]
    fn pros_cons_table_prompt_does_not_add_brittle_generic_summary_table() {
        let profile = Profile {
            theme: Theme::None,
            ..Profile::default()
        };
        let output = shape(
            "JS와 Rust 선택 이유를 pros and cons 표와 수식으로 보여줘",
            "JS는 빠른 실험과 provider 연동, JSON 문자열 처리, CLI UX 반복에 유리하다. Rust는 단일 바이너리, 낮은 메모리, 빠른 시작 속도, 타입 안정성, 터미널 렌더링에 유리하다.",
            &profile,
            88,
        );

        assert!(output.contains("JS / Node"), "{output}");
        assert!(output.contains("수식 박스"), "{output}");
        assert!(!output.contains("│ 낮 "), "{output}");
        assert!(!output.contains("│ 구분"), "{output}");
        assert_visible_lines_fit(&output, 88);
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

        assert!(output.contains("영역"), "{output}");
        assert!(output.contains("━━━━━━━━"), "{output}");
        assert!(output.contains("│ Prompt Input"), "{output}");
        let table_pos = output.find("영역").unwrap();
        let flow_pos = output.find("│ Prompt Input").unwrap();
        assert!(table_pos > flow_pos, "{output}");
        assert_visible_lines_fit(&output, 60);
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
            (ExplanationIntent::OrderedSteps, "1. 첫째입니다"),
            (ExplanationIntent::CauseEffectReport, "원인"),
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
