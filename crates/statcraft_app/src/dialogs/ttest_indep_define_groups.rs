use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use super::ttest_indep_dialog::IndependentSamplesTTestDialogState;
use statcraft_i18n::t;

pub fn render_define_groups_subdialog(
    ctx: &egui::Context,
    state: &mut IndependentSamplesTTestDialogState,
) {
    if !state.is_define_groups_open {
        return;
    }
    let mut def_open = true;
    DialogShell::subdialog(t!("dialog-ttest-indep-define-groups-title")).show(
        ctx,
        &mut def_open,
        |ui| {
            GroupBox::new(t!("dialog-ttest-indep-define-groups-title")).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("dialog-ttest-indep-group1"));
                    ui.add(egui::TextEdit::singleline(&mut state.temp_group1).desired_width(120.0));
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(t!("dialog-ttest-indep-group2"));
                    ui.add(egui::TextEdit::singleline(&mut state.temp_group2).desired_width(120.0));
                });
            });

            let can_continue = !state.temp_group1.trim().is_empty()
                && !state.temp_group2.trim().is_empty()
                && state.temp_group1.trim() != state.temp_group2.trim();

            let sub_action = SubdialogActionBar::new(can_continue).show(ui);
            match sub_action {
                SubdialogAction::Continue => {
                    state.group1_val = state.temp_group1.trim().to_string();
                    state.group2_val = state.temp_group2.trim().to_string();
                    state.is_define_groups_open = false;
                }
                SubdialogAction::Cancel => {
                    state.is_define_groups_open = false;
                }
                SubdialogAction::Help | SubdialogAction::None => {}
            }
        },
    );

    if !def_open {
        state.is_define_groups_open = false;
    }
}
