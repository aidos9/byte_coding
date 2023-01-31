use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::Path;
use syn::{
    spanned::Spanned, Data, DataEnum, DeriveInput, FieldsNamed, FieldsUnnamed, Index, Variant,
};

use crate::byte_coding_attr::{
    ByteCodingAttr, ByteCodingEnumVariantAttr, ByteCodingStructFieldAttr, EnumEncodingType,
};
use crate::parsing::parse_enum_variant_value;

pub fn encoding(input: &DeriveInput) -> TokenStream {
    let (toplevel_attr, first_enum_attr) = match ByteCodingAttr::from_data(input) {
        Ok(v) => v,
        Err(s) => return s,
    };

    let body = match input.data {
        Data::Enum(ref data) => match generate_enum_code(&toplevel_attr, data) {
            Ok(s) => s,
            Err(s) => return s,
        },
        Data::Struct(ref data) => {
            if let Some(attr) = first_enum_attr {
                return quote_spanned! {attr.span()=>
                    compile_error!("Enum argument supplied to attribute on struct.")
                };
            }

            match data.fields {
                syn::Fields::Named(ref fields) => match generate_named_struct_fields_code(fields) {
                    Ok(s) => s,
                    Err(s) => return s,
                },
                syn::Fields::Unnamed(ref fields) => {
                    match generate_unnamed_struct_fields_code(fields) {
                        Ok(s) => s,
                        Err(s) => return s,
                    }
                }
                syn::Fields::Unit => TokenStream::new(),
            }
        }
        _ => panic!("Unsupported data type"),
    };

    let enc_leading = if let Some(f) = toplevel_attr.pre_enc_func {
        let f_name = syn::parse_str::<Path>(&f).unwrap();

        quote! {
            let data = #f_name (self);
        }
    } else {
        quote! {
            let data = self;
        }
    };

    let enc_post = if let Some(f) = toplevel_attr.post_enc_func {
        let f_name = syn::parse_str::<Path>(&f).unwrap();

        quote! {
            #f_name (buf);
        }
    } else {
        TokenStream::new()
    };

    return quote! {
        #enc_leading

        #body

        #enc_post
    };
}

fn encode_literal<T: TryFrom<u128> + ToTokens>(
    value: u128,
    variant: &Variant,
    name: &str,
) -> Result<TokenStream, TokenStream> {
    let v: T = match T::try_from(value) {
        Ok(v) => v,
        Err(_) => {
            return Err(quote_spanned! {variant.span()=>
                compile_error!("Value too large for specified type")
            });
        }
    };

    let tp = format_ident!("{}", name);

    return Ok(quote! { let value: #tp = #v; });
}

fn generate_enum_code(
    toplevel_attr: &ByteCodingAttr,
    data: &DataEnum,
) -> Result<TokenStream, TokenStream> {
    let mut match_branches: Vec<TokenStream> = Vec::new();
    let mut found_values = BTreeSet::new();
    let mut last_value: Option<u128> = None;
    let inferred_values = toplevel_attr.enum_options.is_some()
        && toplevel_attr.enum_options.as_ref().unwrap().inferred_values;

    for variant in &data.variants {
        let variant_attr = ByteCodingEnumVariantAttr::parse_attributes(&variant.attrs)?;
        last_value = if inferred_values {
            Some(match last_value {
                Some(n) => n.wrapping_add(1),
                None => 0,
            })
        } else {
            None
        };

        let value =
            parse_enum_variant_value(last_value, variant, &variant_attr, &mut found_values)?;

        last_value = last_value.map(|_| value);

        let v: u16 = match value.try_into() {
            Ok(v) => v,
            Err(_) => {
                return Err(quote_spanned! {variant.span()=>
                    compile_error!("Value too large for u16")
                });
            }
        };

        let mut literal = quote! { let value: u16 = #v; };

        if let Some(ref opt) = toplevel_attr.enum_options {
            if let Some(tp) = opt.encoding_type {
                literal = match tp {
                    EnumEncodingType::U8 => encode_literal::<u8>(value, variant, "u8")?,
                    EnumEncodingType::U16 => encode_literal::<u16>(value, variant, "u16")?,
                    EnumEncodingType::U32 => encode_literal::<u32>(value, variant, "u32")?,
                    EnumEncodingType::U64 => encode_literal::<u64>(value, variant, "u64")?,
                    EnumEncodingType::U128 => quote! { let value: u128 = #value; },
                };
            }
        }

        let mut rhs = quote! {
            // Encode the indicator value
            #literal
            value.encode_to_buf(buf);
        };

        let mut field_idents = Vec::new();
        let mut is_tuple_variant = None;

        for (i, field) in variant.fields.iter().enumerate() {
            let f_ident;

            if let Some(n) = &field.ident {
                f_ident = n.into_token_stream();

                is_tuple_variant = Some(false);
            } else {
                f_ident = format_ident!("v{}", i).into_token_stream();
                is_tuple_variant = Some(true);
            }

            rhs = quote_spanned! {field.span()=>
                #rhs

                #f_ident.encode_to_buf(buf);
            };

            field_idents.push(f_ident);
        }

        let variant_ident = &variant.ident;
        let lhs;

        if let Some(is_tuple_variant) = is_tuple_variant {
            if is_tuple_variant {
                lhs = quote! {
                    Self::#variant_ident (#(#field_idents),*)
                };
            } else {
                lhs = quote! {
                    Self::#variant_ident {#(#field_idents),*}
                };
            }
        } else {
            lhs = quote! {
                Self::#variant_ident
            };
        }

        match_branches.push(quote_spanned! {variant.span()=>
            #lhs => { #rhs }
        });
    }

    return Ok(quote! {
        match data {
            #(#match_branches),*
        }
    });
}

fn generate_unnamed_struct_fields_code(fields: &FieldsUnnamed) -> Result<TokenStream, TokenStream> {
    let mut field_attribute_pairs = Vec::new();

    for (i, f) in fields.unnamed.iter().enumerate() {
        let field_attr = ByteCodingStructFieldAttr::parse_attributes(&f.attrs)?;

        if field_attr.ignore {
            continue;
        }

        let span = f.span();
        let index = Index::from(i);

        field_attribute_pairs.push((
            field_attr,
            quote_spanned! {span=>
                data.#index.encode_to_buf(buf);
            },
        ));
    }

    field_attribute_pairs.sort_by(|(a, _), (b, _)| a.orderno_cmp(b));

    let recurse = field_attribute_pairs.into_iter().map(|(_, s)| s);

    return Ok(quote! {
        #(#recurse)*
    });
}

fn generate_named_struct_fields_code(fields: &FieldsNamed) -> Result<TokenStream, TokenStream> {
    let mut field_attribute_pairs = Vec::new();

    for f in fields.named.iter() {
        let field_attr = ByteCodingStructFieldAttr::parse_attributes(&f.attrs)?;

        if field_attr.ignore {
            continue;
        }

        let span = f.span();
        let name = &f.ident;

        field_attribute_pairs.push((
            field_attr,
            quote_spanned! {span=>
                data.#name.encode_to_buf(buf);
            },
        ));
    }

    field_attribute_pairs.sort_by(|(a, _), (b, _)| a.orderno_cmp(b));

    let recurse = field_attribute_pairs.into_iter().map(|(_, s)| s);

    return Ok(quote! {
        #(#recurse)*
    });
}
