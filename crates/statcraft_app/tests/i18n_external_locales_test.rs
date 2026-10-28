use std::fs;

#[test]
fn test_external_locale_end_to_end() {
    let temp_dir =
        std::env::temp_dir().join(format!("statcraft_e2e_locales_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    let de_file = temp_dir.join("de.ftl");
    let de_content = "lang-name-de = Deutsch\nmenu-file = Datei\n";
    let _ = fs::write(&de_file, de_content);

    // Set environment variable to point to temp dir
    std::env::set_var("STATCRAFT_LOCALES_DIR", temp_dir.to_str().unwrap());

    // Initialize or reload
    let _ = statcraft_i18n::init_i18n();

    // 1. Available languages includes "de"
    let langs = statcraft_i18n::available_languages();
    assert!(
        langs.contains(&"de".to_string()),
        "Available languages {:?} should contain 'de'",
        langs
    );

    // 2. Language display name resolves self-name "Deutsch"
    let display_name = statcraft_i18n::language_name("de");
    assert_eq!(display_name, "Deutsch");

    // 3. Switch language to "de"
    assert!(statcraft_i18n::set_language("de"));
    assert_eq!(statcraft_i18n::current_language(), "de");

    // 4. Overridden key translates to German
    assert_eq!(statcraft_i18n::t!("menu-file"), "Datei");

    // 5. Unspecified key falls back to English
    assert_eq!(statcraft_i18n::t!("menu-edit"), "Edit");

    // Clean up
    std::env::remove_var("STATCRAFT_LOCALES_DIR");
    let _ = fs::remove_dir_all(&temp_dir);
    let _ = statcraft_i18n::set_language("ru");
}
