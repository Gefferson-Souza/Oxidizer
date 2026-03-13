use swc_ecma_ast::{TsKeywordTypeKind, TsType, TsTypeAnn};

use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// Lower a SWC `TsTypeAnn` to `TyrusType`
pub fn lower_ts_type(ann: Option<&TsTypeAnn>) -> TyrusType {
    let Some(ann) = ann else {
        return TyrusType::Inferred;
    };
    lower_ts_type_inner(&ann.type_ann)
}

fn lower_ts_type_inner(ts: &TsType) -> TyrusType {
    match ts {
        TsType::TsKeywordType(kw) => lower_keyword(kw.kind),
        TsType::TsArrayType(arr) => {
            let elem = lower_ts_type_inner(&arr.elem_type);
            TyrusType::Array(Box::new(elem))
        }
        TsType::TsTypeRef(ref_type) => lower_type_ref(ref_type),
        TsType::TsUnionOrIntersectionType(
            swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
        ) => lower_union(union),
        _ => TyrusType::Inferred,
    }
}

fn lower_keyword(kind: TsKeywordTypeKind) -> TyrusType {
    match kind {
        TsKeywordTypeKind::TsStringKeyword => TyrusType::String,
        TsKeywordTypeKind::TsNumberKeyword => TyrusType::Number,
        TsKeywordTypeKind::TsBooleanKeyword => TyrusType::Boolean,
        TsKeywordTypeKind::TsVoidKeyword => TyrusType::Void,
        _ => TyrusType::Inferred,
    }
}

fn lower_type_ref(ref_type: &swc_ecma_ast::TsTypeRef) -> TyrusType {
    let name = if let Some(ident) = ref_type.type_name.as_ident() {
        ident.sym.to_string()
    } else {
        return TyrusType::Inferred;
    };

    let type_args: Vec<TyrusType> = ref_type
        .type_params
        .as_ref()
        .map(|params| {
            params
                .params
                .iter()
                .map(|p| lower_ts_type_inner(p))
                .collect()
        })
        .unwrap_or_default();

    match name.as_str() {
        "Array" => {
            let elem = type_args.into_iter().next().unwrap_or(TyrusType::Inferred);
            TyrusType::Array(Box::new(elem))
        }
        "Promise" => {
            let elem = type_args.into_iter().next().unwrap_or(TyrusType::Void);
            TyrusType::Promise(Box::new(elem))
        }
        "Record" => {
            let mut iter = type_args.into_iter();
            let key = iter.next().unwrap_or(TyrusType::String);
            let val = iter.next().unwrap_or(TyrusType::Inferred);
            TyrusType::Map(Box::new(key), Box::new(val))
        }
        other => {
            let ident = Ident::new(other, TyrusSpan::dummy());
            if type_args.is_empty() {
                TyrusType::Named(ident)
            } else {
                TyrusType::Generic(ident, type_args)
            }
        }
    }
}

fn lower_union(union: &swc_ecma_ast::TsUnionType) -> TyrusType {
    let non_undefined: Vec<_> = union
        .types
        .iter()
        .filter(|t| {
            !matches!(
                &***t,
                TsType::TsKeywordType(kw) if kw.kind == TsKeywordTypeKind::TsUndefinedKeyword
            )
        })
        .collect();

    if non_undefined.len() == 1 && non_undefined.len() < union.types.len() {
        let inner = lower_ts_type_inner(non_undefined[0]);
        TyrusType::Option(Box::new(inner))
    } else {
        TyrusType::Inferred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_none_is_inferred() {
        let result = lower_ts_type(None);
        assert_eq!(result, TyrusType::Inferred);
    }

    #[test]
    fn test_lower_keyword_string() {
        assert_eq!(
            lower_keyword(TsKeywordTypeKind::TsStringKeyword),
            TyrusType::String,
        );
    }

    #[test]
    fn test_lower_keyword_number() {
        assert_eq!(
            lower_keyword(TsKeywordTypeKind::TsNumberKeyword),
            TyrusType::Number,
        );
    }

    #[test]
    fn test_lower_keyword_boolean() {
        assert_eq!(
            lower_keyword(TsKeywordTypeKind::TsBooleanKeyword),
            TyrusType::Boolean,
        );
    }

    #[test]
    fn test_lower_keyword_void() {
        assert_eq!(
            lower_keyword(TsKeywordTypeKind::TsVoidKeyword),
            TyrusType::Void,
        );
    }

    #[test]
    fn test_lower_keyword_unknown_is_inferred() {
        assert_eq!(
            lower_keyword(TsKeywordTypeKind::TsAnyKeyword),
            TyrusType::Inferred,
        );
    }
}
