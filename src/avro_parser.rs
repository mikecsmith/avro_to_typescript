use avro_rs::Schema;
use std::{fs::File, io::Read};

pub fn parse_avro_schema(schema_path: &str) -> Result<Schema, Box<dyn std::error::Error>> {
    let mut file = File::open(schema_path)?;
    let mut schema_json = String::new();

    file.read_to_string(&mut schema_json)?;

    let schema = Schema::parse_str(&schema_json)?;

    Ok(schema)
}
