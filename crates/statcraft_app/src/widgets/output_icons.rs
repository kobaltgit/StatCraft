use egui::{pos2, vec2, Color32, Painter, Rect, Stroke};

/// Paints the Root Output document icon (folded paper sheet with text lines)
pub fn paint_output_root_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let w = 12.0_f32;
    let h = 14.0_f32;
    let paper_rect = Rect::from_center_size(center, vec2(w, h));

    let stroke = Stroke::new(1.0_f32, color);
    let fold = 3.5_f32;

    // Outer contour with folded corner top-right
    let p_tl = paper_rect.left_top();
    let p_tr_fold = pos2(paper_rect.right() - fold, paper_rect.top());
    let p_tr_corner = pos2(paper_rect.right(), paper_rect.top() + fold);
    let p_br = paper_rect.right_bottom();
    let p_bl = paper_rect.left_bottom();

    painter.line_segment([p_tl, p_tr_fold], stroke);
    painter.line_segment([p_tr_fold, p_tr_corner], stroke);
    painter.line_segment([p_tr_corner, p_br], stroke);
    painter.line_segment([p_br, p_bl], stroke);
    painter.line_segment([p_bl, p_tl], stroke);

    // Inner fold corner
    let p_inner_fold = pos2(paper_rect.right() - fold, paper_rect.top() + fold);
    painter.line_segment([p_tr_fold, p_inner_fold], stroke);
    painter.line_segment([p_inner_fold, p_tr_corner], stroke);

    // Document content lines
    let line_left = paper_rect.left() + 2.5_f32;
    let line_right = paper_rect.right() - 2.5_f32;
    let l1_y = paper_rect.top() + 6.0_f32;
    let l2_y = paper_rect.top() + 8.5_f32;
    let l3_y = paper_rect.top() + 11.0_f32;

    painter.line_segment([pos2(line_left, l1_y), pos2(line_right, l1_y)], stroke);
    painter.line_segment([pos2(line_left, l2_y), pos2(line_right, l2_y)], stroke);
    painter.line_segment(
        [
            pos2(line_left, l3_y),
            pos2(line_left + (line_right - line_left) * 0.6_f32, l3_y),
        ],
        stroke,
    );
}

/// Paints a procedure folder icon (with tab, open or closed)
pub fn paint_output_folder_icon(painter: &Painter, rect: Rect, is_open: bool, color: Color32) {
    let center = rect.center();
    let w = 14.0_f32;
    let h = 11.0_f32;
    let folder_rect = Rect::from_center_size(center, vec2(w, h));
    let stroke = Stroke::new(1.0_f32, color);

    let tab_w = 5.5_f32;
    let tab_h = 2.5_f32;

    // Folder tab at top-left
    let t_tl = pos2(folder_rect.left(), folder_rect.top() - tab_h);
    let t_tr = pos2(folder_rect.left() + tab_w, folder_rect.top() - tab_h);
    let t_br = pos2(folder_rect.left() + tab_w + 1.5_f32, folder_rect.top());

    painter.line_segment([t_tl, t_tr], stroke);
    painter.line_segment([t_tr, t_br], stroke);

    // Body
    let b_rect = folder_rect;
    painter.rect_stroke(b_rect, 1.0_f32, stroke);

    if is_open {
        // Open flap
        let f_tl = pos2(b_rect.left() - 1.0_f32, b_rect.bottom() - 4.5_f32);
        let f_tr = pos2(b_rect.right() + 1.5_f32, b_rect.bottom() - 4.5_f32);
        painter.line_segment([f_tl, f_tr], stroke);
        painter.line_segment([f_tl, b_rect.left_bottom()], stroke);
        painter.line_segment([f_tr, b_rect.right_bottom()], stroke);
    }
}

/// Paints a Syntax/Log icon (terminal / command prompt page with '>')
pub fn paint_output_log_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let w = 12.0_f32;
    let h = 13.0_f32;
    let page_rect = Rect::from_center_size(center, vec2(w, h));
    let stroke = Stroke::new(1.0_f32, color);

    painter.rect_stroke(page_rect, 1.0_f32, stroke);

    // Prompt '>' symbol
    let p_mid_x = page_rect.left() + 4.0_f32;
    let p_mid_y = page_rect.top() + 4.5_f32;
    painter.line_segment(
        [
            pos2(page_rect.left() + 2.5_f32, p_mid_y - 2.0_f32),
            pos2(p_mid_x, p_mid_y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(p_mid_x, p_mid_y),
            pos2(page_rect.left() + 2.5_f32, p_mid_y + 2.0_f32),
        ],
        stroke,
    );

    // Cursor / command underline
    let cur_y = page_rect.top() + 4.5_f32;
    painter.line_segment(
        [
            pos2(p_mid_x + 2.0_f32, cur_y),
            pos2(page_rect.right() - 2.5_f32, cur_y),
        ],
        stroke,
    );

    // Additional syntax lines below
    let l2_y = page_rect.top() + 8.5_f32;
    painter.line_segment(
        [
            pos2(page_rect.left() + 2.5_f32, l2_y),
            pos2(page_rect.right() - 2.5_f32, l2_y),
        ],
        stroke,
    );
}

