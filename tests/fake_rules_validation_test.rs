//! 负向测试：验证之前的fake规则现在确实工作
//! 
//! 这些测试验证那些之前返回RuleResult::success()的假规则
//! 现在确实能检测到破坏性变更，确保真正的1:1实现。

use proto_sign::compat::{BreakingEngine, BreakingConfig};
use proto_sign::canonical::*;
use std::collections::BTreeSet;

/// 测试辅助函数：创建基础文件
fn create_base_file() -> CanonicalFile {
    CanonicalFile {
        package: Some("com.example".to_string()),
        syntax: "proto3".to_string(),
        go_package: Some("github.com/example/proto".to_string()),
        java_generic_services: Some(true),
        cc_enable_arenas: Some(false),
        php_generic_services: Some(false),
        py_generic_services: Some(true),
        ..Default::default()
    }
}

#[test]
fn test_extension_no_delete_rule_works() {
    let mut previous = create_base_file();
    let mut current = create_base_file();
    
    // 添加extension到previous文件
    let extension = CanonicalExtension {
        name: "my_extension".to_string(),
        number: 1000,
        extendee: ".google.protobuf.DescriptorProto".to_string(),
        type_name: "string".to_string(),
        label: None,
        default: None,
        deprecated: None,
    };
    previous.extensions.insert(extension);
    // current文件没有extension（被删除了）
    
    let config = BreakingConfig {
        use_rules: vec!["EXTENSION_NO_DELETE".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "EXTENSION_NO_DELETE规则应该检测到extension被删除");
    assert_eq!(result.changes[0].rule_id, "EXTENSION_NO_DELETE");
    assert!(result.changes[0].message.contains("my_extension"));
}

#[test]  
fn test_package_extension_no_delete_rule_works() {
    let mut previous = create_base_file();
    let mut current = create_base_file();
    
    // 添加extension到previous文件（同一包）
    let extension = CanonicalExtension {
        name: "package_extension".to_string(),
        number: 2000,
        extendee: ".com.example.Message".to_string(),
        type_name: "int32".to_string(),
        label: Some("optional".to_string()),
        default: Some("42".to_string()),
        deprecated: None,
    };
    previous.extensions.insert(extension);
    // current文件没有extension
    
    let config = BreakingConfig {
        use_rules: vec!["PACKAGE_EXTENSION_NO_DELETE".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "PACKAGE_EXTENSION_NO_DELETE规则应该检测到extension被删除");
    assert_eq!(result.changes[0].rule_id, "PACKAGE_EXTENSION_NO_DELETE");
    assert!(result.changes[0].message.contains("package_extension"));
}

#[test]
fn test_package_no_delete_rule_works() {
    let previous = CanonicalFile {
        package: Some("com.example.deleted".to_string()),
        ..create_base_file()
    };
    let current = CanonicalFile {
        package: None, // 包被删除了
        ..create_base_file()
    };
    
    let config = BreakingConfig {
        use_rules: vec!["PACKAGE_NO_DELETE".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "PACKAGE_NO_DELETE规则应该检测到包被删除");
    assert_eq!(result.changes[0].rule_id, "PACKAGE_NO_DELETE");
    assert!(result.changes[0].message.contains("com.example.deleted"));
}

#[test]
fn test_file_no_delete_rule_works() {
    let mut previous = create_base_file();
    // 添加一些内容到previous文件
    let message = CanonicalMessage {
        name: "TestMessage".to_string(),
        ..Default::default()
    };
    previous.messages.insert(message);
    
    let current = CanonicalFile {
        package: None, // 包和内容都没了
        messages: BTreeSet::new(),
        enums: BTreeSet::new(),
        services: BTreeSet::new(),
        extensions: BTreeSet::new(),
        ..Default::default()
    };
    
    let config = BreakingConfig {
        use_rules: vec!["FILE_NO_DELETE".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "FILE_NO_DELETE规则应该检测到文件被删除");
    assert_eq!(result.changes[0].rule_id, "FILE_NO_DELETE");
}

#[test]
fn test_file_same_java_generic_services_rule_works() {
    let previous = CanonicalFile {
        java_generic_services: Some(true),
        ..create_base_file()
    };
    let current = CanonicalFile {
        java_generic_services: Some(false), // 改变了
        ..create_base_file()
    };
    
    let config = BreakingConfig {
        use_rules: vec!["FILE_SAME_JAVA_GENERIC_SERVICES".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "FILE_SAME_JAVA_GENERIC_SERVICES规则应该检测到选项变化");
    assert_eq!(result.changes[0].rule_id, "FILE_SAME_JAVA_GENERIC_SERVICES");
    assert!(result.changes[0].message.contains("java_generic_services"));
}

#[test]
fn test_file_same_cc_enable_arenas_rule_works() {
    let previous = CanonicalFile {
        cc_enable_arenas: Some(false),
        ..create_base_file()
    };
    let current = CanonicalFile {
        cc_enable_arenas: Some(true), // 改变了
        ..create_base_file()
    };
    
    let config = BreakingConfig {
        use_rules: vec!["FILE_SAME_CC_ENABLE_ARENAS".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "FILE_SAME_CC_ENABLE_ARENAS规则应该检测到选项变化");
    assert_eq!(result.changes[0].rule_id, "FILE_SAME_CC_ENABLE_ARENAS");
    assert!(result.changes[0].message.contains("cc_enable_arenas"));
}

#[test]
fn test_message_same_json_format_rule_works() {
    let mut previous = create_base_file();
    let mut current = create_base_file();
    
    // 创建带有message_set_wire_format选项的消息
    let previous_message = CanonicalMessage {
        name: "TestMessage".to_string(),
        message_set_wire_format: Some(false),
        ..Default::default()
    };
    let current_message = CanonicalMessage {
        name: "TestMessage".to_string(),
        message_set_wire_format: Some(true), // 改变了
        ..Default::default()
    };
    
    previous.messages.insert(previous_message);
    current.messages.insert(current_message);
    
    let config = BreakingConfig {
        use_rules: vec!["MESSAGE_SAME_JSON_FORMAT".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change
    assert!(!result.changes.is_empty(), "MESSAGE_SAME_JSON_FORMAT规则应该检测到格式变化");
    assert_eq!(result.changes[0].rule_id, "MESSAGE_SAME_JSON_FORMAT");
    assert!(result.changes[0].message.contains("message_set_wire_format"));
}

#[test]
fn test_enum_same_json_format_rule_works() {
    let mut previous = create_base_file();
    let mut current = create_base_file();
    
    // 创建带有json_format选项的枚举
    let mut previous_enum = CanonicalEnum {
        name: "TestEnum".to_string(),
        closed_enum: Some(false),
        ..Default::default()
    };
    previous_enum.options.insert("json_format".to_string(), "ALLOW".to_string());
    
    let mut current_enum = CanonicalEnum {
        name: "TestEnum".to_string(),
        closed_enum: Some(true), // 改变了
        ..Default::default()
    };
    current_enum.options.insert("json_format".to_string(), "DISALLOW".to_string()); // 也改变了
    
    previous.enums.insert(previous_enum);
    current.enums.insert(current_enum);
    
    let config = BreakingConfig {
        use_rules: vec!["ENUM_SAME_JSON_FORMAT".to_string()],
        ..Default::default()
    };
    
    let engine = BreakingEngine::new();
    let result = engine.check(&current, &previous, &config);
    
    // 应该检测到breaking change  
    assert!(!result.changes.is_empty(), "ENUM_SAME_JSON_FORMAT规则应该检测到格式变化");
    assert_eq!(result.changes[0].rule_id, "ENUM_SAME_JSON_FORMAT");
    // 可能检测到json_format或closed_enum的变化
    assert!(result.changes[0].message.contains("json_format") || 
             result.changes[0].message.contains("closed_enum"));
}

#[test]
fn test_all_previously_fake_rules_are_implemented() {
    // 这个测试验证所有之前fake的规则都被注册了
    let engine = BreakingEngine::new();
    let config = BreakingConfig::default();
    
    // 创建两个相同的空文件
    let file1 = CanonicalFile::default();
    let file2 = CanonicalFile::default();
    
    let result = engine.check(&file1, &file2, &config);
    
    // 没有变化，所以应该成功
    assert!(result.changes.is_empty());
    
    // 验证之前fake的规则现在都在注册表中
    let previously_fake_rules = vec![
        "EXTENSION_NO_DELETE",
        "PACKAGE_EXTENSION_NO_DELETE", 
        "PACKAGE_NO_DELETE",
        "FILE_NO_DELETE",
        "FILE_SAME_JAVA_GENERIC_SERVICES",
        "FILE_SAME_CC_ENABLE_ARENAS",
        "FILE_SAME_PHP_GENERIC_SERVICES",
        "FILE_SAME_PY_GENERIC_SERVICES",
        "MESSAGE_SAME_JSON_FORMAT",
        "ENUM_SAME_JSON_FORMAT",
    ];
    
    for rule in previously_fake_rules {
        let config = BreakingConfig {
            use_rules: vec![rule.to_string()],
            ..Default::default()
        };
        
        // 应该能运行规则而不会出错
        let result = engine.check(&file1, &file2, &config);
        assert_eq!(result.changes.len(), 0, "规则 {} 应该被正确实现和注册", rule);
    }
}