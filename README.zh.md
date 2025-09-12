# Protobuf 语义化指纹库

一个用于计算 Protobuf 文件语义化指纹并检查向后兼容性的 Rust 库。

本库提供两大核心功能：
1.  **精确语义指纹**: 一个 SHA-256 哈希值，它对 `.proto` 文件中的任何语义变更都敏感，但对注释、空格、字段顺序等非语义化的表面变更不敏感。
2.  **兼容性检查器**: 一个高级 API，用于比较两个版本的 `.proto` 文件，并判断变更是都向后兼容。

## 高级 API: `Spec` 检查器

对于绝大多数使用场景，我们推荐使用高级的 `Spec` API。它提供了一个简洁的方式来比较两个 `.proto` 文件，并给出一个清晰的三色等级结果。

### 使用方法

```rust
use proto_sign::spec::{Spec, Compatibility};

fn main() {
    let old_proto = r#"
        syntax = "proto3";
        message Test {
            string name = 1;
        }
    "#;

    let new_proto_compatible = r#"
        syntax = "proto3";
        // 增加一个新字段是向后兼容的
        message Test {
            string name = 1;
            int32 id = 2;
        }
    "#;

    let new_proto_breaking = r#"
        syntax = "proto3";
        // 修改字段类型是破坏性变更
        message Test {
            int64 name = 1;
        }
    "#;

    let spec_old = Spec::try_from(old_proto).unwrap();
    let spec_compatible = Spec::try_from(new_proto_compatible).unwrap();
    let spec_breaking = Spec::try_from(new_proto_breaking).unwrap();

    // 比较两个完全相同的规约 -> Green
    assert_eq!(spec_old.compare_with(&spec_old), Compatibility::Green);

    // 比较一个向后兼容的变更 -> Yellow
    assert_eq!(spec_old.compare_with(&spec_compatible), Compatibility::Yellow);

    // 比较一个破坏性变更 -> Red
    assert_eq!(spec_old.compare_with(&spec_breaking), Compatibility::Red);
}
```

### 兼容性等级

`compare_with` 方法返回一个 `Compatibility` 枚举，它有三个可能的值：

*   `Compatibility::Green`: 两个 `.proto` 文件在语义上完全相同。无功能性变更。
*   `Compatibility::Yellow`: 新文件向后兼容旧文件。这通常意味着增加了新的可选字段或消息。
*   `Compatibility::Red`: 新文件相比旧文件存在破坏性变更。这意味着有字段或消息被移除，或者某个字段的类型或编号被修改。

## 底层 API

对于更高级的用例，底层的核心函数也是公开的：

*   `proto_sign::generate_fingerprint(content: &str) -> Result<String>`: 计算精确的语义指纹。
*   `proto_sign::compatibility::get_compatibility_model(content: &str) -> Result<CompatibilityModel>`: 将文件解析为一个只包含兼容性相关信息的模型。
*   `proto_sign::compatibility::is_compatible(old: &CompatibilityModel, new: &CompatibilityModel) -> bool`: 执行详细的子集比较以检查向后兼容性。

---
