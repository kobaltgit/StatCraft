use super::common::{ActionBarAction, DialogActionBar, DialogShell, GroupBox};
use egui::vec2;
use statcraft_algo::{SortDirection, VariableSortAttribute};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct SortVariablesDialogState {
    pub is_open: bool,
    pub attribute: VariableSortAttribute,
    pub direction: SortDirection,
}

impl Default for SortVariablesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            attribute: VariableSortAttribute::Name,
            direction: SortDirection::Ascending,
        }
    }
}

pub enum SortVariablesDialogAction {
    Run {
        attribute: VariableSortAttribute,
        direction: SortDirection,
    },
    Paste {
        attribute: VariableSortAttribute,
        direction: SortDirection,
    },
    None,
}

pub fn render_sort_variables_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut SortVariablesDialogState,
) -> SortVariablesDialogAction {
    if !state.is_open {
        return SortVariablesDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return SortVariablesDialogAction::None;
        }
    };

    let mut action = SortVariablesDialogAction::None;
    let mut close_dialog = false;

    let mut main_open = true;
    DialogShell::standard(t!("dialog-sort-variables-title")).show(ctx, &mut main_open, |ui| {
        ui.horizontal(|ui| {
            // Left GroupBox: Sort By Attribute (260x260)
            ui.allocate_ui_with_layout(
                vec2(260.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    GroupBox::new(t!("dialog-sort-vars-attribute-group")).show(ui, |ui| {
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Name,
                            t!("sort-attr-name"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Type,
                            t!("sort-attr-type"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Label,
                            t!("sort-attr-label"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Measure,
                            t!("sort-attr-measure"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Role,
                            t!("sort-attr-role"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Alignment,
                            t!("sort-attr-align"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Width,
                            t!("sort-attr-width"),
                        );
                        ui.radio_value(
                            &mut state.attribute,
                            VariableSortAttribute::Decimals,
                            t!("sort-attr-decimals"),
                        );
                    });
                },
            );

            // Right GroupBox: Sort Order (250x260)
            ui.allocate_ui_with_layout(
                vec2(250.0, 260.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    GroupBox::new(t!("dialog-sort-cases-order-group")).show(ui, |ui| {
                        ui.radio_value(
                            &mut state.direction,
                            SortDirection::Ascending,
                            t!("dialog-sort-cases-order-ascending"),
                        );
                        ui.radio_value(
                            &mut state.direction,
                            SortDirection::Descending,
                            t!("dialog-sort-cases-order-descending"),
                        );
                    });
                },
            );
        });

        // Bottom Action Bar
        let can_run = ds.col_count() > 0;
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                close_dialog = true;
                action = SortVariablesDialogAction::Run {
                    attribute: state.attribute,
                    direction: state.direction,
                };
            }
            ActionBarAction::Paste => {
                close_dialog = true;
                action = SortVariablesDialogAction::Paste {
                    attribute: state.attribute,
                    direction: state.direction,
                };
            }
            ActionBarAction::Reset => {
                state.attribute = VariableSortAttribute::Name;
                state.direction = SortDirection::Ascending;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if !main_open || close_dialog {
        state.is_open = false;
    }

    action
}
