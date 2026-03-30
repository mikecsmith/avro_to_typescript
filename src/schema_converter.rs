use crate::errors::GenerationError;
use crate::types::GenerationResult;
use avro_rs::{schema::RecordField, Schema};

pub trait SchemaConverter {
    type TypeOutput;

    fn convert_record(&self, name: &str, fields: &[RecordField]) -> GenerationResult;
    fn convert_type(&self, schema: &Schema) -> Self::TypeOutput;
}

pub fn process_nested_records<C: SchemaConverter>(
    schema: &Schema,
    converter: &C,
) -> GenerationResult {
    match schema {
        Schema::Record { name, fields, .. } => converter.convert_record(&name.name, fields),
        Schema::Map(value_schema) => process_nested_records(value_schema, converter),
        Schema::Union(union_schema) => {
            let mut items = Vec::new();
            for variant in union_schema.variants() {
                items.extend(process_nested_records(variant, converter)?);
            }
            Ok(items)
        }
        Schema::Array(item_schema) => process_nested_records(item_schema, converter),
        _ => Ok(Vec::new()),
    }
}

pub fn convert_avro_schema<C: SchemaConverter>(schema: &Schema, converter: &C) -> GenerationResult {
    match schema {
        Schema::Record { name, fields, .. } => converter.convert_record(&name.name, fields),
        _ => Err(GenerationError::Module(format!(
            "Unsupported schema type: {:?}",
            schema
        ))),
    }
}
