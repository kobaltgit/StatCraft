use super::common::{
    format_var_display, render_variable_list_panel, ActionBarAction, DialogActionBar, DialogShell,
    ListEvent, TransferButton, VariableListItem,
};
use egui::{vec2, ComboBox, ScrollArea, Stroke, TextEdit, TextStyle, Ui};
use statcraft_core::DataSet;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FuncGroup {
    #[default]
    All,
    Arithmetic,
    Statistical,
}

#[derive(Debug, Clone, Default)]
pub struct ComputeDialogState {
    pub is_open: bool,
    pub target_var: String,
    pub label: String,
    pub expression: String,
    pub selected_var_idx: Option<usize>,
    pub func_group: FuncGroup,
    pub selected_func_idx: Option<usize>,
}

pub enum ComputeDialogAction {
    Run {
        target: String,
        expression: String,
        label: Option<String>,
    },
    Paste {
        target: String,
        expression: String,
        label: Option<String>,
    },
    None,
}

const ALL_FUNCTIONS: &[(&str, &str, FuncGroup)] = &[
    ("ABS(x)", "ABS(?)", FuncGroup::Arithmetic),
    ("EXP(x)", "EXP(?)", FuncGroup::Arithmetic),
    ("LN(x)", "LN(?)", FuncGroup::Arithmetic),
    ("LOG10(x)", "LOG10(?)", FuncGroup::Arithmetic),
    ("MAX(x1, x2, ...)", "MAX(?, ?)", FuncGroup::Statistical),
    ("MEAN(x1, x2, ...)", "MEAN(?, ?)", FuncGroup::Statistical),
    ("MIN(x1, x2, ...)", "MIN(?, ?)", FuncGroup::Statistical),
    ("RND(x)", "RND(?)", FuncGroup::Arithmetic),
    ("SQRT(x)", "SQRT(?)", FuncGroup::Arithmetic),
    ("SUM(x1, x2, ...)", "SUM(?, ?)", FuncGroup::Statistical),
    ("TRUNC(x)", "TRUNC(?)", FuncGroup::Arithmetic),
];

