use std::env;

/// Detects the system or environment language tag (e.g. "ru", "en", "de").
///
/// Priority order:
/// 1. Environment variable `STATCRAFT_LANG` (explicit process override)
/// 2. System locale via OS API (`sys_locale::get_locale()`)
/// 3. POSIX environment variables: `LC_ALL`, `LC_MESSAGES`, `LANG`
pub fn detect_system_language() -> Option<String> {
    // 1. STATCRAFT_LANG override
    if let Ok(val) = env::var("STATCRAFT_LANG") {
        let normalized = normalize_language_tag(&val);
        if !normalized.is_empty() {
            return Some(normalized);
        }
    }

    // 2. sys_locale OS API
    if let Some(locale) = sys_locale::get_locale() {
        let normalized = normalize_language_tag(&locale);
        if !normalized.is_empty() {
            return Some(normalized);
        }
    }

    // 3. Fallback standard locale environment variables
    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = env::var(var) {
            let normalized = normalize_language_tag(&val);
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }

    None
}

/// Normalizes a locale string (e.g. "ru-RU", "ru_RU.UTF-8", "en-US", "EN", "de_DE@euro")
/// to a lowercase primary language tag ("ru", "en", "de").
pub fn normalize_language_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Strip encoding / charset (.UTF-8, .utf8) and modifier (@euro)
    let base = trimmed
        .split('.')
        .next()
        .unwrap_or("")
        .split('@')
        .next()
        .unwrap_or("");

    // Extract primary language code before delimiter ('-' or '_')
    base.split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_language_tag() {
        assert_eq!(normalize_language_tag("ru-RU"), "ru");
        assert_eq!(normalize_language_tag("ru_RU.UTF-8"), "ru");
        assert_eq!(normalize_language_tag("en-US"), "en");
        assert_eq!(normalize_language_tag("EN"), "en");
        assert_eq!(normalize_language_tag("de_DE@euro"), "de");
        assert_eq!(normalize_language_tag(""), "");
        assert_eq!(normalize_language_tag("   "), "");
    }

    #[test]
    fn test_detect_language_override() {
        env::set_var("STATCRAFT_LANG", "ru");
        assert_eq!(detect_system_language(), Some("ru".to_string()));
        env::set_var("STATCRAFT_LANG", "en-US");
        assert_eq!(detect_system_language(), Some("en".to_string()));
        env::remove_var("STATCRAFT_LANG");
    }
}
