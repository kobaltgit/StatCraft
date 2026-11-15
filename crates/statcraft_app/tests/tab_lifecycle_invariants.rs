use statcraft_app::{is_tab_closeable, StatCraftApp, Tab};

#[test]
fn test_core_tabs_are_not_closeable() {
    assert!(!is_tab_closeable(Tab::DataView));
    assert!(!is_tab_closeable(Tab::VariableView));
}

#[test]
fn test_auxiliary_tabs_are_closeable() {
    assert!(is_tab_closeable(Tab::OutputViewer));
    assert!(is_tab_closeable(Tab::SyntaxEditor));
}

#[test]
fn test_output_viewer_close_and_reopen_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Verify initial state: OutputViewer exists
    let initial_pos = app.dock_state.find_tab(&Tab::OutputViewer);
    assert!(
        initial_pos.is_some(),
        "OutputViewer should exist in headless app"
    );

    // Close / remove OutputViewer tab
    let pos = initial_pos.unwrap();
    let removed = app.dock_state.remove_tab(pos);
    assert_eq!(removed, Some(Tab::OutputViewer));
    assert!(app.dock_state.find_tab(&Tab::OutputViewer).is_none());

    // Restore via ensure_tab_open
    app.ensure_tab_open(Tab::OutputViewer);

    let restored_pos = app.dock_state.find_tab(&Tab::OutputViewer);
    assert!(restored_pos.is_some(), "OutputViewer must be restored");

    // Verify calling ensure_tab_open again doesn't duplicate
    app.ensure_tab_open(Tab::OutputViewer);
    let mut output_count = 0;
    for (_, tab) in app.dock_state.iter_all_tabs() {
        if *tab == Tab::OutputViewer {
            output_count += 1;
        }
    }
    assert_eq!(output_count, 1, "OutputViewer must not be duplicated");
}

#[test]
fn test_syntax_editor_close_and_reopen_lifecycle() {
    let mut app = StatCraftApp::new_headless();

    // Verify initial state: SyntaxEditor exists
    let initial_pos = app.dock_state.find_tab(&Tab::SyntaxEditor);
    assert!(
        initial_pos.is_some(),
        "SyntaxEditor should exist in headless app"
    );

    // Close / remove SyntaxEditor tab
    let pos = initial_pos.unwrap();
    let removed = app.dock_state.remove_tab(pos);
    assert_eq!(removed, Some(Tab::SyntaxEditor));
    assert!(app.dock_state.find_tab(&Tab::SyntaxEditor).is_none());

    // Restore via ensure_tab_open
    app.ensure_tab_open(Tab::SyntaxEditor);

    let restored_pos = app.dock_state.find_tab(&Tab::SyntaxEditor);
    assert!(restored_pos.is_some(), "SyntaxEditor must be restored");

    // Verify calling ensure_tab_open again doesn't duplicate
    app.ensure_tab_open(Tab::SyntaxEditor);
    let mut syntax_count = 0;
    for (_, tab) in app.dock_state.iter_all_tabs() {
        if *tab == Tab::SyntaxEditor {
            syntax_count += 1;
        }
    }
    assert_eq!(syntax_count, 1, "SyntaxEditor must not be duplicated");
}

#[test]
fn test_ensure_tab_open_for_existing_core_tabs() {
    let mut app = StatCraftApp::new_headless();

    app.ensure_tab_open(Tab::DataView);
    assert!(app.dock_state.find_tab(&Tab::DataView).is_some());

    app.ensure_tab_open(Tab::VariableView);
    assert!(app.dock_state.find_tab(&Tab::VariableView).is_some());
}
