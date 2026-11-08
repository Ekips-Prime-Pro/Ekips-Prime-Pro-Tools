//! `ekips`: command-line tool for Ekips Prime Pro.
//!
//! See `docs/en/cli.md` (or `docs/de/cli.md`) for the user documentation.

mod check;
mod cli;
mod placeholder;
mod render;

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use anstream::{AutoStream, ColorChoice};
use anstyle::AnsiColor;
use ekips_i18n::{Lang, t};

use placeholder::Planned;

/// Exit status, ordered by severity (the worst one wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExitStatus {
    /// Everything is fine.
    Ok = 0,
    /// The program has mistakes.
    Problems = 1,
    /// Wrong usage or a file could not be read.
    Usage = 2,
    /// The command is not available yet.
    NotAvailable = 3,
}

/// Colored standard output and error streams.
pub struct Output {
    /// Normal output
    pub stdout: AutoStream<std::io::Stdout>,
    /// Problems and notes
    pub stderr: AutoStream<std::io::Stderr>,
    lang: Lang,
}

impl Output {
    fn new(choice: ColorChoice, lang: Lang) -> Self {
        Output {
            stdout: AutoStream::new(std::io::stdout(), choice),
            stderr: AutoStream::new(std::io::stderr(), choice),
            lang,
        }
    }

    fn labelled(&mut self, color: AnsiColor, en: &str, de: &str, message: &str) {
        let style = color.on_default().bold();
        let label = t(self.lang, en, de);
        let _ = writeln!(self.stderr, "{style}{label}{style:#}: {message}");
    }

    /// Prints an error that is not tied to a place in a program.
    pub fn error(&mut self, message: &str) {
        self.labelled(AnsiColor::Red, "error", "Fehler", message);
    }

    /// Prints a warning that is not tied to a place in a program.
    pub fn warn(&mut self, message: &str) {
        self.labelled(AnsiColor::Yellow, "warning", "Warnung", message);
    }

    /// Prints an additional note.
    pub fn note(&mut self, message: &str) {
        self.labelled(AnsiColor::Cyan, "note", "Hinweis", message);
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    let lang = cli::lang_from_args(&args).unwrap_or_else(ekips_i18n::detect_lang);

    let matches = match cli::command(lang).try_get_matches_from(&args) {
        Ok(m) => m,
        Err(e) => {
            let _ = e.print();
            if e.use_stderr() && lang == Lang::De {
                // clap's own error messages are English only.
                eprintln!("\nHinweis: Mit `ekips --help` siehst du alle Befehle und Optionen.");
            }
            return ExitCode::from(if e.use_stderr() {
                ExitStatus::Usage as u8
            } else {
                0
            });
        }
    };

    let choice = match matches.get_one::<String>("color").map(String::as_str) {
        Some("always") => ColorChoice::Always,
        Some("never") => ColorChoice::Never,
        _ => ColorChoice::Auto,
    };
    let mut out = Output::new(choice, lang);

    let status = match matches.subcommand() {
        Some(("check", sub)) => {
            let opts = check::CheckOptions {
                paths: sub
                    .get_many::<PathBuf>("files")
                    .map(|v| v.cloned().collect())
                    .unwrap_or_default(),
                show_ast: sub.get_flag("ast"),
            };
            check::run(&opts, lang, &mut out)
        }
        Some(("run", sub)) => {
            let planned = if sub.get_flag("sim") {
                Planned::RunSim
            } else {
                Planned::RunHub
            };
            placeholder::run(planned, lang, &mut out)
        }
        Some(("upload", _)) => placeholder::run(Planned::Upload, lang, &mut out),
        Some(("monitor", _)) => placeholder::run(Planned::Monitor, lang, &mut out),
        Some(("flash-firmware", _)) => placeholder::run(Planned::FlashFirmware, lang, &mut out),
        Some(("backup-firmware", _)) => placeholder::run(Planned::BackupFirmware, lang, &mut out),
        // clap guarantees a known subcommand (`subcommand_required`).
        _ => ExitStatus::Usage,
    };
    let _ = out.stdout.flush();
    let _ = out.stderr.flush();
    ExitCode::from(status as u8)
}
