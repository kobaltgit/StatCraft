//! SVG Icon management and rendering for StatCraft.
//! Icons are embedded at compile-time, recolored to white, and rasterized by egui_extras
//! at the exact screen DPI, allowing dynamic tinting via `egui::Image::tint`.

use egui::{load::Bytes, Color32, Image, ImageSource, Vec2};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::icon_data::ICONS;

pub struct SvgIconSet {
    icons: HashMap<&'static str, Arc<[u8]>>,
}

impl SvgIconSet {
    pub fn new(icons: &'static [(&'static str, &'static [u8])]) -> Self {
        let mut map = HashMap::new();
        for &(name, bytes) in icons {
            let svg_str = String::from_utf8_lossy(bytes)
                .replace("currentColor", "#ffffff")
                .replace("stroke-width=\"2\"", "stroke-width=\"1.75\"");
            map.insert(name, Arc::from(svg_str.into_bytes()));
        }
        Self { icons: map }
    }

    pub fn source(&self, name: &str) -> ImageSource<'static> {
        let bytes = self.icons.get(name).cloned().unwrap_or_default();
        ImageSource::Bytes {
            uri: format!("bytes://statcraft/icons/{name}.svg").into(),
            bytes: Bytes::Shared(bytes),
        }
    }
}

pub fn icon_image(name: &str, size: f32, tint: Color32) -> Image<'static> {
    static SET: OnceLock<SvgIconSet> = OnceLock::new();
    let set = SET.get_or_init(|| SvgIconSet::new(ICONS));
    Image::new(set.source(name))
        .fit_to_exact_size(Vec2::splat(size))
        .tint(tint)
}
