//! Command-line definition.
//!
//! The command is built at runtime (not with `derive`) so that all help
//! texts can be shown in the language of the student.

use clap::{Arg, ArgAction, Command, value_parser};
use ekips_i18n::{Lang, t};

/// Builds the `ekips` command with help texts in `lang`.
pub fn command(lang: Lang) -> Command {
    let root = Command::new("ekips")
        .version(env!("CARGO_PKG_VERSION"))
        .about(t(
            lang,
            "Tools for Ekips Prime Pro, the robot language for the LEGO SPIKE Prime hub",
            "Werkzeuge für Ekips Prime Pro, die Robotersprache für den LEGO SPIKE Prime Hub",
        ))
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand_help_heading(t(lang, "Commands", "Befehle"))
        .subcommand_value_name(t(lang, "COMMAND", "BEFEHL"))
        .override_usage(usage(lang, "ekips", "<COMMAND>", "<BEFEHL>"))
        .arg(
            Arg::new("lang")
                .long("lang")
                .value_name(t(lang, "LANGUAGE", "SPRACHE"))
                .value_parser(["en", "de"])
                .hide_possible_values(true)
                .global(true)
                .help_heading(options_heading(lang))
                .help(t(
                    lang,
                    "Language of all messages: en or de [default: system language]",
                    "Sprache aller Meldungen: en oder de [Standard: Systemsprache]",
                )),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .value_name(t(lang, "WHEN", "WANN"))
                .value_parser(["auto", "always", "never"])
                .default_value("auto")
                .hide_possible_values(true)
                .hide_default_value(true)
                .global(true)
                .help_heading(options_heading(lang))
                .help(t(
                    lang,
                    "When to use colors: auto, always or never [default: auto]",
                    "Wann Farben benutzt werden: auto, always oder never [Standard: auto]",
                )),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::Version)
                .help_heading(options_heading(lang))
                .help(t(lang, "Print version", "Version anzeigen")),
        )
        .subcommand(check(lang))
        .subcommand(run(lang))
        .subcommand(
            Command::new("upload")
                .override_usage(usage(lang, "ekips upload", "[FILE]", "[DATEI]"))
                .about(t(
                    lang,
                    "Upload a program to the hub (not available yet)",
                    "Ein Programm auf den Hub übertragen (noch nicht verfügbar)",
                ))
                .arg(file_arg(lang, false)),
        )
        .subcommand(
            Command::new("monitor")
                .override_usage(usage(lang, "ekips monitor", "", ""))
                .about(t(
                    lang,
                    "Show the output of the hub (not available yet)",
                    "Die Ausgaben des Hubs anzeigen (noch nicht verfügbar)",
                )),
        )
        .subcommand(
            Command::new("flash-firmware")
                .override_usage(usage(lang, "ekips flash-firmware", "", ""))
                .about(t(
                    lang,
                    "Install the Ekips Prime Pro firmware on the hub (not available yet)",
                    "Die Ekips-Prime-Pro-Firmware auf den Hub spielen (noch nicht verfügbar)",
                )),
        )
        .subcommand(
            Command::new("backup-firmware")
                .override_usage(usage(lang, "ekips backup-firmware", "[FILE]", "[DATEI]"))
                .about(t(
                    lang,
                    "Save the current firmware of the hub to a file (not available yet)",
                    "Die aktuelle Firmware des Hubs in eine Datei sichern (noch nicht verfügbar)",
                ))
                .arg(
                    Arg::new("output")
                        .value_name(t(lang, "FILE", "DATEI"))
                        .value_parser(value_parser!(std::path::PathBuf))
                        .help_heading(args_heading(lang))
                        .help(t(
                            lang,
                            "Where to save the backup",
                            "Wohin die Sicherung gespeichert wird",
                        )),
                ),
        );
    localize(root, lang)
}

