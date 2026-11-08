//! `ekips-lsp`: the Ekips Prime Pro language server (LSP over stdio).

use std::process::ExitCode;

use ekips_i18n::{Lang, t};

fn main() -> ExitCode {
    let mut config = ekips_lsp::Config::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let lang = config.lang.unwrap_or_else(ekips_i18n::detect_lang);
        match arg.as_str() {
            // Editors often pass `--stdio`; stdio is the only transport anyway.
            "--stdio" => {}
            "--lang" => match args.next().as_deref().and_then(ekips_i18n::parse_lang) {
                Some(l) => config.lang = Some(l),
                None => return usage_error(lang, "--lang"),
            },
            a if a.starts_with("--lang=") => match ekips_i18n::parse_lang(&a[7..]) {
                Some(l) => config.lang = Some(l),
                None => return usage_error(lang, "--lang"),
            },
            "-V" | "--version" => {
                println!("ekips-lsp {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "-h" | "--help" => {
                print_help(lang);
                return ExitCode::SUCCESS;
            }
            other => return usage_error(lang, other),
        }
    }
    if config.lang.is_none() {
        config.lang = std::env::var(ekips_i18n::LANG_ENV)
            .ok()
            .as_deref()
            .and_then(ekips_i18n::parse_lang);
    }

    let (connection, io_threads) = lsp_server::Connection::stdio();
    let result = ekips_lsp::run(&connection, config);
    drop(connection);
    let joined = io_threads.join();
    match result
        .map_err(|e| e.to_string())
        .and(joined.map_err(|e| e.to_string()))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ekips-lsp: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_help(lang: Lang) {
    println!(
        "{}",
        t(
            lang,
            "ekips-lsp – language server for Ekips Prime Pro

Started by your editor (e.g. the VS Code extension). It talks to the
editor over stdin/stdout using the Language Server Protocol.

Usage: ekips-lsp [OPTIONS]

Options:
      --lang <en|de>  Language of messages [default: editor or system language]
      --stdio         Use stdin/stdout (the default, accepted for compatibility)
  -V, --version       Print version
  -h, --help          Print help",
            "ekips-lsp – Language Server für Ekips Prime Pro

Wird von deinem Editor gestartet (z. B. der VS-Code-Erweiterung). Er spricht
über stdin/stdout mit dem Editor, mit dem Language Server Protocol.

Aufruf: ekips-lsp [OPTIONEN]

Optionen:
      --lang <en|de>  Sprache der Meldungen [Standard: Sprache des Editors oder Systems]
      --stdio         stdin/stdout benutzen (Standard, für Kompatibilität akzeptiert)
  -V, --version       Version anzeigen
  -h, --help          Hilfe anzeigen",
        )
    );
}

fn usage_error(lang: Lang, arg: &str) -> ExitCode {
    match lang {
        Lang::En => {
            eprintln!("error: unknown or invalid argument `{arg}`. Try `ekips-lsp --help`.")
        }
        Lang::De => eprintln!(
            "Fehler: Unbekanntes oder ungültiges Argument `{arg}`. Siehe `ekips-lsp --help`."
        ),
    }
    ExitCode::from(2)
}
