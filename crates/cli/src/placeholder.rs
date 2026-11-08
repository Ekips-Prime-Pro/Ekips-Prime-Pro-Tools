//! Commands that are planned but not available yet.

use ekips_i18n::{Lang, t};

use crate::{ExitStatus, Output};

/// A planned command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Planned {
    /// `ekips run --sim`
    RunSim,
    /// `ekips run` (on the hub)
    RunHub,
    /// `ekips upload`
    Upload,
    /// `ekips monitor`
    Monitor,
    /// `ekips flash-firmware`
    FlashFirmware,
    /// `ekips backup-firmware`
    BackupFirmware,
}

impl Planned {
    /// How the student called the command.
    pub fn command(self) -> &'static str {
        match self {
            Planned::RunSim => "ekips run --sim",
            Planned::RunHub => "ekips run",
            Planned::Upload => "ekips upload",
            Planned::Monitor => "ekips monitor",
            Planned::FlashFirmware => "ekips flash-firmware",
            Planned::BackupFirmware => "ekips backup-firmware",
        }
    }

    /// Why the command is not there yet.
    pub fn reason(self, lang: Lang) -> &'static str {
        match self {
            Planned::RunSim => t(
                lang,
                "The simulator needs the Ekips Prime Pro virtual machine, which is still being built.",
                "Der Simulator braucht die virtuelle Maschine von Ekips Prime Pro, die gerade noch gebaut wird.",
            ),
            Planned::RunHub | Planned::Upload | Planned::Monitor => t(
                lang,
                "Talking to the hub needs the Ekips Prime Pro firmware, which is still being built.",
                "Für die Verbindung zum Hub wird die Ekips-Prime-Pro-Firmware gebraucht, die gerade noch gebaut wird.",
            ),
            Planned::FlashFirmware | Planned::BackupFirmware => t(
                lang,
                "The Ekips Prime Pro firmware is still being built. Your hub has not been changed.",
                "Die Ekips-Prime-Pro-Firmware wird gerade noch gebaut. An deinem Hub wurde nichts verändert.",
            ),
        }
    }
}

/// Prints a friendly "not available yet" message.
pub fn run(planned: Planned, lang: Lang, out: &mut Output) -> ExitStatus {
    let headline = match lang {
        Lang::En => format!("`{}` is not available yet.", planned.command()),
        Lang::De => format!("`{}` gibt es noch nicht.", planned.command()),
    };
    out.error(&headline);
    out.note(planned.reason(lang));
    out.note(t(
        lang,
        "Until then you can find mistakes in your program with `ekips check <file>`.",
        "Bis dahin kannst du dein Programm mit `ekips check <datei>` auf Fehler prüfen.",
    ));
    ExitStatus::NotAvailable
}
