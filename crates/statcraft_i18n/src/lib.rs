use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use unic_langid::LanguageIdentifier;

pub static EMBEDDED_EN: &str = include_str!("../../../locales/en.ftl");
pub static EMBEDDED_RU: &str = include_str!("../../../locales/ru.ftl");

static GLOBAL_I18N: RwLock<Option<I18nCatalog>> = RwLock::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct I18nCatalog {
    current_lang: String,
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl I18nCatalog {
    pub fn new_with_defaults() -> Result<Self, String> {
        let mut catalog = Self {
            current_lang: "ru".to_string(), // Default Russian for ArtCraft, switchable to EN
            bundles: HashMap::new(),
        };

        catalog.load_bundle_from_str("en", EMBEDDED_EN)?;
        catalog.load_bundle_from_str("ru", EMBEDDED_RU)?;

        Ok(catalog)
    }

    pub fn load_bundle_from_str(
        &mut self,
        lang_code: &str,
        ftl_content: &str,
    ) -> Result<(), String> {
        let lang_id: LanguageIdentifier = lang_code
            .parse()
            .map_err(|e| format!("Invalid language identifier '{}': {}", lang_code, e))?;

        let res = FluentResource::try_new(ftl_content.to_string()).map_err(|(_, errors)| {
            format!("Failed to parse FTL for '{}': {:?}", lang_code, errors)
        })?;

        if let Some(bundle) = self.bundles.get_mut(lang_code) {
            bundle.add_resource_overriding(res);
        } else {
            let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
            bundle.set_use_isolating(false);
            bundle.add_resource_overriding(res);
            self.bundles.insert(lang_code.to_string(), bundle);
        }
        Ok(())
    }

    pub fn load_from_dir(&mut self, dir_path: &Path) -> Result<usize, String> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(0);
        }

        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read locale dir {}: {}", dir_path.display(), e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ftl") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(()) = self.load_bundle_from_str(stem, &content) {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    pub fn set_language(&mut self, lang_code: &str) -> bool {
        if self.bundles.contains_key(lang_code) {
            self.current_lang = lang_code.to_string();
            true
        } else {
            false
        }
    }

    pub fn current_language(&self) -> &str {
        &self.current_lang
    }

    pub fn available_languages(&self) -> Vec<String> {
        let mut langs: Vec<String> = self.bundles.keys().cloned().collect();
        langs.sort();
        langs
    }

    pub fn translate(&self, key: &str, args: Option<&FluentArgs>) -> String {
        // 1. Try current language
        if let Some(bundle) = self.bundles.get(&self.current_lang) {
            if let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) {
                let mut errors = vec![];
                let value = bundle.format_pattern(pattern, args, &mut errors);
                if errors.is_empty() {
                    return value.to_string();
                }
            }
        }

        // 2. Fallback to English if current wasn't English
        if self.current_lang != "en" {
            if let Some(bundle) = self.bundles.get("en") {
                if let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, args, &mut errors);
                    if errors.is_empty() {
                        return value.to_string();
                    }
                }
            }
        }

        // 3. Fallback to raw key
        format!("[{}]", key)
    }

    pub fn language_display_name(&self, lang_code: &str) -> String {
        // 1. Try finding native self-name in bundle for lang_code (e.g. lang-name-de or lang-name)
        if let Some(bundle) = self.bundles.get(lang_code) {
            let specific_key = format!("lang-name-{}", lang_code);
            let pattern = bundle
                .get_message(&specific_key)
                .or_else(|| bundle.get_message("lang-name"))
                .and_then(|m| m.value());
            if let Some(pat) = pattern {
                let mut errors = vec![];
                let val = bundle.format_pattern(pat, None, &mut errors);
                if errors.is_empty() && !val.trim().is_empty() {
                    return val.to_string();
                }
            }
        }

        // 2. Try translating "lang-name-{lang_code}" in current language or English fallback
        let key = format!("lang-name-{}", lang_code);
        let tr_val = self.translate(&key, None);
        if !tr_val.starts_with('[') && !tr_val.ends_with(']') {
            return tr_val;
        }

        // 3. Fallback: uppercase code (e.g. "DE", "ES", "FR")
        lang_code.to_uppercase()
    }

    pub fn discover_and_load_locales(&mut self) -> usize {
        let mut count = 0;

        // 1. Environment variable override
        if let Ok(env_dir) = std::env::var("STATCRAFT_LOCALES_DIR") {
            if let Ok(loaded) = self.load_from_dir(Path::new(&env_dir)) {
                count += loaded;
            }
        }

        // 2. Folder adjacent to current executable (for portable releases)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let locales_path = exe_dir.join("locales");
                if let Ok(loaded) = self.load_from_dir(&locales_path) {
                    count += loaded;
                }

                // 2b. macOS app bundle Resources directory (Contents/Resources/locales)
                #[cfg(target_os = "macos")]
                {
                    let mac_locales = exe_dir.join("../Resources/locales");
                    if let Ok(loaded) = self.load_from_dir(&mac_locales) {
                        count += loaded;
                    }
                }
            }
        }

        // 3. System-wide share directory for Linux packages (/usr/share/statcraft/locales)
        #[cfg(target_os = "linux")]
        {
            let linux_share = Path::new("/usr/share/statcraft/locales");
            if let Ok(loaded) = self.load_from_dir(linux_share) {
                count += loaded;
            }
        }

        // 4. Current working directory ./locales (for cargo run and terminal launches)
        let local_locales = Path::new("locales");
        if let Ok(loaded) = self.load_from_dir(local_locales) {
            count += loaded;
        }

        count
    }
}

