use super::matrix::format_layer_banner;
use crate::theme::Tokens;
use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};
use statcraft_algo::CrosstabResult;
use statcraft_i18n::t;

const CLUSTER_PALETTE: [Color32; 8] = [
    Color32::from_rgb(0x4F, 0x81, 0xBD),
    Color32::from_rgb(0xC0, 0x50, 0x4D),
    Color32::from_rgb(0x9B, 0xBB, 0x59),
    Color32::from_rgb(0x80, 0x64, 0xA2),
    Color32::from_rgb(0x4B, 0xAC, 0xC6),
    Color32::from_rgb(0xF7, 0x96, 0x46),
    Color32::from_rgb(0x71, 0x84, 0x99),
    Color32::from_rgb(0xD9, 0x7A, 0x53),
];

pub fn render_clustered_bar_chart(ui: &mut Ui, res: &CrosstabResult, selected: bool) {
    let base_title = t!(
        "output-crosstabs-chart-title",
        row = res.row_var.as_str(),
        col = res.col_var.as_str()
    );
    let title = if let Some(banner) = format_layer_banner(&res.layer_keys) {
        format!("{} ({})", base_title, banner)
    } else {
        base_title
    };

    ui.label(
        egui::RichText::new(title)
            .font(FontId::proportional(12.5))
            .strong()
            .color(if selected {
                Tokens::accent()
            } else {
                Tokens::text_primary()
            }),
    );
    ui.add_space(4.0);

    let chart_width = 580.0_f32;
    let chart_height = 290.0_f32;
    let (rect, _) = ui.allocate_exact_size(vec2(chart_width, chart_height), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);

    painter.rect_filled(rect, 3.0_f32, Tokens::bg_panel());
    painter.rect_stroke(rect, 3.0_f32, Stroke::new(1.0_f32, Tokens::border()));

    if res.rows.is_empty() || res.col_categories.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            t!("chart-no-data"),
            FontId::proportional(12.0),
            Tokens::text_muted(),
        );
        return;
    }

    let left_m = 56.0_f32;
    let right_m = 130.0_f32;
    let top_m = 24.0_f32;
    let bottom_m = 44.0_f32;

    let plot_rect = Rect::from_min_max(
        pos2(rect.min.x + left_m, rect.min.y + top_m),
        pos2(rect.max.x - right_m, rect.max.y - bottom_m),
    );

    let mut max_count = 0.0_f64;
    for r in &res.rows {
        for c in &r.cells {
            if (c.observed as f64) > max_count {
                max_count = c.observed as f64;
            }
        }
    }
    let nice_max = nice_axis_max(max_count.max(1.0));

    // Y gridlines and labels
    for i in 0..=4 {
        let frac = i as f32 / 4.0;
        let y = plot_rect.max.y - frac * plot_rect.height();
        let val = nice_max * (frac as f64);
        painter.line_segment(
            [pos2(plot_rect.min.x, y), pos2(plot_rect.max.x, y)],
            Stroke::new(0.5_f32, Tokens::border()),
        );
        painter.text(
            pos2(plot_rect.min.x - 6.0_f32, y),
            Align2::RIGHT_CENTER,
            format!("{:.0}", val),
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );
    }

    // Axes
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.min.y),
            pos2(plot_rect.min.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::text_muted()),
    );
    painter.line_segment(
        [
            pos2(plot_rect.min.x, plot_rect.max.y),
            pos2(plot_rect.max.x, plot_rect.max.y),
        ],
        Stroke::new(1.0_f32, Tokens::text_muted()),
    );

    let n_groups = res.rows.len();
    let n_bars = res.col_categories.len();
    let group_w = plot_rect.width() / n_groups as f32;
    let inner_w = group_w * 0.76_f32;
    let bar_w = (inner_w / n_bars as f32).max(4.0_f32);

    for (g_idx, row) in res.rows.iter().enumerate() {
        let group_left = plot_rect.min.x + g_idx as f32 * group_w + (group_w - inner_w) * 0.5_f32;
        for (b_idx, cell) in row.cells.iter().enumerate() {
            let h_frac = (cell.observed as f64 / nice_max) as f32;
            let bar_h = (h_frac * plot_rect.height()).max(0.0_f32);
            let x0 = group_left + b_idx as f32 * bar_w;
            let bar_rect = Rect::from_min_max(
                pos2(x0 + 1.0_f32, plot_rect.max.y - bar_h),
                pos2(x0 + bar_w - 1.0_f32, plot_rect.max.y),
            );
            let color = CLUSTER_PALETTE[b_idx % CLUSTER_PALETTE.len()];
            painter.rect_filled(bar_rect, 1.0_f32, color);
        }

        let r_label = row
            .category
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| row.category.value.clone());
        painter.text(
            pos2(
                plot_rect.min.x + (g_idx as f32 + 0.5_f32) * group_w,
                plot_rect.max.y + 12.0_f32,
            ),
            Align2::CENTER_CENTER,
            r_label,
            FontId::proportional(10.5),
            Tokens::text_primary(),
        );
    }

    // Legend on the right
    let legend_x = plot_rect.max.x + 12.0_f32;
    let mut legend_y = plot_rect.min.y + 4.0_f32;
    painter.text(
        pos2(legend_x, legend_y),
        Align2::LEFT_TOP,
        res.col_var.as_str(),
        FontId::proportional(10.5),
        Tokens::text_primary(),
    );
    legend_y += 16.0_f32;

    for (b_idx, col_cat) in res.col_categories.iter().enumerate() {
        if legend_y > plot_rect.max.y - 10.0_f32 {
            break;
        }
        let color = CLUSTER_PALETTE[b_idx % CLUSTER_PALETTE.len()];
        let swatch = Rect::from_min_size(pos2(legend_x, legend_y + 2.0_f32), vec2(10.0, 10.0));
        painter.rect_filled(swatch, 1.0_f32, color);
        let c_label = col_cat
            .label
            .as_ref()
            .cloned()
            .unwrap_or_else(|| col_cat.value.clone());
        painter.text(
            pos2(legend_x + 14.0_f32, legend_y),
            Align2::LEFT_TOP,
            c_label,
            FontId::proportional(10.0),
            Tokens::text_muted(),
        );
        legend_y += 16.0_f32;
    }
}

fn nice_axis_max(raw: f64) -> f64 {
    let exp = raw.log10().floor();
    let base = 10.0_f64.powf(exp);
    let frac = raw / base;
    let nice_frac = if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice_frac * base
}
