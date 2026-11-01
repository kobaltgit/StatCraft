use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, TransferButton, VariableListItem,
};
use crate::theme::Tokens;
use egui::{vec2, Color32, ScrollArea, Stroke, TextEdit};
use statcraft_algo::ProgrammabilityScript;
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub struct ProgrammabilityDialogState {
    pub is_open: bool,
    pub script: String,
    pub selected_var_idx: Option<usize>,
}

impl Default for ProgrammabilityDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            script: String::from(
                "# StatCraft Programmability Transformation Script\n# Example: target = salary * 1.1\n# Example: IF educ >= 16 THEN high_educ = 1 ELSE 0\n",
            ),
            selected_var_idx: None,
        }
    }
}

pub enum ProgrammabilityDialogAction {
    Run { script: String },
    Paste { script: String },
    None,
}

pub fn render_programmability_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ProgrammabilityDialogState,
) -> ProgrammabilityDialogAction {
    if !state.is_open {
        return ProgrammabilityDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ProgrammabilityDialogAction::None;
        }
    };

    let mut action = ProgrammabilityDialogAction::None;
    let mut close_dialog = false;
    let mut reset_requested = false;

    // Available dataset variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    // Check script syntax validity
    let parse_result = ProgrammabilityScript::parse(&state.script);
    let is_valid = parse_result.is_ok();

    DialogShell::standard(t!("dialog-programmability-title")).show(ctx, &mut state.is_open, |ui| {
        ui.horizontal(|ui| {
            // 1. Left panel: Variable Picker (width ~190)
            ui.vertical(|ui| {
                let var_items: Vec<VariableListItem> = available_vars
                    .iter()
                    .enumerate()
                    .map(|(idx, (_, name, label, measure))| VariableListItem {
                        name,
                        display_text: format_var_display(name, label),
                        measure: *measure,
                        is_selected: state.selected_var_idx == Some(idx),
                    })
                    .collect();

                let list_event = render_variable_list_panel(
                    ui,
                    t!("dialog-programmability-variables"),
                    vec2(190.0, 280.0),
                    "programmability_var_list",
                    &var_items,
                );

                match list_event {
                    ListEvent::Clicked(idx) => {
                        state.selected_var_idx = Some(idx);
                    }
                    ListEvent::DoubleClicked(idx) => {
                        state.selected_var_idx = Some(idx);
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            state.script.push_str(name);
                            state.script.push(' ');
                        }
                    }
                    ListEvent::None => {}
                }
            });

            // 2. Middle transfer column
            ui.vertical(|ui| {
                ui.add_space(80.0);
                let transfer_enabled = state.selected_var_idx.is_some();
                if TransferButton::to_target(transfer_enabled)
                    .show(ui)
                    .clicked()
                {
                    if let Some(idx) = state.selected_var_idx {
                        if let Some((_, name, _, _)) = available_vars.get(idx) {
                            state.script.push_str(name);
                            state.script.push(' ');
                        }
                    }
                }
            });

            // 3. Right panel: Script Editor and Snippets
            ui.vertical(|ui| {
                ui.label(t!("dialog-programmability-script-label"));

                // Quick snippet insertion toolbar
                ui.horizontal(|ui| {
                    if ui
                        .button(t!("dialog-programmability-snippet-formula"))
                        .clicked()
                    {
                        state.script.push_str("new_var = salary * 1.05\n");
                    }
                    if ui
                        .button(t!("dialog-programmability-snippet-ifelse"))
                        .clicked()
                    {
                        state
                            .script
                            .push_str("IF educ >= 16 THEN high_educ = 1 ELSE high_educ = 0\n");
                    }
                    if ui
                        .button(t!("dialog-programmability-snippet-normalize"))
                        .clicked()
                    {
                        state.script.push_str("z_score = (salary - 10000) / 2500\n");
                    }
                    if ui
                        .button(t!("dialog-programmability-snippet-math"))
                        .clicked()
                    {
                        state.script.push_str("log_salary = LN(salary)\n");
                    }
                });

                ui.add_space(4.0);

                // Multiline script editor
                ScrollArea::vertical()
                    .id_salt("prog_script_scroll")
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut state.script)
                                .desired_rows(12)
                                .desired_width(f32::INFINITY)
                                .code_editor(),
                        );
                    });

                ui.add_space(6.0);

                // Syntax validation indicator (Vector-First)
                ui.horizontal(|ui| {
                    let (icon_color, status_text) = match &parse_result {
                        Ok(script) => (
                            Color32::from_rgb(46, 125, 50),
                            format!(
                                "{} ({} {})",
                                t!("dialog-programmability-status-valid"),
                                script.rules.len(),
                                t!("dialog-programmability-rules-count")
                            ),
                        ),
                        Err(statcraft_algo::ProgrammabilityError::EmptyScript) => (
                            Tokens::TEXT_MUTED,
                            t!("dialog-programmability-status-empty"),
                        ),
                        Err(e) => (
                            Color32::from_rgb(198, 40, 40),
                            format!("{}: {}", t!("dialog-programmability-status-error"), e),
                        ),
                    };

                    let (rect, _response) =
                        ui.allocate_exact_size(vec2(12.0, 12.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 4.5_f32, icon_color);
                    ui.painter().circle_stroke(
                        rect.center(),
                        4.5_f32,
                        Stroke::new(1.0_f32, icon_color.linear_multiply(0.8_f32)),
                    );

                    ui.colored_label(icon_color, status_text);
                });
            });
        });

        ui.add_space(10.0);

        // Bottom action bar (Type 1: OK, Paste, Reset, Cancel, Help)
        let bar_action = DialogActionBar::new(is_valid).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = ProgrammabilityDialogAction::Run {
                    script: state.script.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = ProgrammabilityDialogAction::Paste {
                    script: state.script.clone(),
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                reset_requested = true;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if reset_requested {
        *state = ProgrammabilityDialogState::default();
        state.is_open = true;
    } else if close_dialog {
        state.is_open = false;
    }

    action
}
