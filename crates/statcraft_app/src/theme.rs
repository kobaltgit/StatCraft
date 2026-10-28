use egui::Color32;

pub struct Tokens;

impl Tokens {
    pub const BG_APP: Color32 = Color32::from_rgb(24, 25, 28);
    pub const BG_PANEL: Color32 = Color32::from_rgb(32, 33, 37);
    pub const BG_HEADER: Color32 = Color32::from_rgb(40, 42, 48);
    pub const BG_ROW_EVEN: Color32 = Color32::from_rgb(30, 31, 35);
    pub const BG_ROW_ODD: Color32 = Color32::from_rgb(26, 27, 30);
    pub const BG_SELECTED: Color32 = Color32::from_rgb(45, 65, 100);
    pub const ACCENT: Color32 = Color32::from_rgb(50, 100, 180);

    pub const BORDER: Color32 = Color32::from_rgb(55, 58, 66);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 232, 238);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 145, 155);

    // Variable Measure Colors (Vector UI)
    pub const MEASURE_SCALE: Color32 = Color32::from_rgb(70, 150, 230); // Blue (ruler)
    pub const MEASURE_ORDINAL: Color32 = Color32::from_rgb(230, 160, 40); // Amber (bars)
    pub const MEASURE_NOMINAL: Color32 = Color32::from_rgb(70, 190, 120); // Green (circles)

    // Window Caption Tokens (Custom Titlebar)
    pub const CAPTION_CLOSE_HOVER: Color32 = Color32::from_rgb(232, 17, 35);
    pub const CAPTION_CLOSE_PRESSED: Color32 = Color32::from_rgb(241, 112, 122);
    pub const CAPTION_HOVER: Color32 = Color32::from_rgb(52, 54, 60);
    pub const CAPTION_PRESSED: Color32 = Color32::from_rgb(64, 67, 75);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_tokens_contrast() {
        assert_ne!(Tokens::BG_APP, Tokens::TEXT_PRIMARY);
        assert_ne!(Tokens::MEASURE_SCALE, Tokens::MEASURE_NOMINAL);
    }
}
