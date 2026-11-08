//! Language selection for the Ekips Prime Pro tools.
//!
//! All student-facing output exists in English and German. This crate
//! decides which one to use and offers the [`t`] helper for picking the
//! right text.
//!
//! ```
//! use ekips_i18n::{Lang, t};
//! assert_eq!(t(Lang::De, "Hello", "Hallo"), "Hallo");
//! ```

pub use ekips_compiler::Lang;

/// Name of the environment variable that overrides the system language.
pub const LANG_ENV: &str = "EKIPS_LANG";

/// Picks the English or German text.
pub fn t<'a>(lang: Lang, en: &'a str, de: &'a str) -> &'a str {
    match lang {
        Lang::En => en,
        Lang::De => de,
    }
}

/// Parses a language code such as `en`, `de`, `de-DE` or `de_AT.UTF-8`.
///
/// Returns `None` for languages that are not supported.
pub fn parse_lang(code: &str) -> Option<Lang> {
    let primary: String = code
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();
    match primary.as_str() {
        "en" => Some(Lang::En),
        "de" => Some(Lang::De),
        _ => None,
    }
}

/// Language from a locale string; unsupported languages fall back to English.
pub fn lang_from_locale(locale: Option<&str>) -> Lang {
    locale.and_then(parse_lang).unwrap_or(Lang::En)
}

/// The default language: `EKIPS_LANG` if set and valid, otherwise the
/// system language, otherwise English.
pub fn detect_lang() -> Lang {
    if let Some(lang) = std::env::var(LANG_ENV).ok().as_deref().and_then(parse_lang) {
        return lang;
    }
    lang_from_locale(sys_locale::get_locale().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_locale_formats() {
        assert_eq!(parse_lang("de"), Some(Lang::De));
        assert_eq!(parse_lang("DE"), Some(Lang::De));
        assert_eq!(parse_lang("de-DE"), Some(Lang::De));
        assert_eq!(parse_lang("de_AT.UTF-8"), Some(Lang::De));
        assert_eq!(parse_lang("en-US"), Some(Lang::En));
        assert_eq!(parse_lang("fr-FR"), None);
        assert_eq!(parse_lang(""), None);
    }

    #[test]
    fn unsupported_locale_falls_back_to_english() {
        assert_eq!(lang_from_locale(Some("fr-FR")), Lang::En);
        assert_eq!(lang_from_locale(None), Lang::En);
        assert_eq!(lang_from_locale(Some("de-CH")), Lang::De);
    }
}
