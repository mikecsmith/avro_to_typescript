mod avro_parser;
mod errors;
mod generate_ts_code;
mod schema_converter;
mod type_converter;
mod types;

use crate::avro_parser::parse_avro_schema;
use crate::schema_converter::convert_avro_schema;
use crate::type_converter::TypeConverter;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    schema: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match parse_avro_schema(&args.schema) {
        Ok(schema) => {
            let converter = TypeConverter;
            let module_item = convert_avro_schema(&schema, &converter);
            let ts_code = generate_ts_code::generate_ts_code(module_item)?;

            println!("{}", ts_code);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
