use crate::schema_converter::{process_nested_records, SchemaConverter};
use crate::types::GenerationResult;
use avro_rs::Schema;
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

pub struct TypeConverter;

impl TypeConverter {
    /// Checks if a field schema is a nullable union (i.e. contains `null`).
    /// Returns `(is_optional, type)` where the type has `null` stripped from the union.
    /// For example, `["null", "string"]` returns `(true, string)` and
    /// `["null", "string", "int"]` returns `(true, string | number)`.
    fn convert_field_type(&self, schema: &Schema) -> (bool, TsType) {
        if let Schema::Union(union_schema) = schema {
            let variants: Vec<&Schema> = union_schema.variants().iter().collect();
            let has_null = variants.iter().any(|v| matches!(v, Schema::Null));

            if has_null {
                let non_null: Vec<&Schema> = variants
                    .into_iter()
                    .filter(|v| !matches!(v, Schema::Null))
                    .collect();

                let ts_type = if non_null.len() == 1 {
                    self.convert_type(non_null[0])
                } else {
                    TsType::TsUnionOrIntersectionType(
                        TsUnionOrIntersectionType::TsUnionType(TsUnionType {
                            span: DUMMY_SP,
                            types: non_null
                                .iter()
                                .map(|v| Box::new(self.convert_type(v)))
                                .collect(),
                        }),
                    )
                };

                return (true, ts_type);
            }
        }

        (false, self.convert_type(schema))
    }
}

impl SchemaConverter for TypeConverter {
    type TypeOutput = TsType;
    fn convert_record(
        &self,
        name: &str,
        fields: &[avro_rs::schema::RecordField],
    ) -> GenerationResult {
        let mut nested_items = Vec::new();
        for field in fields {
            nested_items.extend(process_nested_records(&field.schema, self)?);
        }

        let properties = fields
            .iter()
            .map(|field| {
                let (is_optional, ts_type) = self.convert_field_type(&field.schema);

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
                    optional: is_optional,
                    type_ann: Some(Box::new(TsTypeAnn {
                        span: DUMMY_SP,
                        type_ann: Box::new(ts_type),
                    })),
                })
            })
            .collect();

        let interface_decl = TsInterfaceDecl {
            span: DUMMY_SP,
            id: Ident {
                span: DUMMY_SP,
                sym: name.into(),
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

    fn convert_type(&self, schema: &Schema) -> TsType {
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
                elem_type: Box::new(self.convert_type(item_schema)),
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
                        Box::new(self.convert_type(value_schema)),
                    ],
                })),
            }),
            Schema::Union(union_schema) => TsType::TsUnionOrIntersectionType(
                TsUnionOrIntersectionType::TsUnionType(TsUnionType {
                    span: DUMMY_SP,
                    types: union_schema
                        .variants()
                        .iter()
                        .map(|variant| Box::new(self.convert_type(variant)))
                        .collect(),
                }),
            ),
            Schema::Record { name, .. } => TsType::TsTypeRef(TsTypeRef {
                span: DUMMY_SP,
                type_name: TsEntityName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: name.name.clone().into(),
                    ctxt: SyntaxContext::empty(),
                    optional: false,
                }),
                type_params: None,
            }),
            Schema::Enum { symbols, .. } => TsType::TsUnionOrIntersectionType(
                TsUnionOrIntersectionType::TsUnionType(TsUnionType {
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
                }),
            ),
            Schema::Fixed { .. } => TsType::TsTypeRef(TsTypeRef {
                span: DUMMY_SP,
                type_name: TsEntityName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "Uint8Array".into(),
                    ctxt: SyntaxContext::empty(),
                    optional: false,
                }),
                type_params: None,
            }),
            Schema::Decimal { .. } => TsType::TsKeywordType(TsKeywordType {
                span: DUMMY_SP,
                kind: TsKeywordTypeKind::TsStringKeyword,
            }),
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
}
