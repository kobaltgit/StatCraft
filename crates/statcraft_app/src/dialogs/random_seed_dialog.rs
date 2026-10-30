use super::common::{ActionBarAction, DialogActionBar, DialogShell, GroupBox};
use egui::TextEdit;
use statcraft_i18n::t;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSeedDialogState {
    pub is_open: bool,
    pub is_random: bool,
    pub seed_text: String,
}

impl Default for RandomSeedDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_random: false,
            seed_text: "2000000".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomSeedDialogAction {
    Run { seed: Option<u64> },
    Paste { seed: Option<u64> },
    None,
}

pub fn render_random_seed_dialog(
    ctx: &egui::Context,
    state: &mut RandomSeedDialogState,
) -> RandomSeedDialogAction {
    if !state.is_open {
        return RandomSeedDialogAction::None;
    }

    let mut action = RandomSeedDialogAction::None;
    let mut is_open = state.is_open;

    DialogShell::properties(t!("dialog-seed-title")).show(ctx, &mut is_open, |ui| {
        ui.add_space(8.0);

        let parsed_seed = if state.is_random {
            Some(None)
        } else {
            let trimmed = state.seed_text.trim();
            match trimmed.parse::<i64>() {
                Ok(val) if (1..=2_000_000_000).contains(&val) => Some(Some(val as u64)),
                _ => None,
            }
        };

        let can_run = parsed_seed.is_some();

        GroupBox::new(t!("dialog-seed-title")).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.radio_value(&mut state.is_random, true, t!("dialog-seed-mode-random"));
                ui.add_space(6.0);
                ui.radio_value(&mut state.is_random, false, t!("dialog-seed-mode-fixed"));
                ui.add_space(6.0);

                ui.indent("seed_value_indent", |ui| {
                    ui.add_enabled_ui(!state.is_random, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t!("dialog-seed-value-label"));
                            ui.add(TextEdit::singleline(&mut state.seed_text).desired_width(130.0));
                        });
                    });
                });
            });
        });

        ui.add_space(16.0);

        let action_bar = DialogActionBar::new(can_run).show(ui);
        match action_bar {
            ActionBarAction::Ok => {
                if let Some(seed) = parsed_seed {
                    action = RandomSeedDialogAction::Run { seed };
                    state.is_open = false;
                }
            }
            ActionBarAction::Paste => {
                if let Some(seed) = parsed_seed {
                    action = RandomSeedDialogAction::Paste { seed };
                    state.is_open = false;
                }
            }
            ActionBarAction::Reset => {
                state.is_random = false;
                state.seed_text = "2000000".to_string();
            }
            ActionBarAction::Cancel => {
                state.is_open = false;
            }
            ActionBarAction::Help | ActionBarAction::None => {}
        }
    });

    if !is_open {
        state.is_open = false;
    }

    action
}
