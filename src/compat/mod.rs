//! Breaking change detection system ported from Buf
//! 
//! This module provides comprehensive breaking change detection for Protocol Buffers,
//! implementing the same rules and logic as the Buf project to ensure compatibility.

pub mod engine;
pub mod categories;
pub mod types;
pub mod handlers;
pub mod bulk_file_rules;
pub mod bulk_field_rules;
pub mod bulk_other_rules;
pub mod bulk_rule_registry;
pub mod bulk_package_rules;
pub mod bulk_extension_rules;
pub mod bulk_enum_rules;
pub mod bulk_message_rules;
pub mod bulk_service_rules;
pub mod bulk_reserved_rules;
pub mod bulk_special_rules;

pub use engine::{BreakingEngine, BreakingConfig, BreakingResult};
pub use types::{BreakingChange, BreakingSeverity, BreakingLocation};
pub use categories::BreakingCategory;