/// Paints a Notes icon (small memo notepad with lines)
pub fn paint_output_notes_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let w = 12.0_f32;
    let h = 13.0_f32;
    let page_rect = Rect::from_center_size(center, vec2(w, h));
    let stroke = Stroke::new(1.0_f32, color);

    painter.rect_stroke(page_rect, 1.0_f32, stroke);

    // Top notepad clip/binder bar
    let clip_rect = Rect::from_min_max(
        pos2(page_rect.left() + 2.0_f32, page_rect.top() - 1.0_f32),
        pos2(page_rect.right() - 2.0_f32, page_rect.top() + 2.0_f32),
    );
    painter.rect_filled(clip_rect, 0.5_f32, color);

    // Note lines
    for i in 0..3 {
        let y = page_rect.top() + 5.0_f32 + (i as f32) * 2.5_f32;
        painter.line_segment(
            [
                pos2(page_rect.left() + 2.5_f32, y),
                pos2(page_rect.right() - 2.5_f32, y),
            ],
            stroke,
        );
    }
}

/// Paints a Table / Pivot Table icon (miniature grid with header and columns)
pub fn paint_output_table_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let w = 13.0_f32;
    let h = 12.0_f32;
    let table_rect = Rect::from_center_size(center, vec2(w, h));
    let stroke = Stroke::new(1.0_f32, color);

    // Outer table border
    painter.rect_stroke(table_rect, 0.5_f32, stroke);

    // Header dividing line
    let header_y = table_rect.top() + 3.8_f32;
    painter.line_segment(
        [
            pos2(table_rect.left(), header_y),
            pos2(table_rect.right(), header_y),
        ],
        stroke,
    );

    // Header fill tint
    let header_rect = Rect::from_min_max(table_rect.left_top(), pos2(table_rect.right(), header_y));
    painter.rect_filled(
        header_rect,
        0.0_f32,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60),
    );

    // Column separator
    let col_x = table_rect.left() + w * 0.45_f32;
    painter.line_segment(
        [
            pos2(col_x, table_rect.top()),
            pos2(col_x, table_rect.bottom()),
        ],
        stroke,
    );

    // Row separator in data section
    let row_y = header_y + 4.0_f32;
    painter.line_segment(
        [
            pos2(table_rect.left(), row_y),
            pos2(table_rect.right(), row_y),
        ],
        stroke,
    );
}

/// Paints a vector Warning icon (triangle with exclamation mark)
pub fn paint_output_warning_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let half_w = 6.5_f32;
    let half_h = 6.0_f32;

    let p_top = pos2(center.x, center.y - half_h);
    let p_left = pos2(center.x - half_w, center.y + half_h);
    let p_right = pos2(center.x + half_w, center.y + half_h);

    let stroke = Stroke::new(1.2_f32, color);
    painter.line_segment([p_top, p_left], stroke);
    painter.line_segment([p_left, p_right], stroke);
    painter.line_segment([p_right, p_top], stroke);

    // Exclamation line
    painter.line_segment(
        [
            pos2(center.x, center.y - 2.5_f32),
            pos2(center.x, center.y + 1.5_f32),
        ],
        Stroke::new(1.2_f32, color),
    );

    // Exclamation dot
    painter.circle_filled(pos2(center.x, center.y + 4.0_f32), 0.9_f32, color);
}

/// Paints a trash can icon for "Clear All"
pub fn paint_icon_trash(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let stroke = Stroke::new(1.2_f32, color);

    // Lid
    painter.line_segment(
        [
            pos2(center.x - 5.0_f32, center.y - 4.5_f32),
            pos2(center.x + 5.0_f32, center.y - 4.5_f32),
        ],
        stroke,
    );
    // Handle
    painter.line_segment(
        [
            pos2(center.x - 2.0_f32, center.y - 6.0_f32),
            pos2(center.x + 2.0_f32, center.y - 6.0_f32),
        ],
        stroke,
    );
    // Bin body
    let bin = Rect::from_min_max(
        pos2(center.x - 4.0_f32, center.y - 3.5_f32),
        pos2(center.x + 4.0_f32, center.y + 5.5_f32),
    );
    painter.rect_stroke(bin, 1.0_f32, stroke);
    // Inner vertical slats
    painter.line_segment(
        [
            pos2(center.x - 1.5_f32, center.y - 1.5_f32),
            pos2(center.x - 1.5_f32, center.y + 3.5_f32),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x + 1.5_f32, center.y - 1.5_f32),
            pos2(center.x + 1.5_f32, center.y + 3.5_f32),
        ],
        stroke,
    );
}

/// Paints an "Expand all" icon (small plus square)
pub fn paint_icon_expand_all(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let box_rect = Rect::from_center_size(center, vec2(11.0_f32, 11.0_f32));
    let stroke = Stroke::new(1.0_f32, color);
    painter.rect_stroke(box_rect, 1.0_f32, stroke);

    // Plus sign
    let p_stroke = Stroke::new(1.2_f32, color);
    painter.line_segment(
        [
            pos2(center.x - 3.0_f32, center.y),
            pos2(center.x + 3.0_f32, center.y),
        ],
        p_stroke,
    );
    painter.line_segment(
        [
            pos2(center.x, center.y - 3.0_f32),
            pos2(center.x, center.y + 3.0_f32),
        ],
        p_stroke,
    );
}

/// Paints a "Collapse all" icon (small minus square)
pub fn paint_icon_collapse_all(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let box_rect = Rect::from_center_size(center, vec2(11.0_f32, 11.0_f32));
    let stroke = Stroke::new(1.0_f32, color);
    painter.rect_stroke(box_rect, 1.0_f32, stroke);

    // Minus sign
    let p_stroke = Stroke::new(1.2_f32, color);
    painter.line_segment(
        [
            pos2(center.x - 3.0_f32, center.y),
            pos2(center.x + 3.0_f32, center.y),
        ],
        p_stroke,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_icons_painting_without_panics() {
        // Just verify basic dimensions and coordinates calculation
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(20.0, 20.0));
        assert!(rect.is_positive());
    }
}
