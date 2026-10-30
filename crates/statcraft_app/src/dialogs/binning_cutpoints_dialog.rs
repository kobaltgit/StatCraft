use super::binning_dialog::{CutpointMethod, VisualBinningDialogState};
use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use egui::TextEdit;
use statcraft_algo::{
    generate_equal_percentile_cutpoints, generate_equal_width_cutpoints, generate_stddev_cutpoints,
};
use statcraft_core::DataSet;
use statcraft_i18n::t;

pub fn render_binning_cutpoints_subdialog(
    ctx: &egui::Context,
    dataset: Option<&DataSet>,
    state: &mut VisualBinningDialogState,
) {
    if !state.is_cutpoints_subdialog_open {
        return;
    }

    let mut is_open = true;
    DialogShell::subdialog(t!("dialog-binning-make-cutpoints-title")).show(
        ctx,
        &mut is_open,
        |ui| {
            ui.add_space(4.0);

            GroupBox::new(t!("dialog-binning-make-cutpoints-title")).show(ui, |ui| {
                ui.radio_value(
                    &mut state.cutpoint_method,
                    CutpointMethod::EqualWidth,
                    t!("dialog-binning-equal-width"),
                );
                ui.add_space(2.0);

                ui.radio_value(
                    &mut state.cutpoint_method,
                    CutpointMethod::EqualPercentiles,
                    t!("dialog-binning-equal-percentiles"),
                );
                ui.add_space(2.0);

                ui.radio_value(
                    &mut state.cutpoint_method,
                    CutpointMethod::StdDev,
                    t!("dialog-binning-stddev"),
                );
                ui.add_space(6.0);

                if state.cutpoint_method != CutpointMethod::StdDev {
                    ui.horizontal(|ui| {
                        ui.label(t!("dialog-binning-num-intervals"));
                        ui.add(
                            TextEdit::singleline(&mut state.subdialog_num_intervals)
                                .desired_width(60.0),
                        );
                    });
                } else {
                    ui.label(t!("dialog-binning-stddev-desc"));
                }
            });

            ui.add_space(8.0);

            let can_continue = match state.cutpoint_method {
                CutpointMethod::StdDev => true,
                _ => state
                    .subdialog_num_intervals
                    .trim()
                    .parse::<usize>()
                    .map(|n| (1..=100).contains(&n))
                    .unwrap_or(false),
            };

            let action = SubdialogActionBar::new(can_continue).show(ui);
            match action {
                SubdialogAction::Continue => {
                    apply_cutpoints_generation(dataset, state);
                    state.is_cutpoints_subdialog_open = false;
                }
                SubdialogAction::Cancel => {
                    state.is_cutpoints_subdialog_open = false;
                }
                SubdialogAction::None | SubdialogAction::Help => {}
            }
        },
    );

    if !is_open {
        state.is_cutpoints_subdialog_open = false;
    }
}

fn apply_cutpoints_generation(dataset: Option<&DataSet>, state: &mut VisualBinningDialogState) {
    let ds = match dataset {
        Some(d) => d,
        None => return,
    };

    let var_name = match &state.selected_var {
        Some(v) => v,
        None => return,
    };

    let col_idx = match ds.dictionary.find_index(var_name) {
        Some(i) => i,
        None => return,
    };

    let values = match ds.table.columns.get(col_idx) {
        Some(col) => &col.values,
        None => return,
    };

    let num_cutpoints = state
        .subdialog_num_intervals
        .trim()
        .parse::<usize>()
        .unwrap_or(3);
    let num_bins = num_cutpoints + 1;

    let generated = match state.cutpoint_method {
        CutpointMethod::EqualWidth => generate_equal_width_cutpoints(values, num_bins),
        CutpointMethod::EqualPercentiles => generate_equal_percentile_cutpoints(values, num_bins),
        CutpointMethod::StdDev => generate_stddev_cutpoints(values),
    };

    state.cutpoints.clear();
    for cp in generated {
        state.cutpoints.push(cp.upper_bound);
    }
    state.sync_labels();
}
