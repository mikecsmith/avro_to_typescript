use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

use crate::errors::GenerationError;
use crate::types::GenerationResult;

use avro_rs::Schema;

fn process_nested_schema(schema: &Schema) -> GenerationResult {
    match schema {
        Schema::Record { .. } => avro_to_swc_interface(schema),
        Schema::Map(value_schema) => {
            if let Schema::Record { .. } = **value_schema {
                avro_to_swc_interface(value_schema)
            } else {
                Ok(Vec::new())
            }
        }
        Schema::Union(union_schema) => {
            let mut items = Vec::new();
            for variant in union_schema.variants() {
                items.extend(avro_to_swc_interface(variant)?);
            }
            Ok(items)
        }
        Schema::Array(item_schema) => {
            if let Schema::Record { .. } = **item_schema {
                avro_to_swc_interface(item_schema)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

pub fn avro_to_swc_interface(schema: &Schema) -> GenerationResult {
    match schema {
        Schema::Record { name, fields, .. } => {
            let mut nested_items = Vec::new();
            for field in fields {
                nested_items.extend(process_nested_schema(&field.schema)?);
            }

            let properties = fields
                .iter()
                .map(|field| {
                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        span: DUMMY_SP,
                        readonly: false,
                        key: Box::new(Expr::Ident(Ident {
                            span: DUMMY_SP,
                            sym: field.name.clone().into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        })),
                        computed: false,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: DUMMY_SP,
                            type_ann: Box::new(avro_to_swc_type(&field.schema)),
                        })),
                    })
                })
                .collect();

            let interface_decl = TsInterfaceDecl {
                span: DUMMY_SP,
                id: Ident {
                    span: DUMMY_SP,
                    sym: name.name.clone().into(),
                    ctxt: SyntaxContext::empty(),
                    optional: false,
                },
                declare: false,
                type_params: None,
                extends: vec![],
                body: TsInterfaceBody {
                    span: DUMMY_SP,
                    body: properties,
                },
            };

            let all_items = [
                nested_items,
                vec![ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                    span: DUMMY_SP,
                    decl: Decl::TsInterface(Box::new(interface_decl)),
                }))],
            ]
            .concat();

            Ok(all_items)
        }
        _ => Err(GenerationError::Module(format!(
            "Unsupported schema type: {:?}",
            schema
        ))),
    }
}

fn avro_to_swc_type(schema: &Schema) -> TsType {
    match schema {
        Schema::Null => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNullKeyword,
        }),
        Schema::Boolean => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        }),
        Schema::Int | Schema::Long | Schema::Float | Schema::Double => {
            TsType::TsKeywordType(TsKeywordType {
                span: DUMMY_SP,
                kind: TsKeywordTypeKind::TsNumberKeyword,
            })
        }
        Schema::Bytes => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Uint8Array".into(),
                ctxt: SyntaxContext::empty(),
                optional: false,
            }),
            type_params: None,
        }),
        Schema::String => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        Schema::Array(item_schema) => TsType::TsArrayType(TsArrayType {
            span: DUMMY_SP,
            elem_type: Box::new(avro_to_swc_type(item_schema)),
        }),
        Schema::Map(value_schema) => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Record".into(),
                ctxt: SyntaxContext::empty(),
                optional: false,
            }),
            type_params: Some(Box::new(TsTypeParamInstantiation {
                span: DUMMY_SP,
                params: vec![
                    Box::new(TsType::TsKeywordType(TsKeywordType {
                        span: DUMMY_SP,
                        kind: TsKeywordTypeKind::TsStringKeyword,
                    })),
                    Box::new(avro_to_swc_type(value_schema)),
                ],
            })),
        }),
        Schema::Union(union_schema) => {
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(TsUnionType {
                span: DUMMY_SP,
                types: union_schema
                    .variants()
                    .iter()
                    .map(|variant| Box::new(avro_to_swc_type(variant)))
                    .collect(),
            }))
        }
        Schema::Record { name, .. } => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident {
                span: DUMMY_SP,
                sym: name.name.clone().into(),
                ctxt: SyntaxContext::empty(),
                optional: false,
            }),
            type_params: None, // No generics for now
        }),
        Schema::Enum { symbols, .. } => {
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(TsUnionType {
                span: DUMMY_SP,
                types: symbols
                    .iter()
                    .map(|symbol| {
                        Box::new(TsType::TsLitType(TsLitType {
                            span: DUMMY_SP,
                            lit: TsLit::Str(Str {
                                span: DUMMY_SP,
                                value: symbol.clone().into(),
                                raw: None,
                            }),
                        }))
                    })
                    .collect(),
            }))
        }
        Schema::Fixed { .. } => {
            // Use `Uint8Array` for fixed-size binary data
            TsType::TsTypeRef(TsTypeRef {
                span: DUMMY_SP,
                type_name: TsEntityName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "Uint8Array".into(),
                    ctxt: SyntaxContext::empty(),
                    optional: false,
                }),
                type_params: None, // No type parameters for Uint8Array
            })
        }
        Schema::Decimal { .. } => {
            TsType::TsKeywordType(TsKeywordType {
                span: DUMMY_SP,
                kind: TsKeywordTypeKind::TsStringKeyword, // Default to string representation
            })
        }
        Schema::Uuid
        | Schema::Date
        | Schema::Duration
        | Schema::TimeMillis
        | Schema::TimeMicros => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),

        Schema::TimestampMillis | Schema::TimestampMicros => TsType::TsTypeRef(TsTypeRef {
            span: DUMMY_SP,
            type_name: TsEntityName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Date".into(),
                ctxt: SyntaxContext::empty(),
                optional: false,
            }),
            type_params: None,
        }),
    }
}
