use std::fs;
use std::path::Path;
use std::process::Command;

const PRE_PUSH_SCRIPT: &str = r#"#!/bin/sh
# StatCraft Quality Gate Pre-Push Hook (ArtCraft standard)
# Automatically executes all CI quality gates before allowing git push.

echo ""
echo "================================================================"
echo " [Pre-Push Hook] Running StatCraft Quality Gates (xtask ci)..."
echo "================================================================"
echo ""

cargo run -p xtask -j 1 -- ci

EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "================================================================"
    echo " [BLOCKED] Push rejected: CI Quality Gates failed locally!     "
    echo " Please fix the errors listed above before pushing to remote.   "
    echo " Exit code: $EXIT_CODE                                          "
    echo "================================================================"
    echo ""
    exit 1
fi

echo ""
echo ">>> [Pre-Push Hook] All Quality Gates PASSED! Proceeding to push."
echo ""
exit 0
"#;

pub fn install_hooks(workspace_root: &Path) -> Result<(), String> {
    let githooks_dir = workspace_root.join(".githooks");
    if !githooks_dir.exists() {
        fs::create_dir_all(&githooks_dir)
            .map_err(|e| format!("Failed to create .githooks directory: {}", e))?;
    }

    let target_pre_push = githooks_dir.join("pre-push");
    fs::write(&target_pre_push, PRE_PUSH_SCRIPT)
        .map_err(|e| format!("Failed to write .githooks/pre-push: {}", e))?;

    // Try setting git config core.hooksPath .githooks
    let mut config_cmd = Command::new("git");
    config_cmd
        .current_dir(workspace_root)
        .args(["config", "core.hooksPath", ".githooks"]);

    match config_cmd.status() {
        Ok(status) if status.success() => {
            println!(">>> [OK] Git core.hooksPath configured to '.githooks'.");
        }
        _ => {
            // Also try writing to .git/hooks directly as fallback
            let git_hooks = workspace_root.join(".git").join("hooks");
            if git_hooks.exists() {
                let _ = fs::write(git_hooks.join("pre-push"), PRE_PUSH_SCRIPT);
            }
        }
    }

    println!(
        ">>> [OK] Version-controlled hook created at: {}",
        target_pre_push.display()
    );
    println!(
        ">>> Quality Gate will now run 'cargo run -p xtask -j 1 -- ci' before every 'git push'."
    );

    Ok(())
}
