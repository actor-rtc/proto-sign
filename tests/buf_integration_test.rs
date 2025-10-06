//! Integration test using real Buf test cases
//!
//! This test verifies that our implementation produces the same results as Buf
//! by using the extracted test cases from the Buf project.

use proto_sign::compat::BreakingConfig;
use proto_sign::spec::Spec;

/// Test enum value deletion detection using Buf compatible rule
#[test]
fn test_enum_value_no_delete_basic() {
    let previous_proto = r#"
syntax = "proto3";

package a;

enum Status {
  STATUS_UNSPECIFIED = 0;
  ACTIVE = 1;
  INACTIVE = 2;
}
"#;

    let current_proto = r#"
syntax = "proto3";

package a;

enum Status {
  STATUS_UNSPECIFIED = 0;
  ACTIVE = 1;
  // INACTIVE = 2; was deleted
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["ENUM_VALUE_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    // Should detect deleted enum value
    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one enum value deletion"
    );

    println!("Detected breaking changes:");
    for change in &result.changes {
        println!(
            "  - {}: {} (location: {})",
            change.rule_id, change.message, change.location.element_name
        );
    }

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "ENUM_VALUE_NO_DELETE");
    assert!(
        change.message.contains("INACTIVE"),
        "Should mention deleted 'INACTIVE' enum value"
    );
    assert!(
        change.message.contains("2"),
        "Should mention enum value number 2"
    );
}

/// Test field deletion detection
#[test]
fn test_field_no_delete_basic() {
    let previous_proto = r#"
syntax = "proto3";

message Test {
    string name = 1;
    int32 id = 2;
    bool active = 3;
}
"#;

    let current_proto = r#"
syntax = "proto3";

message Test {
    string name = 1;
    bool active = 3;
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["FIELD_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one field deletion"
    );

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "FIELD_NO_DELETE");
    assert!(
        change.message.contains("id"),
        "Should mention deleted 'id' field"
    );
    assert!(
        change.message.contains("2"),
        "Should mention field number 2"
    );

    println!("Detected change: {}", change.message);
}

/// Test message deletion detection
#[test]
fn test_message_no_delete_basic() {
    let previous_proto = r#"
syntax = "proto3";

message User {
    string name = 1;
}

message Account {
    string email = 1;
}
"#;

    let current_proto = r#"
syntax = "proto3";

message User {
    string name = 1;
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["MESSAGE_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one message deletion"
    );

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "MESSAGE_NO_DELETE");
    assert!(
        change.message.contains("Account"),
        "Should mention deleted 'Account' message"
    );

    println!("Detected change: {}", change.message);
}

/// Test field type changes
#[test]
fn test_field_same_type_basic() {
    let previous_proto = r#"
syntax = "proto3";

message Test {
    string name = 1;
    int32 count = 2;
}
"#;

    let current_proto = r#"
syntax = "proto3";

message Test {
    string name = 1;
    int64 count = 2;
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["FIELD_SAME_TYPE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one type change"
    );

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "FIELD_SAME_TYPE");
    assert!(
        change.message.contains("count"),
        "Should mention 'count' field"
    );
    assert!(change.message.contains("int32"), "Should mention old type");
    assert!(change.message.contains("int64"), "Should mention new type");

    println!("Detected change: {}", change.message);
}

/// Test service deletion detection
#[test]
fn test_service_no_delete_basic() {
    let previous_proto = r#"
syntax = "proto3";

message Request {}
message Response {}

service UserService {
    rpc GetUser(Request) returns (Response);
}

service AccountService {
    rpc GetAccount(Request) returns (Response);
}
"#;

    let current_proto = r#"
syntax = "proto3";

message Request {}
message Response {}

service UserService {
    rpc GetUser(Request) returns (Response);
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["SERVICE_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one service deletion"
    );

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "SERVICE_NO_DELETE");
    assert!(
        change.message.contains("AccountService"),
        "Should mention deleted 'AccountService'"
    );

    println!("Detected change: {}", change.message);
}

/// Test RPC deletion detection
#[test]
fn test_rpc_no_delete_basic() {
    let previous_proto = r#"
syntax = "proto3";

message Request {}
message Response {}

service UserService {
    rpc GetUser(Request) returns (Response);
    rpc DeleteUser(Request) returns (Response);
}
"#;

    let current_proto = r#"
syntax = "proto3";

message Request {}
message Response {}

service UserService {
    rpc GetUser(Request) returns (Response);
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["RPC_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert_eq!(
        result.changes.len(),
        1,
        "Should detect exactly one RPC deletion"
    );

    let change = &result.changes[0];
    assert_eq!(change.rule_id, "RPC_NO_DELETE");
    assert!(
        change.message.contains("DeleteUser"),
        "Should mention deleted 'DeleteUser' RPC"
    );

    println!("Detected change: {}", change.message);
}

/// Test multiple rules together
#[test]
fn test_multiple_breaking_changes() {
    let previous_proto = r#"
syntax = "proto3";

enum Status {
    STATUS_UNSPECIFIED = 0;
    ACTIVE = 1;
}

message User {
    string name = 1;
    int32 age = 2;
    Status status = 3;
}

service UserService {
    rpc GetUser(User) returns (User);
}
"#;

    let current_proto = r#"
syntax = "proto3";

message User {
    string name = 1;
    string status = 3;  // Changed type from Status to string
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes"
    );
    assert!(
        result.changes.len() >= 3,
        "Should detect multiple breaking changes: {:#?}",
        result.changes
    );

    let rule_ids: std::collections::HashSet<&str> =
        result.changes.iter().map(|c| c.rule_id.as_str()).collect();

    // Should detect field deletion, field type change, and service deletion
    // Note: No ENUM_NO_DELETE rule in Buf (only PACKAGE_ENUM_NO_DELETE and ENUM_VALUE_NO_DELETE)
    assert!(
        rule_ids.contains("FIELD_NO_DELETE"),
        "Should detect field deletion"
    );
    assert!(
        rule_ids.contains("FIELD_SAME_TYPE"),
        "Should detect field type change"
    );
    assert!(
        rule_ids.contains("SERVICE_NO_DELETE"),
        "Should detect service deletion"
    );

    println!("Detected {} breaking changes:", result.changes.len());
    for change in &result.changes {
        println!("  - {}: {}", change.rule_id, change.message);
    }
}

/// Test that no breaking changes are detected for identical files
#[test]
fn test_no_breaking_changes() {
    let proto = r#"
syntax = "proto3";

enum Status {
    STATUS_UNSPECIFIED = 0;
    ACTIVE = 1;
}

message User {
    string name = 1;
    int32 age = 2;
    Status status = 3;
}

service UserService {
    rpc GetUser(User) returns (User);
}
"#;

    let spec = Spec::try_from(proto).expect("Failed to parse proto");

    let config = BreakingConfig::default();
    let result = spec.check_breaking_changes_with_config(&spec, &config);

    assert!(
        !result.has_breaking_changes,
        "Should not detect breaking changes for identical files"
    );
    assert_eq!(result.changes.len(), 0, "Should have no changes");

    println!("Rules executed: {:?}", result.executed_rules);
    assert!(
        !result.executed_rules.is_empty(),
        "Should have executed some rules"
    );
}
