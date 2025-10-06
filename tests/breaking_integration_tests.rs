//! Integration tests for breaking change detection
//!
//! These tests mirror Buf's breaking change detection tests to ensure
//! exact compatibility and correctness of rule implementations.

use proto_sign::canonical::CanonicalFile;
use proto_sign::compat::{BreakingConfig, BreakingEngine};
use proto_sign::spec::Spec;
use std::fs;
use std::path::PathBuf;

/// Test helper to load a proto file from testdata
fn load_test_file(relative_path: &str) -> CanonicalFile {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("compat-configs/extracted/testdata");
    path.push(relative_path);

    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read test file {}: {}", path.display(), e));

    let spec = Spec::try_from(&content)
        .unwrap_or_else(|e| panic!("Failed to parse proto file {}: {}", path.display(), e));

    spec.canonical_file
}

/// Test helper to run breaking change detection and get results
fn run_breaking_check(
    current_path: &str,
    previous_path: &str,
    config: Option<BreakingConfig>,
) -> proto_sign::compat::BreakingResult {
    let current = load_test_file(current_path);
    let previous = load_test_file(previous_path);
    let config = config.unwrap_or_default();

    let engine = BreakingEngine::new();
    engine.check(&current, &previous, &config)
}

#[test]
fn test_message_no_delete_basic() {
    // Use only FILE category to avoid PACKAGE_MESSAGE_NO_DELETE interference
    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        use_rules: Vec::new(),
        except_rules: Vec::new(),
        ignore: Vec::new(),
        ignore_only: std::collections::HashMap::new(),
        ignore_unstable_packages: false,
        service_no_change_suffixes: Vec::new(),
        message_no_change_suffixes: Vec::new(),
        enum_no_change_suffixes: Vec::new(),
    };

    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "previous/breaking_message_no_delete/1.proto",
        Some(config),
    );

    // Should detect 3 deleted messages: Two, Three.Four.Five, Three.Seven
    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );

    let message_delete_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "MESSAGE_NO_DELETE")
        .collect();

    assert_eq!(
        message_delete_changes.len(),
        3,
        "Should detect 3 message deletions, found: {message_delete_changes:?}"
    );

    // Verify specific messages were detected as deleted
    let deleted_messages: Vec<&str> = message_delete_changes
        .iter()
        .map(|c| {
            // Extract message name from the error message or location
            if c.message.contains("Two") {
                "Two"
            } else if c.message.contains("Five") {
                "Three.Four.Five"
            } else if c.message.contains("Seven") {
                "Three.Seven"
            } else {
                panic!("Unexpected message deletion: {}", c.message)
            }
        })
        .collect();

    assert!(
        deleted_messages.contains(&"Two"),
        "Should detect deletion of message 'Two'"
    );
    assert!(
        deleted_messages.contains(&"Three.Four.Five"),
        "Should detect deletion of nested message 'Three.Four.Five'"
    );
    assert!(
        deleted_messages.contains(&"Three.Seven"),
        "Should detect deletion of nested message 'Three.Seven'"
    );
}

#[test]
fn test_message_no_delete_no_changes() {
    // Test with identical files - should not report breaking changes
    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "current/breaking_message_no_delete/1.proto", // same file
        None,
    );

    assert!(
        !result.has_breaking_changes,
        "Identical files should not have breaking changes"
    );
    assert_eq!(result.changes.len(), 0);
}

#[test]
fn test_breaking_config_rule_selection() {
    // Test with specific rule enabled
    let config = BreakingConfig {
        use_rules: vec!["MESSAGE_NO_DELETE".to_string()],
        use_categories: Vec::new(),
        except_rules: Vec::new(),
        ignore: Vec::new(),
        ignore_only: std::collections::HashMap::new(),
        ignore_unstable_packages: false,
        service_no_change_suffixes: Vec::new(),
        message_no_change_suffixes: Vec::new(),
        enum_no_change_suffixes: Vec::new(),
    };

    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "previous/breaking_message_no_delete/1.proto",
        Some(config),
    );

    assert!(result.has_breaking_changes);
    assert!(
        result
            .executed_rules
            .contains(&"MESSAGE_NO_DELETE".to_string())
    );

    // Only MESSAGE_NO_DELETE rule should have been executed
    let message_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "MESSAGE_NO_DELETE")
        .collect();

    assert_eq!(message_changes.len(), 3);
}

#[test]
fn test_breaking_config_rule_exclusion() {
    // Test with MESSAGE_NO_DELETE explicitly disabled
    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        use_rules: Vec::new(),
        except_rules: vec!["MESSAGE_NO_DELETE".to_string()],
        ignore: Vec::new(),
        ignore_only: std::collections::HashMap::new(),
        ignore_unstable_packages: false,
        service_no_change_suffixes: Vec::new(),
        message_no_change_suffixes: Vec::new(),
        enum_no_change_suffixes: Vec::new(),
    };

    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "previous/breaking_message_no_delete/1.proto",
        Some(config),
    );

    // Should not find MESSAGE_NO_DELETE violations since it's excluded
    let message_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "MESSAGE_NO_DELETE")
        .collect();

    assert_eq!(
        message_changes.len(),
        0,
        "MESSAGE_NO_DELETE should be excluded"
    );
}

#[test]
fn test_breaking_result_summary() {
    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "previous/breaking_message_no_delete/1.proto",
        None,
    );

    // Verify summary contains FILE category
    assert!(
        result.summary.contains_key("FILE"),
        "Summary should contain FILE category"
    );

    // Verify changes by category
    let file_changes = result.summary.get("FILE").unwrap_or(&0);
    assert_eq!(*file_changes, 3);
}

#[test]
fn test_default_config_no_duplicates() {
    // Test with default configuration (FILE + PACKAGE) - should not duplicate results
    let result = run_breaking_check(
        "current/breaking_message_no_delete/1.proto",
        "previous/breaking_message_no_delete/1.proto",
        None, // Use default config
    );

    assert!(result.has_breaking_changes);

    // Count breaking changes by rule ID to detect duplicates
    let mut rule_counts = std::collections::HashMap::new();
    for change in &result.changes {
        *rule_counts.entry(change.rule_id.clone()).or_insert(0) += 1;
    }

    // MESSAGE_NO_DELETE should have 3 occurrences (one per deleted message)
    assert_eq!(
        *rule_counts.get("MESSAGE_NO_DELETE").unwrap_or(&0),
        3,
        "MESSAGE_NO_DELETE should detect 3 deletions"
    );

    // PACKAGE_MESSAGE_NO_DELETE should also have 3 occurrences (same package)
    assert_eq!(
        *rule_counts.get("PACKAGE_MESSAGE_NO_DELETE").unwrap_or(&0),
        3,
        "PACKAGE_MESSAGE_NO_DELETE should detect 3 deletions"
    );

    // Total changes should be 6 (3 FILE + 3 PACKAGE), not duplicated
    assert_eq!(result.changes.len(), 6);
}

// Additional tests for other basic rules can be added here
// Removed test_enum_no_delete: ENUM_NO_DELETE rule doesn't exist in Buf
// Use PACKAGE_ENUM_NO_DELETE or ENUM_VALUE_NO_DELETE instead for enum-related breaking changes

#[test]
fn test_field_no_delete() {
    let result = run_breaking_check(
        "current/breaking_field_no_delete/1.proto",
        "previous/breaking_field_no_delete/1.proto",
        None,
    );

    assert!(result.has_breaking_changes);

    let field_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "FIELD_NO_DELETE")
        .collect();

    assert!(!field_changes.is_empty(), "Should detect field deletions");
}
