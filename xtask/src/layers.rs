use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerLevel {
    L0Io,
    L1Core,
    L2Algo,
    L3Charts,
    L4Reporting,
    L5Syntax,
    L6App,
    Utility,
}

pub struct CrateSpec {
    pub name: &'static str,
    pub level: LayerLevel,
    pub allowed_dependencies: &'static [&'static str],
}

pub const CRATE_REGISTRY: &[CrateSpec] = &[
    CrateSpec {
        name: "statcraft_i18n",
        level: LayerLevel::Utility,
        allowed_dependencies: &[],
    },
    CrateSpec {
        name: "statcraft_io",
        level: LayerLevel::L0Io,
        allowed_dependencies: &[],
    },
    CrateSpec {
        name: "statcraft_core",
        level: LayerLevel::L1Core,
        allowed_dependencies: &["statcraft_io"],
    },
    CrateSpec {
        name: "statcraft_algo",
        level: LayerLevel::L2Algo,
        allowed_dependencies: &["statcraft_core"],
    },
    CrateSpec {
        name: "statcraft_syntax",
        level: LayerLevel::L5Syntax,
        allowed_dependencies: &["statcraft_core", "statcraft_io", "statcraft_algo"],
    },
    CrateSpec {
        name: "statcraft_app",
        level: LayerLevel::L6App,
        allowed_dependencies: &[
            "statcraft_core",
            "statcraft_io",
            "statcraft_syntax",
            "statcraft_i18n",
            "statcraft_algo",
        ],
    },
];

pub fn check_layers(workspace_root: &Path) -> Result<(), String> {
    let mut crate_map = HashMap::new();
    for spec in CRATE_REGISTRY {
        crate_map.insert(spec.name, spec);
    }

    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }

    let entries =
        fs::read_dir(&crates_dir).map_err(|e| format!("Failed to read crates directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let crate_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        let spec = match crate_map.get(crate_name) {
            Some(s) => *s,
            None => {
                return Err(format!(
                    "Unregistered crate '{}' found in crates/. Please register it in xtask/src/layers.rs",
                    crate_name
                ));
            }
        };

        let cargo_toml_path = path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let content = fs::read_to_string(&cargo_toml_path)
                .map_err(|e| format!("Failed to read {}: {}", cargo_toml_path.display(), e))?;

            for other in CRATE_REGISTRY {
                if other.name == spec.name {
                    continue;
                }
                // Check if other crate name is listed as a dependency
                let needle = format!("{} = ", other.name);
                let needle_path = format!("{} = {{", other.name);
                if (content.contains(&needle) || content.contains(&needle_path))
                    && !spec.allowed_dependencies.contains(&other.name)
                {
                    return Err(format!(
                        "Layer violation: crate '{}' ({:?}) cannot depend on '{}' ({:?})",
                        spec.name, spec.level, other.name, other.level
                    ));
                }
            }
        }
    }

    println!("[xtask layers] All crate layers adhere strictly to ArtCraft architecture rules.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_registry_contains_core_crates() {
        let names: Vec<_> = CRATE_REGISTRY.iter().map(|c| c.name).collect();
        assert!(names.contains(&"statcraft_core"));
        assert!(names.contains(&"statcraft_algo"));
        assert!(names.contains(&"statcraft_io"));
        assert!(names.contains(&"statcraft_syntax"));
        assert!(names.contains(&"statcraft_app"));
        assert!(names.contains(&"statcraft_i18n"));
    }

    #[test]
    fn test_io_cannot_depend_on_core() {
        let io_spec = CRATE_REGISTRY
            .iter()
            .find(|c| c.name == "statcraft_io")
            .unwrap();
        assert!(!io_spec.allowed_dependencies.contains(&"statcraft_core"));
        assert!(!io_spec.allowed_dependencies.contains(&"statcraft_app"));
    }

    #[test]
    fn test_core_cannot_depend_on_app() {
        let core_spec = CRATE_REGISTRY
            .iter()
            .find(|c| c.name == "statcraft_core")
            .unwrap();
        assert!(!core_spec.allowed_dependencies.contains(&"statcraft_app"));
    }
}
