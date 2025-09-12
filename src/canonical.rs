use serde::Serialize;

// Note: Using BTreeSet for sorted, unique collections.
// This requires `Ord` to be derived.
use std::collections::BTreeSet;

//==============================================================================
// Structs for Exact Semantic Fingerprinting
//==============================================================================

/// Represents the semantically significant content of a .proto file.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub imports: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub messages: BTreeSet<CanonicalMessage>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub enums: BTreeSet<CanonicalEnum>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub services: BTreeSet<CanonicalService>,
}

/// Represents a Protobuf message.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMessage {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub fields: BTreeSet<CanonicalField>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub nested_messages: BTreeSet<CanonicalMessage>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub nested_enums: BTreeSet<CanonicalEnum>,
}

/// Represents a field within a Protobuf message.
/// The sort order is primarily by field number.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CanonicalField {
    pub name: String,
    pub number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub type_name: String,
}

// Custom implementation of Ord for CanonicalField to sort by `number` first.
impl Ord for CanonicalField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for CanonicalField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a Protobuf enum.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalEnum {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub values: BTreeSet<CanonicalEnumValue>,
}

/// Represents a single value within a Protobuf enum.
/// The sort order is primarily by number.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CanonicalEnumValue {
    pub name: String,
    pub number: i32,
}

// Custom implementation of Ord for CanonicalEnumValue to sort by `number` first.
impl Ord for CanonicalEnumValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number
            .cmp(&other.number)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for CanonicalEnumValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a Protobuf service.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalService {
    pub name: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub methods: BTreeSet<CanonicalMethod>,
}

/// Represents a method within a service.
#[derive(Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMethod {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub client_streaming: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub server_streaming: bool,
}
