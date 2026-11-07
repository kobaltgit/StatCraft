use egui::Color32;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub bg_app: Color32,
    pub bg_panel: Color32,
    pub bg_header: Color32,
    pub bg_row_even: Color32,
    pub bg_row_odd: Color32,
    pub bg_selected: Color32,
    pub accent: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub measure_scale: Color32,
    pub measure_ordinal: Color32,
    pub measure_nominal: Color32,
    pub caption_close_hover: Color32,
    pub caption_close_pressed: Color32,
    pub caption_hover: Color32,
    pub caption_pressed: Color32,
}

pub static DARK_PALETTE: ThemePalette = ThemePalette {
    bg_app: Color32::from_rgb(24, 25, 28),
    bg_panel: Color32::from_rgb(32, 33, 37),
    bg_header: Color32::from_rgb(40, 42, 48),
    bg_row_even: Color32::from_rgb(30, 31, 35),
    bg_row_odd: Color32::from_rgb(26, 27, 30),
    bg_selected: Color32::from_rgb(45, 65, 100),
    accent: Color32::from_rgb(50, 100, 180),
    border: Color32::from_rgb(55, 58, 66),
    text_primary: Color32::from_rgb(230, 232, 238),
    text_muted: Color32::from_rgb(140, 145, 155),
    measure_scale: Color32::from_rgb(70, 150, 230),
    measure_ordinal: Color32::from_rgb(230, 160, 40),
    measure_nominal: Color32::from_rgb(70, 190, 120),
    caption_close_hover: Color32::from_rgb(232, 17, 35),
    caption_close_pressed: Color32::from_rgb(241, 112, 122),
    caption_hover: Color32::from_rgb(52, 54, 60),
    caption_pressed: Color32::from_rgb(64, 67, 75),
};

pub static LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg_app: Color32::from_rgb(255, 255, 255),
    bg_panel: Color32::from_rgb(247, 248, 250),
    bg_header: Color32::from_rgb(233, 236, 239),
    bg_row_even: Color32::from_rgb(255, 255, 255),
    bg_row_odd: Color32::from_rgb(248, 249, 250),
    bg_selected: Color32::from_rgb(221, 244, 255),
    accent: Color32::from_rgb(9, 105, 218),
    border: Color32::from_rgb(208, 215, 222),
    text_primary: Color32::from_rgb(31, 35, 40),
    text_muted: Color32::from_rgb(101, 109, 118),
    measure_scale: Color32::from_rgb(70, 150, 230),
    measure_ordinal: Color32::from_rgb(217, 119, 6),
    measure_nominal: Color32::from_rgb(22, 163, 74),
    caption_close_hover: Color32::from_rgb(232, 17, 35),
    caption_close_pressed: Color32::from_rgb(241, 112, 122),
    caption_hover: Color32::from_rgb(230, 232, 236),
    caption_pressed: Color32::from_rgb(215, 218, 224),
};

static IS_DARK_MODE: AtomicBool = AtomicBool::new(true);

pub fn set_dark_mode(dark: bool) {
    IS_DARK_MODE.store(dark, Ordering::Relaxed);
}

pub fn is_dark_mode() -> bool {
    IS_DARK_MODE.load(Ordering::Relaxed)
}

pub fn palette() -> &'static ThemePalette {
    if is_dark_mode() {
        &DARK_PALETTE
    } else {
        &LIGHT_PALETTE
    }
}

pub struct Tokens;

impl Tokens {
    #[inline]
    pub fn bg_app() -> Color32 {
        palette().bg_app
    }

    #[inline]
    pub fn bg_panel() -> Color32 {
        palette().bg_panel
    }

    #[inline]
    pub fn bg_header() -> Color32 {
        palette().bg_header
    }

    #[inline]
    pub fn bg_row_even() -> Color32 {
        palette().bg_row_even
    }

    #[inline]
    pub fn bg_row_odd() -> Color32 {
        palette().bg_row_odd
    }

    #[inline]
    pub fn bg_selected() -> Color32 {
        palette().bg_selected
    }

    #[inline]
    pub fn accent() -> Color32 {
        palette().accent
    }

    #[inline]
    pub fn border() -> Color32 {
        palette().border
    }

    #[inline]
    pub fn text_primary() -> Color32 {
        palette().text_primary
    }

    #[inline]
    pub fn text_muted() -> Color32 {
        palette().text_muted
    }

    #[inline]
    pub fn measure_scale() -> Color32 {
        palette().measure_scale
    }

    #[inline]
    pub fn measure_ordinal() -> Color32 {
        palette().measure_ordinal
    }

    #[inline]
    pub fn measure_nominal() -> Color32 {
        palette().measure_nominal
    }

    #[inline]
    pub fn caption_close_hover() -> Color32 {
        palette().caption_close_hover
    }

    #[inline]
    pub fn caption_close_pressed() -> Color32 {
        palette().caption_close_pressed
    }

    #[inline]
    pub fn caption_hover() -> Color32 {
        palette().caption_hover
    }

    #[inline]
    pub fn caption_pressed() -> Color32 {
        palette().caption_pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_luminance(c: Color32) -> f64 {
        fn channel(v: u8) -> f64 {
            let s = (v as f64) / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }

    fn contrast_ratio(c1: Color32, c2: Color32) -> f64 {
        let l1 = relative_luminance(c1);
        let l2 = relative_luminance(c2);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn test_theme_mode_switching() {
        set_dark_mode(true);
        assert!(is_dark_mode());
        assert_eq!(Tokens::bg_app(), DARK_PALETTE.bg_app);

        set_dark_mode(false);
        assert!(!is_dark_mode());
        assert_eq!(Tokens::bg_app(), LIGHT_PALETTE.bg_app);

        // Reset to dark for subsequent tests
        set_dark_mode(true);
    }

    #[test]
    fn test_wcag_contrast() {
        // Light theme contrast
        let light_bg = LIGHT_PALETTE.bg_app;
        let light_text = LIGHT_PALETTE.text_primary;
        let light_muted = LIGHT_PALETTE.text_muted;

        let ratio_primary = contrast_ratio(light_bg, light_text);
        assert!(
            ratio_primary >= 7.0,
            "Light primary text contrast must exceed AAA (7.0): got {ratio_primary}"
        );

        let ratio_muted = contrast_ratio(light_bg, light_muted);
        assert!(
            ratio_muted >= 4.5,
            "Light muted text contrast must exceed AA (4.5): got {ratio_muted}"
        );

        // Dark theme contrast
        let dark_bg = DARK_PALETTE.bg_app;
        let dark_text = DARK_PALETTE.text_primary;
        let dark_muted = DARK_PALETTE.text_muted;

        let ratio_dark_primary = contrast_ratio(dark_bg, dark_text);
        assert!(
            ratio_dark_primary >= 7.0,
            "Dark primary text contrast must exceed AAA (7.0): got {ratio_dark_primary}"
        );

        let ratio_dark_muted = contrast_ratio(dark_bg, dark_muted);
        assert!(
            ratio_dark_muted >= 4.5,
            "Dark muted text contrast must exceed AA (4.5): got {ratio_dark_muted}"
        );
    }
}
