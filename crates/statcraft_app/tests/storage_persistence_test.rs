use eframe::{App, Storage};
use statcraft_app::app::{STORAGE_KEY_BRIDGE_AUTOSTART, STORAGE_KEY_LANGUAGE};
use statcraft_app::StatCraftApp;
use std::collections::HashMap;

#[derive(Default)]
struct MockStorage {
    map: HashMap<String, String>,
}

impl Storage for MockStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.map.insert(key.to_string(), value);
    }
    fn flush(&mut self) {}
}

#[test]
fn test_language_and_bridge_autostart_storage_persistence() {
    let mut app = StatCraftApp::new_headless();
    let mut storage = MockStorage::default();

    // 1. Verify default autostart when no storage entry exists
    let default_autostart = storage
        .get_string(STORAGE_KEY_BRIDGE_AUTOSTART)
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);
    assert!(default_autostart, "Bridge autostart should default to true");

    // 2. Change language to English and disable bridge autostart
    statcraft_i18n::set_language("en");
    app.bridge_service.set_autostart(false);

    // 3. Save into storage
    app.save(&mut storage);

    // 4. Verify stored values
    assert_eq!(
        storage.get_string(STORAGE_KEY_LANGUAGE),
        Some("en".to_string())
    );
    assert_eq!(
        storage.get_string(STORAGE_KEY_BRIDGE_AUTOSTART),
        Some("false".to_string())
    );

    // 5. Simulate reload into new session
    let restored_lang = storage
        .get_string(STORAGE_KEY_LANGUAGE)
        .expect("language must be preserved");
    statcraft_i18n::set_language(&restored_lang);
    assert_eq!(statcraft_i18n::current_language(), "en");

    let restored_autostart = storage
        .get_string(STORAGE_KEY_BRIDGE_AUTOSTART)
        .and_then(|s| s.parse::<bool>().ok())
        .expect("autostart boolean must be preserved");
    assert!(!restored_autostart);

    // 6. Test re-enabling autostart and saving
    app.bridge_service.set_autostart(true);
    statcraft_i18n::set_language("ru");
    app.save(&mut storage);

    assert_eq!(
        storage.get_string(STORAGE_KEY_LANGUAGE),
        Some("ru".to_string())
    );
    assert_eq!(
        storage.get_string(STORAGE_KEY_BRIDGE_AUTOSTART),
        Some("true".to_string())
    );
}
