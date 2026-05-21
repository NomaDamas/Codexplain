use std::env;

#[derive(Clone, Copy)]
enum Frame {
    Unicode,
    Ascii,
}

impl Frame {
    fn from_arg(value: Option<&str>) -> Self {
        match value.unwrap_or("unicode") {
            "ascii" => Self::Ascii,
            _ => Self::Unicode,
        }
    }

    fn chars(self) -> FrameChars {
        match self {
            Self::Ascii => FrameChars {
                h: '-',
                v: '|',
                tl: '+',
                tj: '+',
                tr: '+',
                ml: '+',
                mj: '+',
                mr: '+',
                bl: '+',
                bj: '+',
                br: '+',
            },
            Self::Unicode => FrameChars {
                h: '─',
                v: '│',
                tl: '┌',
                tj: '┬',
                tr: '┐',
                ml: '├',
                mj: '┼',
                mr: '┤',
                bl: '└',
                bj: '┴',
                br: '┘',
            },
        }
    }
}

struct FrameChars {
    h: char,
    v: char,
    tl: char,
    tj: char,
    tr: char,
    ml: char,
    mj: char,
    mr: char,
    bl: char,
    bj: char,
    br: char,
}

fn visible_width(text: &str) -> usize {
    text.chars()
        .map(|ch| {
            if ('\u{ac00}'..='\u{d7a3}').contains(&ch) {
                2
            } else {
                1
            }
        })
        .sum()
}

fn pad(text: &str, width: usize) -> String {
    let mut value = text.to_string();
    value.push_str(&" ".repeat(width.saturating_sub(visible_width(text))));
    value
}

fn border(left: char, join: char, right: char, widths: &[usize], h: char) -> String {
    let mut out = String::new();
    out.push(left);
    for (index, width) in widths.iter().enumerate() {
        out.push_str(&h.to_string().repeat(width + 2));
        out.push(if index + 1 == widths.len() {
            right
        } else {
            join
        });
    }
    out
}

fn table(headers: &[&str], rows: &[Vec<&str>], frame: Frame) -> String {
    let chars = frame.chars();
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            rows.iter()
                .map(|row| visible_width(row.get(index).copied().unwrap_or("")))
                .chain(std::iter::once(visible_width(header)))
                .max()
                .unwrap_or(4)
                .max(4)
        })
        .collect();

    let mut lines = Vec::new();
    lines.push(border(chars.tl, chars.tj, chars.tr, &widths, chars.h));
    lines.push(row(headers, &widths, chars.v));
    lines.push(border(chars.ml, chars.mj, chars.mr, &widths, chars.h));
    for item in rows {
        lines.push(row(item, &widths, chars.v));
    }
    lines.push(border(chars.bl, chars.bj, chars.br, &widths, chars.h));
    lines.join("\n")
}

fn row(cells: &[&str], widths: &[usize], vertical: char) -> String {
    let mut out = String::new();
    out.push(vertical);
    for (index, width) in widths.iter().enumerate() {
        out.push(' ');
        out.push_str(&pad(cells.get(index).copied().unwrap_or(""), *width));
        out.push(' ');
        out.push(vertical);
    }
    out
}

fn pros_cons(frame: Frame) -> String {
    table(
        &["선택지", "장점", "단점", "적합한 때"],
        &[
            vec![
                "JS / Node",
                "빠른 수정, provider 연동, JSON 처리",
                "런타임 의존성, 단일 바이너리 약함",
                "UX 실험과 피드백 루프",
            ],
            vec![
                "Rust",
                "단일 바이너리, 빠른 시작, 낮은 메모리",
                "초기 구현 비용, provider 실험 비용",
                "안정화된 CLI core",
            ],
        ],
        frame,
    )
}

fn formula(frame: Frame) -> String {
    table(
        &["구분", "수식/의미"],
        &[
            vec!["판단식", "선택 = f(반복속도, 배포형태, 안정성, 유지보수)"],
            vec!["초기", "JS 점수 ↑ = 빠른 UX 변경"],
            vec!["제품화", "Rust 점수 ↑ = 배포/성능/안정성"],
        ],
        frame,
    )
}

fn usage() -> &'static str {
    "Usage:
  codexplain-rs pros-cons [--frame unicode|ascii]
  codexplain-rs formula [--frame unicode|ascii]
  codexplain-rs demo"
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|item| item == name)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("demo");
    let frame = Frame::from_arg(arg_value(&args, "--frame"));

    match command {
        "pros-cons" => println!("{}", pros_cons(frame)),
        "formula" => println!("{}", formula(frame)),
        "demo" => println!("{}\n\n{}", pros_cons(frame), formula(frame)),
        "--help" | "-h" => println!("{}", usage()),
        other => {
            eprintln!("Unknown command: {other}\n\n{}", usage());
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_unicode_pros_cons() {
        let output = pros_cons(Frame::Unicode);
        assert!(output.contains('┌'));
        assert!(output.contains("JS / Node"));
        assert!(output.contains("Rust"));
    }

    #[test]
    fn renders_ascii_formula() {
        let output = formula(Frame::Ascii);
        assert!(output.contains('+'));
        assert!(output.contains("선택 = f"));
    }
}
