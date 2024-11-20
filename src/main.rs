mod avro_parser;
mod errors;
mod generate_ts_code;
mod generate_types;
mod generate_zod;
mod types;

use crate::avro_parser::parse_avro_schema;
use crate::generate_types::avro_to_swc_interface;

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
            let module_item = avro_to_swc_interface(&schema);
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
