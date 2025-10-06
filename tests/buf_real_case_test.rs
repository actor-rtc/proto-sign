//! Test with actual Buf extracted test cases
//!
//! This uses the real proto files extracted from Buf's test suite

use proto_sign::compat::BreakingConfig;
use proto_sign::spec::Spec;

#[test]
fn test_real_buf_enum_no_delete_case() {
    // These are the actual proto files from Buf's test suite
    let previous_proto_1 = r#"
syntax = "proto3";

package a;

enum One {
  ONE_UNSPECIFIED = 0;
}

enum Two {
  TWO_UNSPECIFIED = 0;
}

message Three {
  message Four {
    enum Five {
      FIVE_UNSPECIFIED = 0;
    }
    enum Six {
      SIX_UNSPECIFIED = 0;
    }
  }
  enum Seven {
    SEVEN_UNSPECIFIED = 0;
  }
  enum Eight {
    EIGHT_UNSPECIFIED = 0;
  }
}
"#;

    let current_proto_1 = r#"
syntax = "proto3";

package a;

enum One {
  ONE_UNSPECIFIED = 0;
}

message Three {
  message Four {
    enum Six {
      SIX_UNSPECIFIED = 0;
    }
  }
  enum Eight {
    EIGHT_UNSPECIFIED = 0;
  }
}
"#;

    let previous_spec = Spec::try_from(previous_proto_1).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto_1).expect("Failed to parse current proto");

    // Use PACKAGE_ENUM_NO_DELETE which exists in Buf
    let config = BreakingConfig {
        use_rules: vec!["PACKAGE_ENUM_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    // Buf should detect these breaking changes
    assert!(
        result.has_breaking_changes,
        "Should detect breaking changes like Buf does"
    );

    let changes = result.changes;
    assert!(
        changes.len() >= 2,
        "Should detect at least 2 enum deletions, got: {}",
        changes.len()
    );

    // Verify we detect the expected deleted enums
    let deleted_enums: std::collections::HashSet<&str> = changes
        .iter()
        .filter(|c| c.rule_id == "PACKAGE_ENUM_NO_DELETE")
        .map(|c| c.location.element_name.as_str())
        .collect();

    println!("Detected deleted enums: {deleted_enums:?}");

    // Expected deletions: "Two", "Three.Four.Five", "Three.Seven"
    assert!(
        deleted_enums.contains("Two"),
        "Should detect 'Two' deletion"
    );
    assert!(deleted_enums.len() >= 2, "Should detect multiple deletions");

    println!(
        "✅ Real Buf test case PASSED - detected {} breaking changes",
        changes.len()
    );
    for change in changes {
        println!("  - {}", change.message);
    }
}

#[test]
fn test_field_type_compatibility() {
    // Test wire-compatible vs wire-incompatible type changes
    let previous_proto = r#"
syntax = "proto3";

message Test {
    int32 compatible_field = 1;
    string incompatible_field = 2;
}
"#;

    let current_proto = r#"
syntax = "proto3";

message Test {
    uint32 compatible_field = 1;     // int32 -> uint32 is wire compatible
    bytes incompatible_field = 2;    // string -> bytes might be different in JSON
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    // Test FIELD_SAME_TYPE (strict)
    let strict_config = BreakingConfig {
        use_rules: vec!["FIELD_SAME_TYPE".to_string()],
        ..Default::default()
    };

    let strict_result =
        previous_spec.check_breaking_changes_with_config(&current_spec, &strict_config);
    assert!(
        strict_result.has_breaking_changes,
        "FIELD_SAME_TYPE should detect any type change"
    );
    assert_eq!(
        strict_result.changes.len(),
        2,
        "Should detect both type changes"
    );

    // Test FIELD_WIRE_COMPATIBLE_TYPE (more permissive)
    let wire_config = BreakingConfig {
        use_rules: vec!["FIELD_WIRE_COMPATIBLE_TYPE".to_string()],
        ..Default::default()
    };

    let wire_result = previous_spec.check_breaking_changes_with_config(&current_spec, &wire_config);
    // This might detect fewer changes depending on wire compatibility rules
    println!(
        "Wire compatible check found {} changes",
        wire_result.changes.len()
    );

    for change in &strict_result.changes {
        println!("Strict: {}", change.message);
    }

    for change in &wire_result.changes {
        println!("Wire: {}", change.message);
    }

    println!("✅ Wire compatibility test completed");
}

#[test]
fn test_message_required_fields() {
    let previous_proto = r#"
syntax = "proto2";

message User {
    optional string name = 1;
    optional int32 age = 2;
}
"#;

    let current_proto = r#"
syntax = "proto2";

message User {
    required string name = 1;  // Changed from optional to required
    optional int32 age = 2;
    required string email = 3; // Added new required field
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    let config = BreakingConfig {
        use_rules: vec!["MESSAGE_SAME_REQUIRED_FIELDS".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    // Should detect both the changed field and new required field
    assert!(
        result.has_breaking_changes,
        "Should detect required field changes"
    );
    println!(
        "Required fields check found {} changes",
        result.changes.len()
    );

    for change in &result.changes {
        println!("Required field: {}", change.message);
    }

    println!("✅ Required fields test completed");
}

#[test]
fn test_comprehensive_file_comparison() {
    // More comprehensive test with multiple types of changes
    let previous_proto = r#"
syntax = "proto3";

package myapp.v1;

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    ACTIVE = 1;
    INACTIVE = 2;
    DELETED = 3;
}

message User {
    string id = 1;
    string name = 2;
    UserStatus status = 3;
    repeated string tags = 4;
    
    message Settings {
        bool notifications = 1;
        string theme = 2;
    }
}

message Account {
    string account_id = 1;
    User owner = 2;
}

service UserService {
    rpc GetUser(User) returns (User);
    rpc UpdateUser(User) returns (User);
    rpc DeleteUser(User) returns (User);
}
"#;

    let current_proto = r#"
syntax = "proto3";

package myapp.v1;

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    ACTIVE = 1;
    INACTIVE = 2;
    // Removed DELETED = 3
    SUSPENDED = 4;  // Added new value
}

message User {
    string id = 1;
    string full_name = 2;  // Changed field name from 'name' to 'full_name'
    UserStatus status = 3;
    // Removed tags field
    
    message Settings {
        bool notifications = 1;
        string theme = 2;
        int32 version = 3;  // Added new field
    }
    
    message Profile {  // Added new nested message
        string bio = 1;
    }
}

// Removed Account message entirely

service UserService {
    rpc GetUser(User) returns (User);
    rpc UpdateUser(User) returns (User);
    // Removed DeleteUser RPC
    rpc CreateUser(User) returns (User);  // Added new RPC
}

service NotificationService {  // Added new service
    rpc SendNotification(User) returns (User);
}
"#;

    let previous_spec = Spec::try_from(previous_proto).expect("Failed to parse previous proto");
    let current_spec = Spec::try_from(current_proto).expect("Failed to parse current proto");

    // Use FILE category to catch most changes
    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        ..Default::default()
    };

    let result = previous_spec.check_breaking_changes_with_config(&current_spec, &config);

    assert!(
        result.has_breaking_changes,
        "Should detect multiple breaking changes"
    );

    // Group changes by rule type
    let mut changes_by_rule: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for change in &result.changes {
        *changes_by_rule.entry(change.rule_id.clone()).or_insert(0) += 1;
    }

    println!(
        "✅ Comprehensive test found {} total breaking changes:",
        result.changes.len()
    );
    for (rule_id, count) in &changes_by_rule {
        println!("  - {rule_id}: {count} changes");
    }

    // Should detect various types of breaking changes:
    // - ENUM_VALUE_NO_DELETE (for DELETED enum value)
    // - FIELD_NO_DELETE (for tags field)
    // - FIELD_SAME_NAME (for name->full_name change)
    // - MESSAGE_NO_DELETE (for Account message)
    // - RPC_NO_DELETE (for DeleteUser RPC)

    assert!(
        changes_by_rule.contains_key("ENUM_VALUE_NO_DELETE"),
        "Should detect deleted enum value"
    );
    assert!(
        changes_by_rule.contains_key("FIELD_NO_DELETE"),
        "Should detect deleted field"
    );
    assert!(
        changes_by_rule.contains_key("MESSAGE_NO_DELETE"),
        "Should detect deleted message"
    );
    assert!(
        changes_by_rule.contains_key("RPC_NO_DELETE"),
        "Should detect deleted RPC"
    );

    println!("✅ All expected rule types were triggered");
}
