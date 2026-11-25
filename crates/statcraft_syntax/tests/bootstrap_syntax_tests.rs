use serde_json::{json, Value};
use statcraft_algo::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};
use statcraft_core::DataSet;
use statcraft_syntax::{
    bootstrap_param_schema, bootstrap_to_sps, parse_bootstrap_command, parse_set_mtindex,
    CommandBus, CommandError, EngineCommand, SyntaxDispatcher, SyntaxLexer, SyntaxParser,
};

struct ProbeCommand;

impl EngineCommand for ProbeCommand {
    fn id(&self) -> &'static str {
        "test.probe_bootstrap"
    }
    fn menu_path(&self) -> &'static str {
        "Test > Probe"
    }
    fn default_shortcut(&self) -> &'static str {
        ""
    }
    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "bootstrap": bootstrap_param_schema()
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
        Ok(params)
    }
    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }
}

#[test]
fn test_bootstrap_param_schema_valid() {
    let schema = bootstrap_param_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["samples"].is_object());
    assert!(schema["properties"]["ci_method"].is_object());
    assert!(schema["properties"]["sampling_method"].is_object());
}

#[test]
fn test_bootstrap_sps_roundtrip_simple() {
    let opts = BootstrapOptions {
        enabled: true,
        samples: 1000,
        seed: None,
        ci_level: 95.0,
        ci_method: BootstrapCiMethod::Percentile,
        sampling_method: BootstrapSamplingMethod::Simple,
    };

    let sps = bootstrap_to_sps(&opts);
    assert!(sps.contains("BOOTSTRAP"));
    assert!(sps.contains("/SAMPLING METHOD=SIMPLE"));
    assert!(sps.contains("/CRITERIA CILEVEL=95 CITYPE=PERCENTILE NSAMPLES=1000"));
    assert!(!sps.contains("MTINDEX"));

    let blocks = SyntaxLexer::split_commands(&sps).unwrap();
    assert_eq!(blocks.len(), 1);
    let parsed = SyntaxParser::parse_command(&blocks[0]).unwrap();
    let roundtrip = parse_bootstrap_command(&parsed).unwrap();
    assert_eq!(roundtrip, opts);
}

#[test]
fn test_bootstrap_sps_roundtrip_stratified_bca_with_seed() {
    let opts = BootstrapOptions {
        enabled: true,
        samples: 500,
        seed: Some(2_000_000),
        ci_level: 99.0,
        ci_method: BootstrapCiMethod::Bca,
        sampling_method: BootstrapSamplingMethod::Stratified {
            strata_vars: vec!["gender".to_string(), "region".to_string()],
        },
    };

    let sps = bootstrap_to_sps(&opts);
    assert!(sps.starts_with("SET MTINDEX=2000000."));
    assert!(sps.contains("/SAMPLING METHOD=STRATIFIED(STRATA=gender region)"));
    assert!(sps.contains("/CRITERIA CILEVEL=99 CITYPE=BCA NSAMPLES=500"));

    let blocks = SyntaxLexer::split_commands(&sps).unwrap();
    assert_eq!(blocks.len(), 2);

    let parsed_set = SyntaxParser::parse_command(&blocks[0]).unwrap();
    let seed = parse_set_mtindex(&parsed_set).unwrap();
    assert_eq!(seed, Some(Some(2_000_000)));

    let parsed_boot = SyntaxParser::parse_command(&blocks[1]).unwrap();
    let mut roundtrip = parse_bootstrap_command(&parsed_boot).unwrap();
    roundtrip.seed = seed.flatten();
    assert_eq!(roundtrip, opts);
}

#[test]
fn test_dispatcher_pending_bootstrap_is_one_shot() {
    let mut bus = CommandBus::with_defaults();
    bus.register(Box::new(ProbeCommand));
    let mut ds = Some(DataSet::empty());

    let script = r#"
    SET MTINDEX=2000000.
    BOOTSTRAP
      /SAMPLING METHOD=STRATIFIED(STRATA=group)
      /CRITERIA CILEVEL=95 CITYPE=BCA NSAMPLES=250
      /MISSING USERMISSING=EXCLUDE.
    RUN COMMAND "test.probe_bootstrap" { "x": 1 }.
    RUN COMMAND "test.probe_bootstrap" { "x": 2 }.
    "#;

    let results = SyntaxDispatcher::execute_batch(script, &mut ds, &mut bus).unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].1["command_id"], "util.set_mtindex");
    assert_eq!(results[1].1["command_id"], "util.bootstrap");

    // First probe receives injected bootstrap options
    let first_boot = &results[2].1["bootstrap"];
    assert_eq!(first_boot["enabled"], true);
    assert_eq!(first_boot["samples"], 250);
    assert_eq!(first_boot["seed"], 2_000_000);
    assert_eq!(first_boot["ci_method"], "bca");
    assert_eq!(first_boot["sampling_method"]["method"], "stratified");

    // Second probe does NOT receive bootstrap options (one-shot cleared)
    assert!(results[3].1.get("bootstrap").is_none());
}
