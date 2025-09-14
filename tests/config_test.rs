//! Tests for YAML configuration loading and parsing

use proto_sign::compat::BreakingConfig;

#[test]
fn test_load_yaml_config() {
    let yaml_content = r#"
version: v1
breaking:
  use_categories:
    - FILE
    - PACKAGE
  except_rules:
    - FIELD_SAME_DEFAULT
  ignore:
    - generated/**
    - test/fixtures/**
  ignore_only:
    FIELD_NO_DELETE:
      - deprecated/old.proto
    MESSAGE_NO_DELETE:
      - internal/legacy/**
  ignore_unstable_packages: true
"#;

    let config = BreakingConfig::from_yaml_str(yaml_content).unwrap();
    
    assert_eq!(config.use_categories, vec!["FILE", "PACKAGE"]);
    assert_eq!(config.except_rules, vec!["FIELD_SAME_DEFAULT"]);
    assert_eq!(config.ignore, vec!["generated/**", "test/fixtures/**"]);
    assert!(config.ignore_unstable_packages);
    
    // Check rule-specific ignores
    assert_eq!(
        config.ignore_only.get("FIELD_NO_DELETE").unwrap(),
        &vec!["deprecated/old.proto"]
    );
    assert_eq!(
        config.ignore_only.get("MESSAGE_NO_DELETE").unwrap(),
        &vec!["internal/legacy/**"]
    );
}

#[test]
fn test_load_minimal_yaml_config() {
    let yaml_content = r#"
version: v1
breaking:
  use_categories:
    - WIRE_JSON
"#;

    let config = BreakingConfig::from_yaml_str(yaml_content).unwrap();
    
    assert_eq!(config.use_categories, vec!["WIRE_JSON"]);
    assert!(config.except_rules.is_empty());
    assert!(config.ignore.is_empty());
    assert!(!config.ignore_unstable_packages);
}

#[test]
fn test_load_empty_yaml_config() {
    let yaml_content = r#"
version: v1
"#;

    let config = BreakingConfig::from_yaml_str(yaml_content).unwrap();
    
    // Should use default values
    assert_eq!(config.use_categories, vec!["FILE", "PACKAGE"]);
    assert!(config.except_rules.is_empty());
    assert!(config.ignore.is_empty());
    assert!(!config.ignore_unstable_packages);
}

#[test]
fn test_load_rules_only_config() {
    let yaml_content = r#"
version: v1
breaking:
  use_rules:
    - MESSAGE_NO_DELETE
    - FIELD_NO_DELETE
    - ENUM_NO_DELETE
"#;

    let config = BreakingConfig::from_yaml_str(yaml_content).unwrap();
    
    assert_eq!(
        config.use_rules, 
        vec!["MESSAGE_NO_DELETE", "FIELD_NO_DELETE", "ENUM_NO_DELETE"]
    );
    assert!(config.use_categories.is_empty()); // Should override categories
}