//! Colored rendering of diagnostics.
//!
//! The layout matches `Diagnostic::render` from the compiler, but adds
//! colors. Colors are written as ANSI codes; `anstream` removes them when
//! the output is not a terminal or colors are switched off.

use std::fmt::Write as _;

use anstyle::{AnsiColor, Style};
use ekips_compiler::Diagnostic;
use ekips_compiler::diagnostic::Severity;
use ekips_compiler::span::LineIndex;
use ekips_i18n::{Lang, t};

const BOLD: Style = Style::new().bold();
const ERROR: Style = AnsiColor::Red.on_default().bold();
const WARNING: Style = AnsiColor::Yellow.on_default().bold();
const GUTTER: Style = AnsiColor::Blue.on_default().bold();
const HELP: Style = AnsiColor::Cyan.on_default().bold();

/// Formats one diagnostic with the affected source line.
///
/// ```text
/// error: I don't understand the word `cosnt` here.
///  --> mission.ekips:5:3
///   |
/// 5 |   cosnt speed = 50%;
///   |   ^^^^^
///   = help: Did you mean `const`?
/// ```
pub fn diagnostic(diag: &Diagnostic, source: &str, path: &str, lang: Lang) -> String {
    let index = LineIndex::new(source);
    let pos = index.line_col(diag.span.start);
    let line_text = index.line_text(pos.line);
    let gutter_width = pos.line.to_string().len();
    let pad = " ".repeat(gutter_width);

    // Caret length: characters of the span on this line, at least one.
    let end = index.line_col(diag.span.end);
    let end_col = if end.line == pos.line {
        end.col
    } else {
        line_text.chars().count() + 1
    };
    let carets = end_col.saturating_sub(pos.col).max(1);

    let (label, style) = match diag.severity() {
        Severity::Error => (t(lang, "error", "Fehler"), ERROR),
        Severity::Warning => (t(lang, "warning", "Warnung"), WARNING),
    };
    // Tabs would shift the carets, so show them as single spaces.
    let line_text = line_text.replace('\t', " ");

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{style}{label}{style:#}{BOLD}: {}{BOLD:#}",
        diag.message(lang)
    );
    let _ = writeln!(
        out,
        "{pad}{GUTTER}-->{GUTTER:#} {path}:{}:{}",
        pos.line, pos.col
    );
    let _ = writeln!(out, "{pad} {GUTTER}|{GUTTER:#}");
    let _ = writeln!(out, "{GUTTER}{} |{GUTTER:#} {line_text}", pos.line);
    let _ = writeln!(
        out,
        "{pad} {GUTTER}|{GUTTER:#} {}{style}{}{style:#}",
        " ".repeat(pos.col - 1),
        "^".repeat(carets)
    );
    if let Some(hint) = diag.hint(lang) {
        let _ = writeln!(
            out,
            "{pad} {GUTTER}={GUTTER:#} {HELP}{}{HELP:#}: {hint}",
            t(lang, "help", "Hilfe")
        );
    }
    out
}

/// Text for "N errors" / "N Fehler" with the right singular or plural.
pub fn count(lang: Lang, n: usize, en: (&str, &str), de: (&str, &str)) -> String {
    let (one, many) = match lang {
        Lang::En => en,
        Lang::De => de,
    };
    format!("{n} {}", if n == 1 { one } else { many })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(s: &str) -> String {
        anstream::adapter::strip_str(s).to_string()
    }

    #[test]
    fn layout_matches_compiler_rendering() {
        let source = "cosnt arm = new Motor(Port.C);\n";
        let result = ekips_compiler::parse(source);
        let diag = &result.diagnostics[0];
        for lang in [Lang::En, Lang::De] {
            let ours = strip(&diagnostic(diag, source, "m.ekips", lang));
            let theirs = diag.render(source, "m.ekips", lang);
            assert_eq!(ours, theirs);
        }
    }

    #[test]
    fn plural() {
        let en = ("error", "errors");
        let de = ("Fehler", "Fehler");
        assert_eq!(count(Lang::En, 1, en, de), "1 error");
        assert_eq!(count(Lang::En, 2, en, de), "2 errors");
    }
}
