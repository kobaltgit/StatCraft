use super::super::tables_common::{compute_xy_bounds, render_xy_scatter_plot};
use crate::theme::Tokens;
use egui::Ui;
use statcraft_algo::DemingRegressionResult;
use statcraft_i18n::t;

pub(super) fn render_scatter_plot(
    ui: &mut Ui,
    res: &DemingRegressionResult,
    dep_display: &str,
    indep_display: &str,
    include_identity: bool,
) {
    ui.strong(t!("output-deming-scatter-title"));
    ui.add_space(4.0);

    let pts: Vec<(f64, f64)> = res.points.iter().map(|p| (p.x, p.y)).collect();
    let (x_min, x_max, y_min, y_max) = compute_xy_bounds(&pts);
    let line_y0 = res.slope * x_min + res.intercept;
    let line_y1 = res.slope * x_max + res.intercept;
    let y_lo = y_min.min(line_y0).min(line_y1);
    let y_hi = y_max.max(line_y0).max(line_y1);

    render_xy_scatter_plot(
        ui,
        &pts,
        (x_min, x_max),
        (y_lo, y_hi),
        Some(((x_min, line_y0), (x_max, line_y1))),
        include_identity,
        indep_display,
        dep_display,
        Tokens::accent(),
    );
}

pub(super) fn render_residual_plot(ui: &mut Ui, res: &DemingRegressionResult) {
    ui.strong(t!("output-deming-residual-title"));
    ui.add_space(4.0);

    let pts: Vec<(f64, f64)> = res
        .points
        .iter()
        .map(|p| (p.fitted, p.orthogonal_residual))
        .collect();
    let (x_min, x_max, y_min, y_max) = compute_xy_bounds(&pts);
    let y_lo = y_min.min(0.0);
    let y_hi = y_max.max(0.0);

    render_xy_scatter_plot(
        ui,
        &pts,
        (x_min, x_max),
        (y_lo, y_hi),
        Some(((x_min, 0.0), (x_max, 0.0))),
        false,
        &t!("output-deming-axis-fitted"),
        &t!("output-deming-axis-residual"),
        Tokens::text_muted(),
    );
}
