use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_spec() {
        let cfg = PlanConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.plan_directory, ".aionrs/plans");
    }

    #[test]
    fn toml_full_override() {
        let toml_str = r#"
enabled = false
plan_directory = "/custom/plans"
"#;
        let cfg: PlanConfig = toml::from_str(toml_str).unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.plan_directory, "/custom/plans");
    }

    #[test]
    fn toml_partial_override_uses_defaults() {
        let toml_str = r#"
enabled = false
"#;
        let cfg: PlanConfig = toml::from_str(toml_str).unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.plan_directory, ".aionrs/plans");
    }

    #[test]
    fn toml_empty_uses_all_defaults() {
        let cfg: PlanConfig = toml::from_str("").unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.plan_directory, ".aionrs/plans");
        assert!(cfg.prompt.is_none());
    }

    #[test]
    fn toml_prompt_field_round_trips() {
        let toml_str = r#"
enabled = true
plan_directory = "/custom/plans"
prompt = "MY OVERRIDE PLAN MODE TEXT"
"#;
        let cfg: PlanConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.prompt.as_deref(), Some("MY OVERRIDE PLAN MODE TEXT"));
    }

    #[test]
    fn json_serialization_roundtrip() {
        let cfg = PlanConfig {
            enabled: false,
            plan_directory: "/tmp/plans".to_string(),
            prompt: None,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PlanConfig = serde_json::from_str(&json).unwrap();
        assert!(!back.enabled);
        assert_eq!(back.plan_directory, "/tmp/plans");
    }
}