pub fn render_compute_dialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut ComputeDialogState,
) -> ComputeDialogAction {
    if !state.is_open {
        return ComputeDialogAction::None;
    }

    let ds = match dataset {
        Some(d) => d,
        None => {
            state.is_open = false;
            return ComputeDialogAction::None;
        }
    };

    let mut action = ComputeDialogAction::None;
    let mut close_dialog = false;

    // Available variables
    let available_vars: Vec<(usize, String, String, statcraft_core::Measure)> = (0..ds.col_count())
        .filter_map(|col_idx| {
            ds.dictionary
                .get(col_idx)
                .map(|meta| (col_idx, meta.name.clone(), meta.label.clone(), meta.measure))
        })
        .collect();

    DialogShell::compute(t!("dialog-compute-title")).show(ctx, &mut state.is_open, |ui| {
        // 1. Target Variable and Label row
        ui.horizontal(|ui| {
            ui.label(t!("dialog-compute-target-var"));
            ui.add(
                TextEdit::singleline(&mut state.target_var)
                    .desired_width(140.0)
                    .hint_text("VAR_NAME"),
            );

            ui.add_space(10.0);

            ui.label(t!("dialog-compute-label"));
            ui.add(
                TextEdit::singleline(&mut state.label)
                    .desired_width(220.0)
                    .hint_text("Description"),
            );
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // 2. Middle work area: Left (Variables) | Center (Expression & Keypad) | Right (Functions)
        ui.horizontal(|ui| {
            // Left: Variable list panel
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

            let var_event = render_variable_list_panel(
                ui,
                t!("dialog-variables-source"),
                vec2(150.0, 260.0),
                "compute_source_vars",
                &var_items,
            );

            match var_event {
                ListEvent::Clicked(idx) => {
                    state.selected_var_idx = Some(idx);
                }
                ListEvent::DoubleClicked(idx) => {
                    state.selected_var_idx = Some(idx);
                    if let Some((_, var_name, _, _)) = available_vars.get(idx) {
                        insert_text(&mut state.expression, var_name);
                    }
                }
                ListEvent::None => {}
            }

            // Transfer button to expression
            ui.vertical(|ui| {
                ui.add_space(100.0);
                let can_transfer = state.selected_var_idx.is_some();
                if TransferButton::to_target(can_transfer).show(ui).clicked() {
                    if let Some(idx) = state.selected_var_idx {
                        if let Some((_, var_name, _, _)) = available_vars.get(idx) {
                            insert_text(&mut state.expression, var_name);
                        }
                    }
                }
            });

            // Center: Expression editor + Keypad
            ui.vertical(|ui| {
                ui.label(t!("dialog-compute-expression"));

                ui.add(
                    TextEdit::multiline(&mut state.expression)
                        .desired_rows(4)
                        .desired_width(220.0)
                        .font(TextStyle::Monospace),
                );

                ui.add_space(6.0);

                // Calculator keypad
                render_keypad(ui, &mut state.expression);
            });

            // Right: Function group & Functions list
            ui.vertical(|ui| {
                ui.label(t!("dialog-compute-func-group"));
                ComboBox::from_id_salt("compute_func_group")
                    .width(130.0)
                    .selected_text(match state.func_group {
                        FuncGroup::All => t!("dialog-compute-group-all"),
                        FuncGroup::Arithmetic => t!("dialog-compute-group-arithmetic"),
                        FuncGroup::Statistical => t!("dialog-compute-group-statistical"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.func_group,
                            FuncGroup::All,
                            t!("dialog-compute-group-all"),
                        );
                        ui.selectable_value(
                            &mut state.func_group,
                            FuncGroup::Arithmetic,
                            t!("dialog-compute-group-arithmetic"),
                        );
                        ui.selectable_value(
                            &mut state.func_group,
                            FuncGroup::Statistical,
                            t!("dialog-compute-group-statistical"),
                        );
                    });

                ui.add_space(4.0);
                ui.label(t!("dialog-compute-functions"));

                let filtered_funcs: Vec<&(&str, &str, FuncGroup)> = ALL_FUNCTIONS
                    .iter()
                    .filter(|(_, _, group)| match state.func_group {
                        FuncGroup::All => true,
                        g => *group == g,
                    })
                    .collect();

                egui::Frame::canvas(ui.style())
                    .fill(crate::theme::Tokens::BG_PANEL)
                    .stroke(Stroke::new(1.0_f32, crate::theme::Tokens::BORDER))
                    .show(ui, |ui| {
                        ui.set_min_size(vec2(130.0, 195.0));
                        ui.set_max_size(vec2(130.0, 195.0));

                        ScrollArea::vertical()
                            .id_salt("compute_func_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (idx, (display_name, snippet, _)) in
                                    filtered_funcs.iter().enumerate()
                                {
                                    let is_selected = state.selected_func_idx == Some(idx);
                                    let res = ui.selectable_label(is_selected, *display_name);
                                    if res.clicked() {
                                        state.selected_func_idx = Some(idx);
                                    }
                                    if res.double_clicked() {
                                        state.selected_func_idx = Some(idx);
                                        insert_text(&mut state.expression, snippet);
                                    }
                                }
                            });
                    });
            });
        });

        // 3. Bottom Action Bar
        let can_run = !state.target_var.trim().is_empty() && !state.expression.trim().is_empty();
        let bar_action = DialogActionBar::new(can_run).show(ui);

        match bar_action {
            ActionBarAction::Ok => {
                action = ComputeDialogAction::Run {
                    target: state.target_var.trim().to_string(),
                    expression: state.expression.trim().to_string(),
                    label: if state.label.trim().is_empty() {
                        None
                    } else {
                        Some(state.label.trim().to_string())
                    },
                };
                close_dialog = true;
            }
            ActionBarAction::Paste => {
                action = ComputeDialogAction::Paste {
                    target: state.target_var.trim().to_string(),
                    expression: state.expression.trim().to_string(),
                    label: if state.label.trim().is_empty() {
                        None
                    } else {
                        Some(state.label.trim().to_string())
                    },
                };
                close_dialog = true;
            }
            ActionBarAction::Reset => {
                state.target_var.clear();
                state.label.clear();
                state.expression.clear();
                state.selected_var_idx = None;
                state.selected_func_idx = None;
            }
            ActionBarAction::Cancel => {
                close_dialog = true;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if close_dialog {
        state.is_open = false;
    }

    action
}

fn insert_text(dest: &mut String, text: &str) {
    if !dest.is_empty() && !dest.ends_with(' ') && !dest.ends_with('(') {
        dest.push(' ');
    }
    dest.push_str(text);
}

fn render_keypad(ui: &mut Ui, expr: &mut String) {
    let btn_size = vec2(30.0, 24.0);

    // Row 1: Operators
    ui.horizontal(|ui| {
        if ui.add_sized(btn_size, egui::Button::new("+")).clicked() {
            insert_text(expr, "+");
        }
        if ui.add_sized(btn_size, egui::Button::new("-")).clicked() {
            insert_text(expr, "-");
        }
        if ui.add_sized(btn_size, egui::Button::new("*")).clicked() {
            insert_text(expr, "*");
        }
        if ui.add_sized(btn_size, egui::Button::new("/")).clicked() {
            insert_text(expr, "/");
        }
        if ui
            .add_sized(vec2(36.0, 24.0), egui::Button::new("**"))
            .clicked()
        {
            insert_text(expr, "**");
        }
        if ui.add_sized(btn_size, egui::Button::new("(")).clicked() {
            insert_text(expr, "(");
        }
        if ui.add_sized(btn_size, egui::Button::new(")")).clicked() {
            insert_text(expr, ")");
        }
    });

    ui.add_space(2.0);

    // Number Pad
    let pad = [
        ["7", "8", "9"],
        ["4", "5", "6"],
        ["1", "2", "3"],
        ["0", ".", "C"],
    ];

    for row in pad {
        ui.horizontal(|ui| {
            for key in row {
                if key == "C" {
                    if ui.add_sized(btn_size, egui::Button::new("C")).clicked() {
                        expr.pop();
                    }
                } else if ui.add_sized(btn_size, egui::Button::new(key)).clicked() {
                    if expr.is_empty()
                        || expr.ends_with('(')
                        || expr.ends_with(' ')
                        || key == "."
                        || expr
                            .chars()
                            .last()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    {
                        expr.push_str(key);
                    } else {
                        expr.push(' ');
                        expr.push_str(key);
                    }
                }
            }
        });
        ui.add_space(1.0);
    }
}
