use std::env;
use std::path::PathBuf;

mod ci;
mod design_guard;
mod file_guard;
mod icon;
mod layers;
mod parity;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    match command {
        "ci" => {
            if let Err(e) = ci::run_ci(&workspace_root) {
                eprintln!("\n[CI GATE FAILED] {}", e);
                std::process::exit(1);
            }
        }
        "icon" => {
            if let Err(e) = icon::generate_icons(&workspace_root) {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
        "layers" => {
            if let Err(e) = layers::check_layers(&workspace_root) {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
        "parity" => {
            if let Err(e) = parity::check_parity(&workspace_root) {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
        "file-guard" => {
            if let Err(e) = file_guard::check_file_sizes(&workspace_root) {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
        "design-guard" => {
            if let Err(e) = design_guard::check_dialog_design(&workspace_root) {
                eprintln!("[ERROR] {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            println!("StatCraft xtask utility");
            println!("Usage:");
            println!("  cargo xtask ci           Run all CI quality gates (fmt, clippy, layers, guards, tests)");
            println!("  cargo xtask design-guard Check DialogKit design standards on all dialogs");
            println!(
                "  cargo xtask file-guard   Check source file size limits (500/1000/1500 lines)"
            );
            println!(
                "  cargo xtask icon       Generate assets/icon.ico and PNGs from assets/icon.svg"
            );
            println!("  cargo xtask layers     Check L0..L6 architectural layering rules");
            println!("  cargo xtask parity     Check and update SPSS feature parity checklist");
        }
    }
}
