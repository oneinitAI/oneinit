// i18n internationalization module — minimal interface for future expansion
//
// Current: English-only strings hardcoded throughout the codebase.
// Future: replace hardcoded strings with t!("key") lookups against a
// translation table loaded from locale files.
//
// Usage (future):
//   i18n::set_locale("zh-CN");
//   println!("{}", i18n::t("install_success"));
//
// For now this module just provides the locale detection + a placeholder
// t() that returns the input string unchanged (pass-through).

use std::sync::OnceLock;

/// Supported locales
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    ZhCN,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::ZhCN => "zh-CN",
        }
    }
}

static CURRENT_LOCALE: OnceLock<Locale> = OnceLock::new();

/// Initialize locale from env (LANG, LC_ALL, LC_MESSAGES) or default to En
pub fn init() {
    let locale = detect_locale();
    let _ = CURRENT_LOCALE.set(locale);
}

/// Detect locale from environment variables
fn detect_locale() -> Locale {
    let candidates = ["LC_ALL", "LC_MESSAGES", "LANG"];
    for var in &candidates {
        if let Ok(val) = std::env::var(var) {
            let lower = val.to_lowercase();
            if lower.starts_with("zh") {
                return Locale::ZhCN;
            }
        }
    }
    // Windows: check system locale
    if cfg!(windows) {
        if let Ok(val) = std::env::var("COMPUTERNAME") {
            let _ = val; // placeholder — real detection would check GetSystemDefaultUILanguage
        }
    }
    Locale::En
}

/// Get current locale
pub fn current() -> Locale {
    *CURRENT_LOCALE.get().unwrap_or(&Locale::En)
}

/// Set locale explicitly (for future runtime switching)
#[allow(dead_code)]
pub fn set(locale: Locale) {
    // OnceLock doesn't allow re-set, so we just store in a static mut for now.
    // In the future this would reload translation tables.
    let _ = locale;
}

/// Translate a key — currently pass-through (returns input unchanged)
///
/// Future: look up in translation table, fall back to key if not found.
/// ```
/// // Current behavior:
/// assert_eq!(i18n::t("hello"), "hello");
/// ```
#[allow(dead_code)]
pub fn t(key: &str) -> &str {
    key
}

/// Translate with formatted arguments — pass-through
#[allow(dead_code)]
pub fn tf(key: &str, args: &[&str]) -> String {
    let mut result = key.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}
