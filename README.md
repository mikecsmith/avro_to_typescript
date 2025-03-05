# Avro to TypeScript Converter

This proof-of-concept project provides a command-line tool to convert Avro schemas to TypeScript types.

## Installation

To build the project, you need to have [Rust](https://www.rust-lang.org/) installed. Clone the repository and run:

```sh
cargo install --path .
```

## Usage

To use the tool, run the following command:

```sh
att --schema /path/to/avro/schema.json > /path/to/output/file.ts
```

If you don't redirect `stdout` to a file then the generated code will be printed to the screen.

## Example

```sh
att --schema ./example.json > ~/projects/my-project/src/types/example.ts
```

This will parse the `example.json` schema, convert it to TypeScript, and write the generated typescript code to `example.ts`.

## Modules

- `avro_parser`: Contains functions to parse Avro schemas.
- `errors`: Defines custom error types.
- `generate_ts_code`: Contains functions to generate TypeScript code from Avro schemas.
- `schema_converter`: Contains functions to convert Avro schemas to an intermediate representation.
- `type_converter`: Contains type conversion logic.
- `types`: Defines types used throughout the project.
