use statcraft_app::system_memory::{
    check_memory_guard_for_files, get_system_memory, SystemMemoryInfo,
};

#[test]
fn test_memory_guard_available_and_valid() {
    let mem = get_system_memory();
    assert!(
        mem.is_some(),
        "Expected get_system_memory() to return valid memory stats on current OS"
    );

    let info = mem.unwrap();
    assert!(info.total_bytes > 0);
    assert!(info.available_bytes > 0);
    assert!(info.available_bytes <= info.total_bytes);
    assert!(info.total_mb() > 0);
    assert!(info.available_ratio() > 0.0 && info.available_ratio() <= 1.0);
}

#[test]
fn test_memory_guard_thresholds() {
    let mock_low = SystemMemoryInfo {
        total_bytes: 4 * 1024 * 1024 * 1024,
        available_bytes: 300 * 1024 * 1024,
    };
    assert!(mock_low.is_low_memory());

    let mock_ok = SystemMemoryInfo {
        total_bytes: 32 * 1024 * 1024 * 1024,
        available_bytes: 16 * 1024 * 1024 * 1024,
    };
    assert!(!mock_ok.is_low_memory());
}

#[test]
fn test_check_memory_guard_small_batch_no_warning() {
    // 5 files of 10 MB each should not trigger warning on normal system
    let warning = check_memory_guard_for_files(5, 50 * 1024 * 1024);
    if let Some(mem) = get_system_memory() {
        if !mem.is_low_memory() {
            assert!(warning.is_none());
        }
    }
}
