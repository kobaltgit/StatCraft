use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use super::kruskal_wallis_dialog::KruskalWallisDialogState;
use statcraft_i18n::t;

pub fn render_kruskal_define_range_subdialog(
    ctx: &egui::Context,
    state: &mut KruskalWallisDialogState,
) {
    if !state.is_define_range_open {
        return;
    }
    let mut def_open = true;
    DialogShell::subdialog(t!("dialog-npar-define-range-title")).show(ctx, &mut def_open, |ui| {
        GroupBox::new(t!("dialog-npar-range-group-box")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t!("dialog-npar-range-min"));
                ui.add(egui::TextEdit::singleline(&mut state.temp_min).desired_width(100.0));
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(t!("dialog-npar-range-max"));
                ui.add(egui::TextEdit::singleline(&mut state.temp_max).desired_width(100.0));
            });
        });

        let min_parsed = state.temp_min.trim().parse::<f64>();
        let max_parsed = state.temp_max.trim().parse::<f64>();
        let can_continue = match (min_parsed, max_parsed) {
            (Ok(min_v), Ok(max_v)) => min_v <= max_v,
            _ => false,
        };

        let sub_action = SubdialogActionBar::new(can_continue).show(ui);
        match sub_action {
            SubdialogAction::Continue => {
                if let (Ok(min_v), Ok(max_v)) = (
                    state.temp_min.trim().parse::<f64>(),
                    state.temp_max.trim().parse::<f64>(),
                ) {
                    state.min_range = min_v;
                    state.max_range = max_v;
                    state.is_range_defined = true;
                    state.is_define_range_open = false;
                }
            }
            SubdialogAction::Cancel => {
                state.is_define_range_open = false;
            }
            SubdialogAction::Help | SubdialogAction::None => {}
        }
    });

    if !def_open {
        state.is_define_range_open = false;
    }
}
