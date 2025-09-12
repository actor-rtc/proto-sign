//! Converts the raw `FileDescriptorProto` AST from the `protobuf` crate
//! into a simplified, serializable `CanonicalFile` representation.

use crate::canonical::{
    CanonicalEnum, CanonicalEnumValue, CanonicalField, CanonicalFile, CanonicalMessage,
    CanonicalMethod, CanonicalService,
};
use crate::compatibility::{
    CompatibilityField, CompatibilityMessage, CompatibilityMethod, CompatibilityModel,
    CompatibilityService,
};
use protobuf::descriptor::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, MethodDescriptorProto, ServiceDescriptorProto, field_descriptor_proto,
};

pub fn normalize_file(file: &FileDescriptorProto) -> CanonicalFile {
    let mut canonical_file = CanonicalFile::default();

    canonical_file.package = file.package.clone();

    for import in file.dependency.iter() {
        canonical_file.imports.insert(import.clone());
    }

    for msg in file.message_type.iter() {
        canonical_file.messages.insert(normalize_message(msg));
    }

    for en in file.enum_type.iter() {
        canonical_file.enums.insert(normalize_enum(en));
    }

    for svc in file.service.iter() {
        canonical_file.services.insert(normalize_service(svc));
    }

    canonical_file
}

fn normalize_message(msg: &DescriptorProto) -> CanonicalMessage {
    let mut canonical_msg = CanonicalMessage {
        name: msg.name().to_string(),
        ..Default::default()
    };

    for field in msg.field.iter() {
        canonical_msg.fields.insert(normalize_field(field));
    }

    for nested in msg.nested_type.iter() {
        canonical_msg
            .nested_messages
            .insert(normalize_message(nested));
    }

    for nested_enum in msg.enum_type.iter() {
        canonical_msg
            .nested_enums
            .insert(normalize_enum(nested_enum));
    }

    canonical_msg
}

fn normalize_field(field: &FieldDescriptorProto) -> CanonicalField {
    let label = match field.label() {
        field_descriptor_proto::Label::LABEL_OPTIONAL => "optional",
        field_descriptor_proto::Label::LABEL_REQUIRED => "required",
        field_descriptor_proto::Label::LABEL_REPEATED => "repeated",
    };

    // For primitive types, `type_name` is empty and `type` is set.
    // For message/enum types, `type_name` is set and `type` is TYPE_MESSAGE/TYPE_ENUM.
    let type_name = if field.type_name().is_empty() {
        format!("{:?}", field.type_())
            .to_lowercase()
            .replace("type_", "")
    } else {
        // Keep the fully qualified name for message/enum types.
        field.type_name().to_string()
    };

    CanonicalField {
        name: field.name().to_string(),
        number: field.number(),
        // In proto3, optional is the default and not explicitly stated,
        // so we can consider omitting it for a cleaner canonical form.
        // However, for full semantic representation, it's better to keep it.
        // Let's remove it if it is the default 'optional'.
        label: if label == "optional" {
            None
        } else {
            Some(label.to_string())
        },
        type_name,
    }
}

fn normalize_enum(en: &EnumDescriptorProto) -> CanonicalEnum {
    let mut canonical_enum = CanonicalEnum {
        name: en.name().to_string(),
        ..Default::default()
    };

    for value in en.value.iter() {
        canonical_enum.values.insert(normalize_enum_value(value));
    }

    canonical_enum
}

fn normalize_enum_value(val: &EnumValueDescriptorProto) -> CanonicalEnumValue {
    CanonicalEnumValue {
        name: val.name().to_string(),
        number: val.number(),
    }
}

fn normalize_service(svc: &ServiceDescriptorProto) -> CanonicalService {
    let mut canonical_svc = CanonicalService {
        name: svc.name().to_string(),
        ..Default::default()
    };

    for method in svc.method.iter() {
        canonical_svc.methods.insert(normalize_method(method));
    }

    canonical_svc
}

fn normalize_method(method: &MethodDescriptorProto) -> CanonicalMethod {
    CanonicalMethod {
        name: method.name().to_string(),
        input_type: method.input_type().to_string(),
        output_type: method.output_type().to_string(),
        client_streaming: method.client_streaming(),
        server_streaming: method.server_streaming(),
    }
}

//==============================================================================
// Normalization for Compatibility Fingerprinting
//==============================================================================

pub fn normalize_compatibility_file(file: &FileDescriptorProto) -> CompatibilityModel {
    let mut compat_model = CompatibilityModel::default();

    for msg in file.message_type.iter() {
        compat_model
            .messages
            .insert(normalize_compatibility_message(msg));
    }

    for svc in file.service.iter() {
        compat_model
            .services
            .insert(normalize_compatibility_service(svc));
    }

    compat_model
}

fn normalize_compatibility_message(msg: &DescriptorProto) -> CompatibilityMessage {
    let mut compat_msg = CompatibilityMessage {
        name: msg.name().to_string(),
        ..Default::default()
    };

    for field in msg.field.iter() {
        // For compatibility, we consider the presence of a field number and its type.
        // Removing any field is a breaking change. Changing a name or label is not.
        // Adding a new field is also not a breaking change, which is handled by comparing
        // the set of fields from an old version to a new one.
        compat_msg
            .fields
            .insert(normalize_compatibility_field(field));
    }

    // Note: We are intentionally not descending into nested messages here,
    // as their compatibility is handled when they are defined as top-level messages.

    compat_msg
}

fn normalize_compatibility_field(field: &FieldDescriptorProto) -> CompatibilityField {
    let type_name = if field.type_name().is_empty() {
        format!("{:?}", field.type_())
            .to_lowercase()
            .replace("type_", "")
    } else {
        field.type_name().to_string()
    };

    CompatibilityField {
        number: field.number(),
        type_name,
    }
}

fn normalize_compatibility_service(svc: &ServiceDescriptorProto) -> CompatibilityService {
    let mut compat_svc = CompatibilityService {
        name: svc.name().to_string(),
        ..Default::default()
    };

    for method in svc.method.iter() {
        compat_svc
            .methods
            .insert(normalize_compatibility_method(method));
    }

    compat_svc
}

fn normalize_compatibility_method(method: &MethodDescriptorProto) -> CompatibilityMethod {
    CompatibilityMethod {
        name: method.name().to_string(),
        input_type: method.input_type().to_string(),
        output_type: method.output_type().to_string(),
    }
}
