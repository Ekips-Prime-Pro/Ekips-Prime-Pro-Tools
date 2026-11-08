//! `ekips check`: finds mistakes in programs.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use ekips_compiler::diagnostic::Severity;
use ekips_i18n::{Lang, t};

use crate::render;
use crate::{ExitStatus, Output};

/// File extension of Ekips Prime Pro programs.
pub const EXTENSION: &str = "ekips";

/// Options for [`run`].
pub struct CheckOptions {
    /// Files and folders given on the command line
    pub paths: Vec<PathBuf>,
    /// Print the syntax tree
    pub show_ast: bool,
}

/// Checks all given files and returns the exit status.
pub fn run(opts: &CheckOptions, lang: Lang, out: &mut Output) -> ExitStatus {
    let mut status = ExitStatus::Ok;
    let mut files = Vec::new();
    for path in &opts.paths {
        if path.is_dir() {
            let before = files.len();
            collect_dir(path, &mut files);
            if files.len() == before {
                out.warn(&match lang {
                    Lang::En => format!(
                        "The folder `{}` contains no .{EXTENSION} files.",
                        path.display()
                    ),
                    Lang::De => format!(
                        "Im Ordner `{}` gibt es keine .{EXTENSION}-Dateien.",
                        path.display()
                    ),
                });
            }
        } else {
            files.push(path.clone());
        }
    }

    let (mut total_errors, mut total_warnings) = (0, 0);
    for path in &files {
        match check_file(path, opts.show_ast, lang, out) {
            Ok((errors, warnings)) => {
                total_errors += errors;
                total_warnings += warnings;
                if errors > 0 {
                    status = status.max(ExitStatus::Problems);
                }
            }
            Err(message) => {
                out.error(&message);
                status = status.max(ExitStatus::Usage);
            }
        }
    }

    if files.len() > 1 {
        let errors = render::count(
            lang,
            total_errors,
            ("error", "errors"),
            ("Fehler", "Fehler"),
        );
        let warnings = render::count(
            lang,
            total_warnings,
            ("warning", "warnings"),
            ("Warnung", "Warnungen"),
        );
        let checked = render::count(lang, files.len(), ("file", "files"), ("Datei", "Dateien"));
        let line = match lang {
            Lang::En => format!("Checked {checked}: {errors}, {warnings}."),
            Lang::De => format!("{checked} geprüft: {errors}, {warnings}."),
        };
        let _ = writeln!(out.stdout, "{line}");
    }
    status
}

/// Checks one file. Returns the number of errors and warnings, or a
/// message if the file could not be read.
fn check_file(
    path: &Path,
    show_ast: bool,
    lang: Lang,
    out: &mut Output,
) -> Result<(usize, usize), String> {
    let shown = path.display().to_string();
    let source = std::fs::read_to_string(path).map_err(|e| read_error(&shown, &e, lang))?;

    if path.extension().and_then(|e| e.to_str()) != Some(EXTENSION) {
        out.warn(&match lang {
            Lang::En => format!("`{shown}` does not end in .{EXTENSION}. Checking it anyway."),
            Lang::De => {
                format!("`{shown}` endet nicht auf .{EXTENSION}. Ich prüfe die Datei trotzdem.")
            }
        });
    }

    let result = ekips_compiler::parse(&source);
    if show_ast {
        let _ = writeln!(out.stdout, "{:#?}", result.program);
    }
    for diag in &result.diagnostics {
        let _ = writeln!(
            out.stderr,
            "{}",
            render::diagnostic(diag, &source, &shown, lang)
        );
    }

    let errors = result
        .diagnostics
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .count();
    let warnings = result.diagnostics.len() - errors;

    let ok = anstyle::AnsiColor::Green.on_default().bold();
    let bad = anstyle::AnsiColor::Red.on_default().bold();
    if errors == 0 && warnings == 0 {
        let _ = writeln!(
            out.stdout,
            "{shown}: {ok}{}{ok:#}",
            t(lang, "OK, no mistakes found.", "OK, keine Fehler gefunden.")
        );
    } else {
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(render::count(
                lang,
                errors,
                ("error", "errors"),
                ("Fehler", "Fehler"),
            ));
        }
        if warnings > 0 {
            parts.push(render::count(
                lang,
                warnings,
                ("warning", "warnings"),
                ("Warnung", "Warnungen"),
            ));
        }
        let style = if errors > 0 { bad } else { ok };
        let _ = writeln!(
            out.stderr,
            "{shown}: {style}{}{style:#}",
            parts.join(t(lang, " and ", " und "))
        );
    }
    Ok((errors, warnings))
}

fn read_error(path: &str, err: &std::io::Error, lang: Lang) -> String {
    match (err.kind(), lang) {
        (std::io::ErrorKind::NotFound, Lang::En) => {
            format!("The file `{path}` does not exist.")
        }
        (std::io::ErrorKind::NotFound, Lang::De) => {
            format!("Die Datei `{path}` gibt es nicht.")
        }
        (std::io::ErrorKind::InvalidData, Lang::En) => {
            format!("The file `{path}` is not a text file (or not saved as UTF-8).")
        }
        (std::io::ErrorKind::InvalidData, Lang::De) => {
            format!("Die Datei `{path}` ist keine Textdatei (oder nicht als UTF-8 gespeichert).")
        }
        (_, Lang::En) => format!("Could not read `{path}`: {err}"),
        (_, Lang::De) => format!("`{path}` konnte nicht gelesen werden: {err}"),
    }
}

/// Adds all `.ekips` files below `dir` (sorted, skipping hidden folders).
fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        let hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            collect_dir(&path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some(EXTENSION) {
            files.push(path);
        }
    }
}
