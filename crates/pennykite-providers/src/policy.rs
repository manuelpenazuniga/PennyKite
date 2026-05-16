//! Policy loader: reads and validates YAML policy files.

use pennykite_types::Policy;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

/// Load and validate a policy from a YAML file.
pub fn load_policy(path: impl AsRef<Path>) -> Result<Policy, PolicyError> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path).map_err(|e| PolicyError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let policy: Policy = serde_yaml::from_str(&contents)?;
    validate(&policy)?;
    Ok(policy)
}

fn validate(policy: &Policy) -> Result<(), PolicyError> {
    if policy.session.budget_usd <= 0.0 {
        return Err(PolicyError::Validation(
            "session.budget_usd must be positive".into(),
        ));
    }
    if policy.session.per_request_cap_usd <= 0.0 {
        return Err(PolicyError::Validation(
            "session.per_request_cap_usd must be positive".into(),
        ));
    }
    if policy.networks.allowed.is_empty() {
        return Err(PolicyError::Validation(
            "networks.allowed must not be empty".into(),
        ));
    }
    if policy.networks.assets.is_empty() {
        return Err(PolicyError::Validation(
            "networks.assets must not be empty".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_YAML: &str = r#"
version: "1.0"
name: test
session:
  budget_usd: 5.0
  per_request_cap_usd: 0.5
  max_duration_minutes: 30
loop_detection:
  enabled: true
  window_size: 10
  similarity_threshold: 0.85
  max_consecutive_similar: 3
networks:
  allowed: ["eip155:8453"]
  assets: ["USDC"]
kite_passport:
  agent_address: "0x0"
  session_key: "0x0"
  attestation_frequency: every_event
"#;

    fn parse(yaml: &str) -> Policy {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_validate_valid_policy() {
        let policy = parse(VALID_YAML);
        assert!(validate(&policy).is_ok());
    }

    #[test]
    fn test_validate_negative_budget() {
        let yaml = VALID_YAML.replace("budget_usd: 5.0", "budget_usd: -5.0");
        let policy = parse(&yaml);
        let err = validate(&policy).unwrap_err();
        assert!(matches!(err, PolicyError::Validation(ref m) if m.contains("budget_usd")));
    }

    #[test]
    fn test_validate_zero_per_request_cap() {
        let yaml = VALID_YAML.replace("per_request_cap_usd: 0.5", "per_request_cap_usd: 0.0");
        let policy = parse(&yaml);
        let err = validate(&policy).unwrap_err();
        assert!(matches!(err, PolicyError::Validation(ref m) if m.contains("per_request_cap_usd")));
    }

    #[test]
    fn test_validate_empty_allowed_networks() {
        let yaml = VALID_YAML.replace("allowed: [\"eip155:8453\"]", "allowed: []");
        let policy = parse(&yaml);
        let err = validate(&policy).unwrap_err();
        assert!(matches!(err, PolicyError::Validation(ref m) if m.contains("networks.allowed")));
    }

    #[test]
    fn test_validate_empty_assets() {
        let yaml = VALID_YAML.replace("assets: [\"USDC\"]", "assets: []");
        let policy = parse(&yaml);
        let err = validate(&policy).unwrap_err();
        assert!(matches!(err, PolicyError::Validation(ref m) if m.contains("networks.assets")));
    }

    #[test]
    fn test_load_conservative_preset() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../policy/examples/conservative.yaml");
        let policy = load_policy(&path).expect("conservative.yaml must load");
        assert_eq!(policy.session.budget_usd, 5.0);
    }
}
