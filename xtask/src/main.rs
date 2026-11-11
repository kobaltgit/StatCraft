use std::env;
use std::path::PathBuf;

mod bench_stress;
mod ci;
mod corpus;
mod design_guard;
mod file_guard;
mod hooks;
mod icon;
mod layers;
mod parity;
mod undo_matrix;

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
        "test-corpus" => {
            let custom_dir = args.get(2).map(PathBuf::from);
            if let Err(e) = corpus::run_test_corpus(&workspace_root, custom_dir.as_deref()) {
                eprintln!("\n[TEST-CORPUS FAILED] {}", e);
                std::process::exit(1);
            }
        }
        "test-undo" => {
            if let Err(e) = undo_matrix::run_undo_matrix(&workspace_root) {
                eprintln!("\n[TEST-UNDO FAILED] {}", e);
                std::process::exit(1);
            }
        }
        "bench-stress" => {
            if let Err(e) = bench_stress::run_bench_stress(&workspace_root) {
                eprintln!("\n[BENCH-STRESS FAILED] {}", e);
                std::process::exit(1);
            }
        }
        "install-hooks" => {
            if let Err(e) = hooks::install_hooks(&workspace_root) {
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
            println!(
                "  cargo xtask install-hooks Install Git pre-push hook for automatic quality gates"
            );
            println!("  cargo xtask layers     Check L0..L6 architectural layering rules");
            println!("  cargo xtask parity     Check and update SPSS feature parity checklist");
            println!("  cargo xtask test-corpus  Run real-world corpora and codec fuzzing audit");
            println!("  cargo xtask test-undo    Run exhaustive Undo/Redo reversibility matrix across all 45 commands");
            println!("  cargo xtask bench-stress Run extreme stress test (100k/1M rows, 21 procs, UI FPS, memory probe)");
        }
    }
}
