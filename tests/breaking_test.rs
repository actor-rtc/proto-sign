use proto_sign::compat::BreakingConfig;
use proto_sign::spec::Spec;

#[test]
fn test_field_deletion_breaking_change() {
    let old_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  int32 age = 2;
}
"#;

    let new_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  // age field deleted - this should be a breaking change
}
"#;

    let old_spec = Spec::try_from(old_proto).expect("Failed to parse old proto");
    let new_spec = Spec::try_from(new_proto).expect("Failed to parse new proto");

    let result = old_spec.check_breaking_changes(&new_spec);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking change when field is deleted"
    );

    // Should find the FIELD_NO_DELETE rule violation
    let field_no_delete_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "FIELD_NO_DELETE")
        .collect();

    assert!(
        !field_no_delete_changes.is_empty(),
        "Should detect FIELD_NO_DELETE violation"
    );
    assert!(
        field_no_delete_changes[0].message.contains("age"),
        "Should mention the deleted field name"
    );
    assert!(
        field_no_delete_changes[0].message.contains("2"),
        "Should mention the field number"
    );
}

#[test]
fn test_message_deletion_breaking_change() {
    let old_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
}

message AnotherMessage {
  int32 id = 1;
}
"#;

    let new_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
}
// AnotherMessage deleted - this should be a breaking change
"#;

    let old_spec = Spec::try_from(old_proto).expect("Failed to parse old proto");
    let new_spec = Spec::try_from(new_proto).expect("Failed to parse new proto");

    let result = old_spec.check_breaking_changes(&new_spec);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking change when message is deleted"
    );

    let message_no_delete_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "MESSAGE_NO_DELETE")
        .collect();

    assert!(
        !message_no_delete_changes.is_empty(),
        "Should detect MESSAGE_NO_DELETE violation"
    );
    assert!(
        message_no_delete_changes[0]
            .message
            .contains("AnotherMessage"),
        "Should mention the deleted message name"
    );
}

#[test]
fn test_no_breaking_changes() {
    let old_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
}
"#;

    let new_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  int32 age = 2; // Adding a field is not breaking
}
"#;

    let old_spec = Spec::try_from(old_proto).expect("Failed to parse old proto");
    let new_spec = Spec::try_from(new_proto).expect("Failed to parse new proto");

    let result = old_spec.check_breaking_changes(&new_spec);

    assert!(
        !result.has_breaking_changes,
        "Adding a field should not be a breaking change"
    );
}

#[test]
fn test_field_type_change_breaking() {
    let old_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  int32 age = 2;
}
"#;

    let new_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  string age = 2; // Changed from int32 to string
}
"#;

    let old_spec = Spec::try_from(old_proto).expect("Failed to parse old proto");
    let new_spec = Spec::try_from(new_proto).expect("Failed to parse new proto");

    let result = old_spec.check_breaking_changes(&new_spec);

    assert!(
        result.has_breaking_changes,
        "Should detect breaking change when field type changes"
    );

    let field_type_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "FIELD_SAME_TYPE")
        .collect();

    assert!(
        !field_type_changes.is_empty(),
        "Should detect FIELD_SAME_TYPE violation"
    );
    assert!(
        field_type_changes[0].message.contains("int32"),
        "Should mention old type"
    );
    assert!(
        field_type_changes[0].message.contains("string"),
        "Should mention new type"
    );
}

#[test]
fn test_custom_config() {
    let old_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  int32 age = 2;
}
"#;

    let new_proto = r#"
syntax = "proto3";

message TestMessage {
  string name = 1;
  // age field deleted
}
"#;

    let old_spec = Spec::try_from(old_proto).expect("Failed to parse old proto");
    let new_spec = Spec::try_from(new_proto).expect("Failed to parse new proto");

    // Test with only FILE category
    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        ..Default::default()
    };

    let result = old_spec.check_breaking_changes_with_config(&new_spec, &config);
    assert!(
        result.has_breaking_changes,
        "Should still detect breaking changes with FILE category"
    );

    // Test with excluding the FIELD_NO_DELETE rule
    let config = BreakingConfig {
        use_categories: vec!["FILE".to_string()],
        except_rules: vec!["FIELD_NO_DELETE".to_string()],
        ..Default::default()
    };

    let result = old_spec.check_breaking_changes_with_config(&new_spec, &config);

    // Should not find FIELD_NO_DELETE violations since we excluded it
    let field_no_delete_changes: Vec<_> = result
        .changes
        .iter()
        .filter(|c| c.rule_id == "FIELD_NO_DELETE")
        .collect();

    assert!(
        field_no_delete_changes.is_empty(),
        "Should not detect FIELD_NO_DELETE when excluded"
    );
}