pub fn init_i18n() -> Result<(), String> {
    let mut catalog = I18nCatalog::new_with_defaults()?;
    catalog.discover_and_load_locales();
    let mut lock = GLOBAL_I18N.write();
    if let Some(existing) = lock.as_ref() {
        catalog.set_language(existing.current_language());
    }
    *lock = Some(catalog);
    INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn language_name(lang_code: &str) -> String {
    let lock = GLOBAL_I18N.read();
    if let Some(catalog) = lock.as_ref() {
        catalog.language_display_name(lang_code)
    } else {
        lang_code.to_uppercase()
    }
}

pub fn reload_locales() -> Result<usize, String> {
    let mut lock = GLOBAL_I18N.write();
    if let Some(catalog) = lock.as_mut() {
        Ok(catalog.discover_and_load_locales())
    } else {
        Err("I18nCatalog is not initialized".to_string())
    }
}

pub fn set_language(lang_code: &str) -> bool {
    let mut lock = GLOBAL_I18N.write();
    if let Some(catalog) = lock.as_mut() {
        catalog.set_language(lang_code)
    } else {
        false
    }
}

pub fn current_language() -> String {
    let lock = GLOBAL_I18N.read();
    if let Some(catalog) = lock.as_ref() {
        catalog.current_language().to_string()
    } else {
        "en".to_string()
    }
}

pub fn available_languages() -> Vec<String> {
    let lock = GLOBAL_I18N.read();
    if let Some(catalog) = lock.as_ref() {
        catalog.available_languages()
    } else {
        vec!["en".to_string(), "ru".to_string()]
    }
}

pub fn tr(key: &str) -> String {
    if !INITIALIZED.load(Ordering::SeqCst) {
        let _ = init_i18n();
    }

    let lock = GLOBAL_I18N.read();
    if let Some(catalog) = lock.as_ref() {
        catalog.translate(key, None)
    } else {
        format!("[{}]", key)
    }
}

pub fn tr_args(key: &str, args: &FluentArgs) -> String {
    if !INITIALIZED.load(Ordering::SeqCst) {
        let _ = init_i18n();
    }

    let lock = GLOBAL_I18N.read();
    if let Some(catalog) = lock.as_ref() {
        catalog.translate(key, Some(args))
    } else {
        format!("[{}]", key)
    }
}

pub use fluent_bundle;

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::tr($key)
    };
    ($key:expr, $($arg_name:ident = $arg_val:expr),* $(,)?) => {{
        let mut args = $crate::fluent_bundle::FluentArgs::new();
        $(
            args.set(stringify!($arg_name), $arg_val);
        )*
        $crate::tr_args($key, &args)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_t_macro_default_is_russian() {
        let _ = init_i18n();
        assert_eq!(t!("menu-file"), "Файл");
    }

    #[test]
    fn test_translate_known_key() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        catalog.set_language("en");
        assert_eq!(catalog.translate("menu-file", None), "File");

        catalog.set_language("ru");
        assert_eq!(catalog.translate("menu-file", None), "Файл");
    }

    #[test]
    fn test_fallback_to_english() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        // Create a custom locale without "menu-file"
        catalog
            .load_bundle_from_str("de", "other-key = Hallo")
            .unwrap();
        catalog.set_language("de");

        // Should fall back to "File" from English
        assert_eq!(catalog.translate("menu-file", None), "File");
        // And use its own key for known ones
        assert_eq!(catalog.translate("other-key", None), "Hallo");
    }

    #[test]
    fn test_missing_key_format() {
        let catalog = I18nCatalog::new_with_defaults().unwrap();
        assert_eq!(
            catalog.translate("non-existent-key-123", None),
            "[non-existent-key-123]"
        );
    }

    #[test]
    fn test_parameter_interpolation() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        catalog.set_language("en");

        let mut args = FluentArgs::new();
        args.set("count", 100);
        let result = catalog.translate("status-rows-count", Some(&args));
        assert_eq!(result, "Rows: 100");

        catalog.set_language("ru");
        let result_ru = catalog.translate("status-rows-count", Some(&args));
        assert_eq!(result_ru, "Наблюдений: 100");
    }

    #[test]
    fn test_language_switching() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        assert!(catalog.set_language("en"));
        assert_eq!(catalog.current_language(), "en");
        assert!(catalog.set_language("ru"));
        assert_eq!(catalog.current_language(), "ru");
        assert!(!catalog.set_language("non-existent-lang"));
    }

    #[test]
    fn test_available_languages() {
        let catalog = I18nCatalog::new_with_defaults().unwrap();
        let langs = catalog.available_languages();
        assert!(langs.contains(&"en".to_string()));
        assert!(langs.contains(&"ru".to_string()));
    }

    #[test]
    fn test_external_bundle_overrides_existing_keys() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        // Override one key in Russian
        catalog
            .load_bundle_from_str("ru", "menu-file = Документ")
            .unwrap();
        catalog.set_language("ru");

        // Overridden key returns new value
        assert_eq!(catalog.translate("menu-file", None), "Документ");
        // Non-overridden key still returns original embedded translation
        assert_eq!(catalog.translate("menu-edit", None), "Правка");
    }

    #[test]
    fn test_external_bundle_adds_new_language() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        catalog
            .load_bundle_from_str("es", "lang-name-es = Español\nmenu-file = Archivo")
            .unwrap();

        assert!(catalog.available_languages().contains(&"es".to_string()));
        catalog.set_language("es");
        assert_eq!(catalog.translate("menu-file", None), "Archivo");
        // Unset key falls back to English
        assert_eq!(catalog.translate("menu-edit", None), "Edit");
    }

    #[test]
    fn test_language_display_name_resolution() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        assert_eq!(catalog.language_display_name("ru"), "Русский");
        assert_eq!(catalog.language_display_name("en"), "English");

        // Custom language with native self-name
        catalog
            .load_bundle_from_str("de", "lang-name-de = Deutsch")
            .unwrap();
        assert_eq!(catalog.language_display_name("de"), "Deutsch");

        // Custom language without self-name falls back to uppercase code
        catalog
            .load_bundle_from_str("it", "menu-file = File")
            .unwrap();
        assert_eq!(catalog.language_display_name("it"), "IT");
    }

    #[test]
    fn test_load_from_directory_structure() {
        let temp_dir =
            std::env::temp_dir().join(format!("statcraft_test_locales_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("fr.ftl");
        let _ = fs::write(&test_file, "lang-name-fr = Français\nmenu-file = Fichier");

        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        let loaded = catalog.load_from_dir(&temp_dir).unwrap();
        assert!(loaded >= 1);
        assert!(catalog.available_languages().contains(&"fr".to_string()));
        assert_eq!(catalog.language_display_name("fr"), "Français");

        catalog.set_language("fr");
        assert_eq!(catalog.translate("menu-file", None), "Fichier");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_duplicate_keys_in_bundle_do_not_fail() {
        let mut catalog = I18nCatalog::new_with_defaults().unwrap();
        // Brand new language with duplicate keys inside the file
        let content = "lang-name-it = Italiano\nmenu-file = File\nmenu-file = Archivio\n";
        let res = catalog.load_bundle_from_str("it", content);
        assert!(
            res.is_ok(),
            "Loading bundle with duplicate keys must not fail: {:?}",
            res
        );
        catalog.set_language("it");
        // Must use the overriding (last) definition
        assert_eq!(catalog.translate("menu-file", None), "Archivio");
    }
}
