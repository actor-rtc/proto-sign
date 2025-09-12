//! Provides the high-level Spec API for comparing Protobuf files.

use crate::compatibility::{CompatibilityModel, get_compatibility_model};
use crate::generate_fingerprint;

/// The result of a compatibility comparison between two Protobuf specifications.
#[derive(Debug, PartialEq, Eq)]
pub enum Compatibility {
    /// The two specifications are semantically identical.
    Green,
    /// The new specification is backward-compatible with the old one (e.g., a new field was added).
    Yellow,
    /// The new specification is not backward-compatible with the old one (e.g., a field was removed or changed).
    Red,
}

/// Represents a single Protobuf specification, holding its content and derived models for comparison.
pub struct Spec<'a> {
    /// The original content of the .proto file.
    pub content: &'a str,
    /// The exact semantic fingerprint.
    pub fingerprint: String,
    /// The compatibility model.
    pub compatibility_model: CompatibilityModel,
}

impl<'a> Spec<'a> {
    /// Creates a new `Spec` from the content of a .proto file.
    ///
    /// This function parses the content and generates the necessary fingerprint and models,
    /// so it should be called once per file.
    pub fn try_from(content: &'a str) -> anyhow::Result<Self> {
        let fingerprint = generate_fingerprint(content)?;
        let compatibility_model = get_compatibility_model(content)?;
        Ok(Spec {
            content,
            fingerprint,
            compatibility_model,
        })
    }

    /// Compares this `Spec` (the "old" version) with another `Spec` (the "new" version)
    /// to determine their compatibility level.
    pub fn compare_with(&self, new_spec: &Spec) -> Compatibility {
        // If the exact fingerprints are identical, the files are semantically identical.
        if self.fingerprint == new_spec.fingerprint {
            return Compatibility::Green;
        }

        // If the fingerprints differ, check for backward compatibility.
        if crate::compatibility::is_compatible(
            &self.compatibility_model,
            &new_spec.compatibility_model,
        ) {
            return Compatibility::Yellow;
        }

        // If it's not identical and not compatible, it's a breaking change.
        Compatibility::Red
    }
}
