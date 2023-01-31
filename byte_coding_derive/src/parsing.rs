use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, Variant};

use crate::byte_coding_attr::ByteCodingEnumVariantAttr;

pub fn u128_to_int_tok_stream<T: TryFrom<u128> + ToTokens>(
    value: u128,
    variant: &Variant,
) -> Result<TokenStream, TokenStream> {
    let v: T = match value.try_into() {
        Ok(v) => v,
        Err(_) => {
            return Err(quote_spanned! {variant.span()=>
                compile_error!("Value too large for u16")
            });
        }
    };

    return Ok(v.into_token_stream());
}

fn expr_to_u128(expr: &Expr) -> Result<u128, TokenStream> {
    return match expr {
        Expr::Lit(e) => match &e.lit {
            syn::Lit::Int(i) => i.base10_parse().map_err(|_| {
                quote_spanned! {e.lit.span()=>
                    compile_error!("Unsupported expression. Base 10 Integer literals only.")
                }
            }),
            _ => Err(quote_spanned! {e.lit.span()=>
                compile_error!("Unsupported expression. Integer literals only.")
            }),
        },
        _ => Err(quote_spanned! {expr.span()=>
            compile_error!("Unsupported expression. Integer literals only.")
        }),
    };
}

pub fn parse_enum_variant_value(
    default_value: Option<u128>,
    variant: &Variant,
    variant_attr: &ByteCodingEnumVariantAttr,
    found_values: &mut BTreeSet<u128>,
) -> Result<u128, TokenStream> {
    if variant.discriminant.is_none() && variant_attr.value.is_none() && default_value.is_none() {
        return Err(quote_spanned! {variant.span()=>
            compile_error!("No discriminant or value provided")
        });
    }

    let value;

    if let Some(v) = variant_attr.value {
        value = v;
    } else if variant.discriminant.is_some() {
        value = expr_to_u128(&variant.discriminant.as_ref().unwrap().1)?;
    } else {
        value = default_value.unwrap();
    }

    if found_values.contains(&value) {
        return Err(quote_spanned! {variant.span() =>
            compile_error!("2 or more variants share the same value.");
        });
    } else {
        found_values.insert(value);
    }

    return Ok(value);
}
