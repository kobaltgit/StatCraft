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

pub fn hex_to_color32(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()? * 17;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

pub fn color32_to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

pub fn color32_to_hsl(c: Color32) -> (f32, f32, f32) {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let l = (max + min) / 2.0;
    if delta.abs() < 1e-5 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let h = if (max - r).abs() < 1e-5 {
        ((g - b) / delta).rem_euclid(6.0) * 60.0
    } else if (max - g).abs() < 1e-5 {
        ((b - r) / delta + 2.0) * 60.0
    } else {
        ((r - g) / delta + 4.0) * 60.0
    };
    (h, s, l)
}

fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs().rem_euclid(360.0);
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Generates a high-contrast Color32 avoiding angular hue collisions (< 45 deg)
/// with any already open tabs/colors.
pub fn generate_contrast_color(existing_colors: &[Color32], seed_str: &str) -> Color32 {
    let mut h: u64 = 0xCBF29CE484222325;
    for &b in seed_str.as_bytes() {
        h = (h ^ (b as u64)).wrapping_mul(0x100000001B3);
    }
    let mut sm = h.wrapping_add(0x9E3779B97F4A7C15);
    sm = (sm ^ (sm >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    sm = (sm ^ (sm >> 27)).wrapping_mul(0x94D049BB133111EB);
    let hash = sm ^ (sm >> 31);

    let mut base_hue = ((hash % 360) as f32) + ((hash >> 16) % 1000) as f32 / 1000.0;

    let existing_hues: Vec<f32> = existing_colors
        .iter()
        .map(|&c| color32_to_hsl(c).0)
        .collect();

    const GOLDEN_ANGLE: f32 = 137.507_77;
    if !existing_hues.is_empty() {
        let mut best_hue = base_hue;
        let mut max_min_dist = -1.0_f32;
        for _ in 0..32 {
            let min_d = existing_hues
                .iter()
                .map(|&eh| hue_distance(base_hue, eh))
                .fold(f32::INFINITY, f32::min);

            if min_d >= 45.0 {
                best_hue = base_hue;
                break;
            }
            if min_d > max_min_dist {
                max_min_dist = min_d;
                best_hue = base_hue;
            }
            base_hue = (base_hue + GOLDEN_ANGLE).rem_euclid(360.0);
        }
        base_hue = best_hue;
    }

    let is_dark = is_dark_mode();
    let sat_offset = ((hash >> 32) % 200) as f32 / 1000.0;
    let saturation = 0.65 + sat_offset;
    let light_offset = ((hash >> 48) % 100) as f32 / 1000.0;
    let lightness = if is_dark {
        0.55 + light_offset
    } else {
        0.40 + light_offset
    };

    hsl_to_color32(base_hue, saturation, lightness)
}

/// Calculates a deterministic, aesthetically pleasing color for any dataset name
/// using SplitMix64 and the Golden Ratio HSL distribution.
pub fn dataset_color(name: &str) -> Color32 {
    let mut h: u64 = 0xCBF29CE484222325;
    for &b in name.as_bytes() {
        h = (h ^ (b as u64)).wrapping_mul(0x100000001B3);
    }
    let mut sm = h.wrapping_add(0x9E3779B97F4A7C15);
    sm = (sm ^ (sm >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    sm = (sm ^ (sm >> 27)).wrapping_mul(0x94D049BB133111EB);
    let hash = sm ^ (sm >> 31);

    let hue = (((hash as f64) * 137.507_764_050_037_85) % 360.0) as f32;
    let is_dark = is_dark_mode();
    let sat_offset = ((hash >> 32) % 150) as f32 / 1000.0;
    let saturation = 0.65 + sat_offset;
    let light_offset = ((hash >> 48) % 80) as f32 / 1000.0;
    let lightness = if is_dark {
        0.58 + light_offset
    } else {
        0.42 + light_offset
    };
    hsl_to_color32(hue, saturation, lightness)
}

#[inline]
pub fn color_with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn hsl_to_color32(h: f32, s: f32, l: f32) -> Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if (0.0..1.0).contains(&h_prime) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h_prime) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h_prime) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h_prime) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h_prime) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c / 2.0;
    let r = ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(r, g, b)
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

    #[test]
    fn test_dataset_color_deterministic() {
        let name = "sample_survey.csv";
        let color1 = dataset_color(name);
        for _ in 0..50 {
            let color2 = dataset_color(name);
            assert_eq!(color1, color2, "dataset_color must be 100% deterministic");
        }
    }

    #[test]
    fn test_dataset_color_differentiation() {
        let c1 = dataset_color("DataSet1");
        let c2 = dataset_color("DataSet2");
        let c3 = dataset_color("sample_survey.csv");
        assert_ne!(c1, c2, "Different datasets should have different colors");
        assert_ne!(c1, c3, "Different datasets should have different colors");
        assert_ne!(c2, c3, "Different datasets should have different colors");
    }

    #[test]
    fn test_color_with_alpha() {
        let base = Color32::from_rgb(100, 150, 200);
        let tinted = color_with_alpha(base, 42);
        assert_eq!(tinted.a(), 42);
        let [r, g, b, a] = tinted.to_srgba_unmultiplied();
        assert!((r as i32 - 100).abs() <= 2);
        assert!((g as i32 - 150).abs() <= 2);
        assert!((b as i32 - 200).abs() <= 2);
        assert_eq!(a, 42);
    }

    #[test]
    fn test_hex_conversion_roundtrip() {
        let hex = "#2E86AB";
        let c = hex_to_color32(hex).unwrap();
        let back = color32_to_hex(c);
        assert_eq!(back, hex);

        let short_hex = "#FFF";
        let c_white = hex_to_color32(short_hex).unwrap();
        assert_eq!(c_white, Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn test_generate_contrast_color_distance() {
        let files = [
            "merge_base.csv",
            "merge_add_cases.csv",
            "merge_add_vars.csv",
        ];
        let mut chosen: Vec<Color32> = Vec::new();

        for file in files {
            let color = generate_contrast_color(&chosen, file);
            chosen.push(color);
        }

        assert_eq!(chosen.len(), 3);
        assert_ne!(chosen[0], chosen[1]);
        assert_ne!(chosen[1], chosen[2]);
        assert_ne!(chosen[0], chosen[2]);

        let h0 = color32_to_hsl(chosen[0]).0;
        let h1 = color32_to_hsl(chosen[1]).0;
        let h2 = color32_to_hsl(chosen[2]).0;

        let d01 = hue_distance(h0, h1);
        let d12 = hue_distance(h1, h2);
        let d02 = hue_distance(h0, h2);

        assert!(
            d01 >= 45.0,
            "Hues h0 ({h0}) and h1 ({h1}) distance must be >= 45 deg, got {d01}"
        );
        assert!(
            d12 >= 45.0,
            "Hues h1 ({h1}) and h2 ({h2}) distance must be >= 45 deg, got {d12}"
        );
        assert!(
            d02 >= 45.0,
            "Hues h0 ({h0}) and h2 ({h2}) distance must be >= 45 deg, got {d02}"
        );
    }
}
