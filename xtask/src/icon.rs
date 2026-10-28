use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::fs;
use std::path::Path;

pub fn generate_icons(workspace_root: &Path) -> Result<(), String> {
    let svg_path = workspace_root.join("assets").join("icon.svg");
    if !svg_path.exists() {
        return Err("assets/icon.svg not found".to_string());
    }

    let svg_data = fs::read_to_string(&svg_path).map_err(|e| e.to_string())?;
    let opt = Options::default();
    let tree = Tree::from_str(&svg_data, &opt).map_err(|e| format!("{:?}", e))?;

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    let sizes = [16, 32, 48, 64, 128, 256];
    for &size in &sizes {
        let mut pixmap = Pixmap::new(size, size).ok_or("Cannot allocate pixmap")?;
        let sx = size as f32 / tree.size().width();
        let sy = size as f32 / tree.size().height();
        let transform = Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let png_bytes = pixmap.encode_png().map_err(|e| e.to_string())?;

        let out_png = workspace_root
            .join("assets")
            .join(format!("icon-{}.png", size));
        fs::write(&out_png, &png_bytes).map_err(|e| e.to_string())?;

        let icon_image = ico::IconImage::read_png(&png_bytes[..]).map_err(|e| e.to_string())?;
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image).map_err(|e| e.to_string())?);
    }

    let ico_path = workspace_root.join("assets").join("icon.ico");
    let mut file = fs::File::create(&ico_path).map_err(|e| e.to_string())?;
    icon_dir.write(&mut file).map_err(|e| e.to_string())?;

    println!("[xtask icon] Successfully generated assets/icon.ico and PNGs (16..256px)");
    Ok(())
}