fn check(lang: Lang) -> Command {
    Command::new("check")
        .override_usage(usage(lang, "ekips check", "<FILES>...", "<DATEIEN>..."))
        .about(t(
            lang,
            "Check programs for mistakes",
            "Programme auf Fehler prüfen",
        ))
        .arg(
            Arg::new("files")
                .value_name(t(lang, "FILES", "DATEIEN"))
                .num_args(1..)
                .required(true)
                .value_parser(value_parser!(std::path::PathBuf))
                .help_heading(args_heading(lang))
                .help(t(
                    lang,
                    ".ekips files or folders to check",
                    ".ekips-Dateien oder Ordner, die geprüft werden",
                )),
        )
        .arg(
            Arg::new("ast")
                .long("ast")
                .action(ArgAction::SetTrue)
                .help_heading(options_heading(lang))
                .help(t(
                    lang,
                    "Also print the syntax tree (for experts)",
                    "Zusätzlich den Syntaxbaum ausgeben (für Profis)",
                )),
        )
}

fn run(lang: Lang) -> Command {
    Command::new("run")
        .override_usage(usage(lang, "ekips run", "<FILE>", "<DATEI>"))
        .about(t(
            lang,
            "Run a program (not available yet)",
            "Ein Programm ausführen (noch nicht verfügbar)",
        ))
        .arg(file_arg(lang, true))
        .arg(
            Arg::new("sim")
                .long("sim")
                .action(ArgAction::SetTrue)
                .help_heading(options_heading(lang))
                .help(t(
                    lang,
                    "Run in the simulator instead of on the hub",
                    "Im Simulator statt auf dem Hub ausführen",
                )),
        )
}

fn file_arg(lang: Lang, required: bool) -> Arg {
    Arg::new("file")
        .value_name(t(lang, "FILE", "DATEI"))
        .required(required)
        .value_parser(value_parser!(std::path::PathBuf))
        .help_heading(args_heading(lang))
        .help(t(lang, "The .ekips program", "Das .ekips-Programm"))
}

/// Usage line with a translated `[OPTIONS]` placeholder.
fn usage(lang: Lang, cmd: &str, en: &str, de: &str) -> String {
    let rest = t(lang, en, de);
    let options = t(lang, "[OPTIONS]", "[OPTIONEN]");
    if rest.is_empty() {
        format!("{cmd} {options}")
    } else {
        format!("{cmd} {options} {rest}")
    }
}

fn options_heading(lang: Lang) -> &'static str {
    t(lang, "Options", "Optionen")
}

fn args_heading(lang: Lang) -> &'static str {
    t(lang, "Arguments", "Argumente")
}

/// Replaces clap's English-only help flag and headings, recursively.
fn localize(cmd: Command, lang: Lang) -> Command {
    let template = format!(
        "{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
        anstyle::Style::new()
            .bold()
            .underline()
            .render()
            .to_string()
            + t(lang, "Usage:", "Aufruf:")
            + &anstyle::Reset.render().to_string()
    );
    let cmd = cmd.help_template(template).disable_help_flag(true).arg(
        Arg::new("help")
            .short('h')
            .long("help")
            .action(ArgAction::Help)
            .help_heading(options_heading(lang))
            .help(t(lang, "Print help", "Hilfe anzeigen")),
    );
    let names: Vec<String> = cmd
        .get_subcommands()
        .map(|s| s.get_name().to_string())
        .collect();
    names.into_iter().fold(cmd, |cmd, name| {
        cmd.mut_subcommand(name, |sub| localize(sub, lang))
    })
}

/// Finds `--lang` in the raw arguments before the full parse, so that even
/// help texts and parse errors use the right language.
pub fn lang_from_args<I, S>(args: I) -> Option<Lang>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg = arg.as_ref().to_string_lossy().into_owned();
        if arg == "--" {
            break;
        }
        if arg == "--lang" {
            return args
                .next()
                .and_then(|v| ekips_i18n::parse_lang(&v.as_ref().to_string_lossy()));
        }
        if let Some(value) = arg.strip_prefix("--lang=") {
            return ekips_i18n::parse_lang(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_is_valid() {
        command(Lang::En).debug_assert();
        command(Lang::De).debug_assert();
    }

    #[test]
    fn finds_lang_in_args() {
        assert_eq!(lang_from_args(["ekips", "--lang", "de"]), Some(Lang::De));
        assert_eq!(
            lang_from_args(["ekips", "check", "--lang=en"]),
            Some(Lang::En)
        );
        assert_eq!(lang_from_args(["ekips", "check", "x.ekips"]), None);
        assert_eq!(lang_from_args(["ekips", "--", "--lang", "de"]), None);
    }
}
