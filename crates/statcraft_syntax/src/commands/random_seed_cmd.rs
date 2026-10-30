use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_core::DataSet;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RandomSeedUndoEntry {
    pub previous_seed: Option<u64>,
}

pub struct RandomSeedCommand {
    history: Mutex<Vec<RandomSeedUndoEntry>>,
}

impl Default for RandomSeedCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomSeedCommand {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn to_sps(seed: Option<u64>) -> String {
        match seed {
            Some(s) => format!("SET SEED = {s}."),
            None => "SET SEED = RANDOM.".to_string(),
        }
    }
}

impl EngineCommand for RandomSeedCommand {
    fn id(&self) -> &'static str {
        "transform.random_seed"
    }

    fn menu_path(&self) -> &'static str {
        "Transform > Set Random Number Seed"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seed": {
                    "type": ["integer", "null"],
                    "minimum": 1,
                    "maximum": 2_000_000_000,
                    "description": "Random number seed (1 - 2,000,000,000), or null for random initialization"
                }
            }
        })
    }

    fn enabled(&self, _dataset: Option<&DataSet>) -> bool {
        true
    }

    fn execute(
        &self,
        _dataset: &mut Option<DataSet>,
        params: Value,
    ) -> Result<Value, CommandError> {
        let seed_val: Option<u64> = match params.get("seed") {
            Some(v) if v.is_null() => None,
            Some(v) => {
                if let Some(i) = v.as_i64() {
                    if !(1..=2_000_000_000).contains(&i) {
                        return Err(CommandError::InvalidParams {
                            id: self.id(),
                            reason: format!(
                                "Seed value must be between 1 and 2,000,000,000, got {}",
                                i
                            ),
                        });
                    }
                    Some(i as u64)
                } else if let Some(u) = v.as_u64() {
                    if !(1..=2_000_000_000).contains(&u) {
                        return Err(CommandError::InvalidParams {
                            id: self.id(),
                            reason: format!(
                                "Seed value must be between 1 and 2,000,000,000, got {}",
                                u
                            ),
                        });
                    }
                    Some(u)
                } else {
                    return Err(CommandError::InvalidParams {
                        id: self.id(),
                        reason: "Seed must be an integer or null".to_string(),
                    });
                }
            }
            None => None,
        };

        let prev_seed = statcraft_core::get_global_random_seed();

        if let Ok(mut hist) = self.history.lock() {
            hist.push(RandomSeedUndoEntry {
                previous_seed: prev_seed,
            });
        }

        match seed_val {
            Some(s) => statcraft_core::set_global_random_seed(s),
            None => statcraft_core::clear_global_random_seed(),
        }

        Ok(json!({
            "status": "ok",
            "seed": seed_val,
            "previous_seed": prev_seed
        }))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        let entry = {
            let mut hist = self
                .history
                .lock()
                .map_err(|_| CommandError::ExecutionFailed("Mutex poison error".to_string()))?;
            hist.pop()
        };

        if let Some(entry) = entry {
            match entry.previous_seed {
                Some(s) => statcraft_core::set_global_random_seed(s),
                None => statcraft_core::clear_global_random_seed(),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_seed_to_sps() {
        assert_eq!(
            RandomSeedCommand::to_sps(Some(2_000_000)),
            "SET SEED = 2000000."
        );
        assert_eq!(RandomSeedCommand::to_sps(Some(12345)), "SET SEED = 12345.");
        assert_eq!(RandomSeedCommand::to_sps(None), "SET SEED = RANDOM.");
    }

    #[test]
    fn test_random_seed_execute_and_undo() {
        let cmd = RandomSeedCommand::new();
        let mut ds = None;

        // 1. Initial clear
        statcraft_core::clear_global_random_seed();
        assert_eq!(statcraft_core::get_global_random_seed(), None);

        // 2. Set seed = 54321
        let res = cmd
            .execute(&mut ds, json!({ "seed": 54321 }))
            .expect("execute should succeed");
        assert_eq!(res["status"], "ok");
        assert_eq!(res["seed"], 54321);
        assert_eq!(statcraft_core::get_global_random_seed(), Some(54321));

        // 3. Set seed = 99999
        let res2 = cmd
            .execute(&mut ds, json!({ "seed": 99999 }))
            .expect("execute should succeed");
        assert_eq!(res2["seed"], 99999);
        assert_eq!(res2["previous_seed"], 54321);
        assert_eq!(statcraft_core::get_global_random_seed(), Some(99999));

        // 4. Undo step 3 -> should restore 54321
        cmd.undo(&mut ds).expect("undo should succeed");
        assert_eq!(statcraft_core::get_global_random_seed(), Some(54321));

        // 5. Undo step 2 -> should restore None
        cmd.undo(&mut ds).expect("undo should succeed");
        assert_eq!(statcraft_core::get_global_random_seed(), None);
    }

    #[test]
    fn test_random_seed_validation() {
        let cmd = RandomSeedCommand::new();
        let mut ds = None;

        // Out of bounds (< 1)
        let err1 = cmd.execute(&mut ds, json!({ "seed": 0 }));
        assert!(err1.is_err());

        // Negative
        let err2 = cmd.execute(&mut ds, json!({ "seed": -5 }));
        assert!(err2.is_err());

        // Out of bounds (> 2,000,000,000)
        let err3 = cmd.execute(&mut ds, json!({ "seed": 2_000_000_001_i64 }));
        assert!(err3.is_err());

        // Invalid type
        let err4 = cmd.execute(&mut ds, json!({ "seed": "not a number" }));
        assert!(err4.is_err());

        // Null is valid (random mode)
        let ok_null = cmd.execute(&mut ds, json!({ "seed": null }));
        assert!(ok_null.is_ok());
        assert_eq!(statcraft_core::get_global_random_seed(), None);
    }
}
