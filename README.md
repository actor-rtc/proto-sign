# Protobuf Semantic Fingerprint

A Rust library for calculating semantic fingerprints of Protobuf files and checking for backward compatibility.

This library provides two main pieces of functionality:
1.  **Exact Semantic Fingerprint**: A SHA-256 hash that is sensitive to any semantic change in a `.proto` file but insensitive to cosmetic changes like comments, whitespace, or field ordering.
2.  **Compatibility Checker**: A high-level API to compare two versions of a `.proto` file and determine if the change is backward-compatible.

## High-Level API: The `Spec` Checker

For most use cases, the high-level `Spec` API is the recommended way to use this library. It provides a simple way to compare two `.proto` files and get a clear, three-level result.

### Usage

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
        // Adding a new field is backward-compatible.
        message Test {
            string name = 1;
            int32 id = 2;
        }
    "#;

    let new_proto_breaking = r#"
        syntax = "proto3";
        // Changing a field's type is a breaking change.
        message Test {
            int64 name = 1;
        }
    "#;

    let spec_old = Spec::try_from(old_proto).unwrap();
    let spec_compatible = Spec::try_from(new_proto_compatible).unwrap();
    let spec_breaking = Spec::try_from(new_proto_breaking).unwrap();

    // Comparing identical specs -> Green
    assert_eq!(spec_old.compare_with(&spec_old), Compatibility::Green);

    // Comparing with a backward-compatible change -> Yellow
    assert_eq!(spec_old.compare_with(&spec_compatible), Compatibility::Yellow);

    // Comparing with a breaking change -> Red
    assert_eq!(spec_old.compare_with(&spec_breaking), Compatibility::Red);
}
```

### Compatibility Levels

The `compare_with` method returns a `Compatibility` enum with three possible values:

*   `Compatibility::Green`: The two `.proto` files are semantically identical. No functional change.
*   `Compatibility::Yellow`: The new file is backward-compatible with the old file. This typically means new optional fields or messages were added.
*   `Compatibility::Red`: The new file has a breaking change compared to the old one. This means a field or message was removed, or a field had its type or number changed.

## Low-Level APIs

For more advanced use cases, the underlying functions are also public:

*   `proto_sign::generate_fingerprint(content: &str) -> Result<String>`: Calculates the exact semantic fingerprint.
*   `proto_sign::compatibility::get_compatibility_model(content: &str) -> Result<CompatibilityModel>`: Parses the file into a model containing only compatibility-relevant information.
*   `proto_sign::compatibility::is_compatible(old: &CompatibilityModel, new: &CompatibilityModel) -> bool`: Performs the detailed subset comparison to check for backward compatibility.

---
