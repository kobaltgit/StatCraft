use std::path::Path;
use std::process::Command;

fn run_command(cmd: &mut Command, desc: &str) -> Result<(), String> {
    println!("\n=== [CI Gate] Running: {} ===", desc);
    let status = cmd
        .status()
        .map_err(|e| format!("{}: failed to spawn: {}", desc, e))?;
    if status.success() {
        println!(">>> [OK] {}", desc);
        Ok(())
    } else {
        Err(format!(
            "{} failed with exit code: {:?}",
            desc,
            status.code()
        ))
    }
}

pub fn run_ci(root: &Path) -> Result<(), String> {
    println!("StatCraft Quality Gate (ArtCraft standard)\n");

    // 1. Rustfmt check
    let mut fmt_cmd = Command::new("cargo");
    fmt_cmd
        .current_dir(root)
        .args(["fmt", "--all", "--", "--check"]);
    run_command(&mut fmt_cmd, "cargo fmt --all -- --check")?;

    // 2. Clippy with -D warnings and -j 1
    let mut clippy_cmd = Command::new("cargo");
    clippy_cmd.current_dir(root).args([
        "clippy",
        "--workspace",
        "--all-targets",
        "-j",
        "1",
        "--",
        "-D",
        "warnings",
    ]);
    run_command(
        &mut clippy_cmd,
        "cargo clippy --workspace --all-targets -j 1 -- -D warnings",
    )?;

    // 3. Architecture layers check (L0-L6)
    println!("\n=== [CI Gate] Architecture Layering (xtask layers) ===");
    crate::layers::check_layers(root)?;
    println!(">>> [OK] Architecture layers");

    // 4. File size guard (500/1000/1500 limits)
    println!("\n=== [CI Gate] File Size Guard (xtask file-guard) ===");
    crate::file_guard::check_file_sizes(root)?;
    println!(">>> [OK] File size limits");

    // 5. Design guard (DialogKit standards)
    println!("\n=== [CI Gate] Design Guard (xtask design-guard) ===");
    crate::design_guard::check_dialog_design(root)?;
    println!(">>> [OK] Design system standards");

    // 6. Full workspace test suite with -j 1
    let mut test_cmd = Command::new("cargo");
    test_cmd
        .current_dir(root)
        .args(["test", "--workspace", "-j", "1"]);
    run_command(&mut test_cmd, "cargo test --workspace -j 1")?;

    // 7. SPSS parity verification
    println!("\n=== [CI Gate] SPSS Feature Parity (xtask parity) ===");
    crate::parity::check_parity(root)?;
    println!(">>> [OK] Parity checklist");

    println!("\n=======================================================");
    println!("  ALL CI QUALITY GATES PASSED! (ArtCraft DoD: 100% OK) ");
    println!("=======================================================\n");

    Ok(())
}
