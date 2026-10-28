use super::common::{ActionBarAction, DialogActionBar, DialogShell};
use egui::{Context, Id};
use statcraft_core::DataType;
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectedTypeKind {
    #[default]
    Numeric,
    Comma,
    Dot,
    Scientific,
    Date,
    Dollar,
    CustomCurrency,
    String,
    RestrictedNumeric,
}

#[derive(Debug, Clone)]
pub struct VariableTypeDialogState {
    pub is_open: bool,
    pub col_index: usize,
    pub var_name: String,
    pub selected_kind: SelectedTypeKind,
    pub width: usize,
    pub decimals: u8,
    pub date_format: String,
    pub custom_currency: String,
}

impl Default for VariableTypeDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            col_index: 0,
            var_name: String::new(),
            selected_kind: SelectedTypeKind::Numeric,
            width: 8,
            decimals: 2,
            date_format: "dd-mmm-yyyy".to_string(),
            custom_currency: "CCA".to_string(),
        }
    }
}

impl VariableTypeDialogState {
    pub fn open(
        col_index: usize,
        var_name: &str,
        current_type: &DataType,
        width: usize,
        decimals: u8,
    ) -> Self {
        let (selected_kind, date_format, custom_currency) = match current_type {
            DataType::Numeric { .. } => (
                SelectedTypeKind::Numeric,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::Comma { .. } => (
                SelectedTypeKind::Comma,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::Dot { .. } => (
                SelectedTypeKind::Dot,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::Scientific { .. } => (
                SelectedTypeKind::Scientific,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::Date { format } => {
                (SelectedTypeKind::Date, format.clone(), "CCA".to_string())
            }
            DataType::Dollar { .. } => (
                SelectedTypeKind::Dollar,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::CustomCurrency { format_code, .. } => (
                SelectedTypeKind::CustomCurrency,
                "dd-mmm-yyyy".to_string(),
                format_code.clone(),
            ),
            DataType::String { .. } => (
                SelectedTypeKind::String,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
            DataType::RestrictedNumeric { .. } => (
                SelectedTypeKind::RestrictedNumeric,
                "dd-mmm-yyyy".to_string(),
                "CCA".to_string(),
            ),
        };

        Self {
            is_open: true,
            col_index,
            var_name: var_name.to_string(),
            selected_kind,
            width,
            decimals,
            date_format,
            custom_currency,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableTypeDialogAction {
    Apply {
        col_index: usize,
        var_type: DataType,
        width: usize,
        decimals: u8,
    },
    Close,
    None,
}

pub fn render_variable_type_dialog(
    ctx: &Context,
    state: &mut VariableTypeDialogState,
) -> VariableTypeDialogAction {
    if !state.is_open {
        return VariableTypeDialogAction::None;
    }

    let mut action = VariableTypeDialogAction::None;
    let window_title = t!("dialog-var-type-title", name = state.var_name.as_str());
    let mut open = true;

    DialogShell::properties(&window_title)
        .id(Id::new("variable_type_modal_dialog"))
        .show(ctx, &mut open, |ui| {
            ui.horizontal(|ui| {
                // Left Column: 9 Type Radio Buttons
                ui.vertical(|ui| {
                    ui.set_width(210.0);

                    let type_options = [
                        (SelectedTypeKind::Numeric, t!("type-numeric")),
                        (SelectedTypeKind::Comma, t!("type-comma")),
                        (SelectedTypeKind::Dot, t!("type-dot")),
                        (SelectedTypeKind::Scientific, t!("type-scientific")),
                        (SelectedTypeKind::Date, t!("type-date")),
                        (SelectedTypeKind::Dollar, t!("type-dollar")),
                        (SelectedTypeKind::CustomCurrency, t!("type-custom-currency")),
                        (SelectedTypeKind::String, t!("type-string")),
                        (
                            SelectedTypeKind::RestrictedNumeric,
                            t!("type-restricted-numeric"),
                        ),
                    ];

                    for (kind, label) in type_options {
                        if ui
                            .radio_value(&mut state.selected_kind, kind, label)
                            .clicked()
                            && state.selected_kind == SelectedTypeKind::String
                        {
                            state.decimals = 0;
                        }
                    }
                });

                ui.separator();

                // Right Column: Parameters for chosen type
                ui.vertical(|ui| {
                    ui.set_width(200.0);

                    match state.selected_kind {
                        SelectedTypeKind::String => {
                            ui.label(t!("dialog-var-type-characters"));
                            ui.add(egui::DragValue::new(&mut state.width).range(1..=32767));
                        }
                        SelectedTypeKind::RestrictedNumeric => {
                            ui.label(t!("dialog-var-type-width"));
                            ui.add(egui::DragValue::new(&mut state.width).range(1..=50));
                        }
                        SelectedTypeKind::Date => {
                            ui.label(t!("dialog-var-type-date-format"));
                            let formats = [
                                "dd-mmm-yyyy",
                                "dd-mmm-yy",
                                "mm/dd/yyyy",
                                "yyyy-mm-dd",
                                "dd.mm.yyyy",
                            ];
                            for fmt in formats {
                                ui.radio_value(&mut state.date_format, fmt.to_string(), fmt);
                            }
                        }
                        SelectedTypeKind::CustomCurrency => {
                            ui.label(t!("dialog-var-type-width"));
                            ui.add(egui::DragValue::new(&mut state.width).range(1..=40));
                            ui.add_space(4.0);
                            ui.label(t!("dialog-var-type-decimals"));
                            ui.add(egui::DragValue::new(&mut state.decimals).range(0..=16));
                            ui.add_space(4.0);
                            ui.label(t!("dialog-var-type-currency"));
                            let currencies = ["CCA", "CCB", "CCC", "CCD", "CCE"];
                            for curr in currencies {
                                ui.radio_value(&mut state.custom_currency, curr.to_string(), curr);
                            }
                        }
                        _ => {
                            ui.label(t!("dialog-var-type-width"));
                            ui.add(egui::DragValue::new(&mut state.width).range(1..=40));
                            ui.add_space(8.0);
                            ui.label(t!("dialog-var-type-decimals"));
                            ui.add(egui::DragValue::new(&mut state.decimals).range(0..=16));
                        }
                    }
                });
            });

            // Bottom action buttons: DialogActionBar (OK, Cancel, Help)
            let bar_action = DialogActionBar::new(true)
                .show_paste(false)
                .show_reset(false)
                .show(ui);

            match bar_action {
                ActionBarAction::Ok => {
                    let var_type = match state.selected_kind {
                        SelectedTypeKind::Numeric => DataType::Numeric {
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::Comma => DataType::Comma {
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::Dot => DataType::Dot {
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::Scientific => DataType::Scientific {
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::Date => DataType::Date {
                            format: state.date_format.clone(),
                        },
                        SelectedTypeKind::Dollar => DataType::Dollar {
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::CustomCurrency => DataType::CustomCurrency {
                            format_code: state.custom_currency.clone(),
                            decimals: state.decimals,
                        },
                        SelectedTypeKind::String => DataType::String {
                            max_len: state.width,
                        },
                        SelectedTypeKind::RestrictedNumeric => {
                            DataType::RestrictedNumeric { width: state.width }
                        }
                    };

                    action = VariableTypeDialogAction::Apply {
                        col_index: state.col_index,
                        var_type,
                        width: state.width,
                        decimals: state.decimals,
                    };
                    state.is_open = false;
                }
                ActionBarAction::Cancel => {
                    action = VariableTypeDialogAction::Close;
                    state.is_open = false;
                }
                ActionBarAction::Help
                | ActionBarAction::Paste
                | ActionBarAction::Reset
                | ActionBarAction::None => {}
            }
        });

    if !open {
        state.is_open = false;
        if action == VariableTypeDialogAction::None {
            action = VariableTypeDialogAction::Close;
        }
    }

    action
}
