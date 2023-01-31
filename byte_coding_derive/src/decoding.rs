use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Index, Path};

use crate::byte_coding_attr::{
    ByteCodingAttr, ByteCodingEnumVariantAttr, ByteCodingStructFieldAttr, EnumEncodingType,
};
use crate::parsing::{parse_enum_variant_value, u128_to_int_tok_stream};

pub fn decoding(input: &DeriveInput) -> TokenStream {
    let (toplevel_attr, first_enum_attr) = match ByteCodingAttr::from_data(input) {
        Ok(v) => v,
        Err(s) => return s,
    };

    let body = match &input.data {
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
                Fields::Named(ref fields) => match generate_named_struct_fields_code(fields) {
                    Ok(s) => s,
                    Err(s) => return s,
                },
                Fields::Unnamed(ref fields) => match generate_unnamed_struct_fields_code(fields) {
                    Ok(s) => s,
                    Err(s) => return s,
                },
                Fields::Unit => quote! { let decoded_res = Self; },
            }
        }
        _ => panic!("Unsupported data type"),
    };

    let pre_dec_func = if let Some(f) = toplevel_attr.pre_dec_func {
        let f_name = syn::parse_str::<Path>(&f).unwrap();

        quote! {
            buffer = #f_name (buffer)?;
        }
    } else {
        TokenStream::new()
    };

    let post_dec_func = if let Some(f) = toplevel_attr.post_dec_func {
        let f_name = syn::parse_str::<Path>(&f).unwrap();

        quote! {
            let r = #f_name (decoded_res, buffer)?;
            let res = r.0;
            buffer = r.1;
        }
    } else {
        quote! {
            let res = decoded_res;
        }
    };

    return quote! {
        #pre_dec_func

        #body

        #post_dec_func

        return Some((res, buffer));
    };
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

    let mut value_parse = quote! { let res: (u16, &[u8]) = Decodable::decode_from_buf(buffer)?; };

    if let Some(ref opt) = toplevel_attr.enum_options {
        if let Some(tp) = opt.encoding_type {
            value_parse = match tp {
                EnumEncodingType::U8 => {
                    quote! { let res: (u8, &[u8]) = Decodable::decode_from_buf(buffer)?; }
                }
                EnumEncodingType::U16 => {
                    quote! { let res: (u16, &[u8]) = Decodable::decode_from_buf(buffer)?; }
                }
                EnumEncodingType::U32 => {
                    quote! { let res: (u32, &[u8]) = Decodable::decode_from_buf(buffer)?; }
                }
                EnumEncodingType::U64 => {
                    quote! { let res: (u64, &[u8]) = Decodable::decode_from_buf(buffer)?; }
                }
                EnumEncodingType::U128 => {
                    quote! { let res: (u128, &[u8]) = Decodable::decode_from_buf(buffer)?; }
                }
            };
        }
    }

    let value_parse = quote! {
       #value_parse

       let variant_value = res.0;
       buffer = res.1;
    };

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

        let mut v = None;

        if let Some(ref opt) = toplevel_attr.enum_options {
            if let Some(tp) = opt.encoding_type {
                v = Some(match tp {
                    EnumEncodingType::U8 => u128_to_int_tok_stream::<u8>(value, variant)?,
                    EnumEncodingType::U16 => u128_to_int_tok_stream::<u16>(value, variant)?,
                    EnumEncodingType::U32 => u128_to_int_tok_stream::<u32>(value, variant)?,
                    EnumEncodingType::U64 => u128_to_int_tok_stream::<u64>(value, variant)?,
                    EnumEncodingType::U128 => u128_to_int_tok_stream::<u128>(value, variant)?,
                });
            }
        }

        let v = v.unwrap_or((value as u16).into_token_stream());

        let mut var_names = Vec::new();
        let mut is_tuple_variant = None;

        let mut rhs = TokenStream::new();

        for (i, field) in variant.fields.iter().enumerate() {
            let f_ident;

            if let Some(n) = &field.ident {
                f_ident = n.into_token_stream();

                is_tuple_variant = Some(false);
            } else {
                f_ident = format_ident!("v{}", i).into_token_stream();
                is_tuple_variant = Some(true);
            }

            let ty = &field.ty;

            rhs = quote_spanned! {field.span()=>
                #rhs

                let res = Decodable::decode_from_buf(buffer)?;
                let #f_ident: #ty = res.0;
                buffer = res.1;
            };

            var_names.push(f_ident);
        }

        let variant_ident = &variant.ident;

        if let Some(is_tuple_variant) = is_tuple_variant {
            if is_tuple_variant {
                rhs = quote! {
                    #rhs

                    Self::#variant_ident (#(#var_names),*)
                };
            } else {
                rhs = quote! {
                    #rhs

                    Self::#variant_ident {
                        #(#var_names),*
                    }
                };
            }
        } else {
            rhs = quote! {
                Self::#variant_ident
            };
        }

        match_branches.push(quote_spanned! {variant.span()=>
            #v => { #rhs }
        });
    }

    return Ok(quote! {
        #value_parse

        let decoded_res = match variant_value {
            #(#match_branches),*
            _ => return None,
        };
    });
}

fn generate_unnamed_struct_fields_code(fields: &FieldsUnnamed) -> Result<TokenStream, TokenStream> {
    let mut field_attribute_pairs = Vec::new();

    for (i, f) in fields.unnamed.iter().enumerate() {
        let field_attr = ByteCodingStructFieldAttr::parse_attributes(&f.attrs)?;

        let span = f.span();
        let index = Index::from(i);

        let res_name = format_ident!("_res_{}", index);
        let name = format_ident!("_{}", index);

        if field_attr.ignore {
            field_attribute_pairs.push((
                field_attr,
                quote_spanned! {span=>
                    let #name = Default::default();
                },
                quote_spanned! {span=>
                    #name
                },
            ));
        } else {
            field_attribute_pairs.push((
                field_attr,
                quote_spanned! {span=>
                    let #res_name = Decodable::decode_from_buf(buffer)?;
                    let #name = #res_name.0;
                    buffer = #res_name.1;
                },
                quote_spanned! {span=>
                    #name
                },
            ));
        }
    }

    field_attribute_pairs.sort_by(|(a, _, _), (b, _, _)| a.orderno_cmp(b));

    let recurse = field_attribute_pairs.iter().map(|(_, s, _)| s);
    let fields = field_attribute_pairs.iter().map(|(_, _, v)| v);

    return Ok(quote! {
        #(#recurse)*

        let decoded_res = Self (
            #(#fields),*
        );
    });
}

fn generate_named_struct_fields_code(fields: &FieldsNamed) -> Result<TokenStream, TokenStream> {
    let mut field_attribute_pairs = Vec::new();

    for f in fields.named.iter() {
        let field_attr = ByteCodingStructFieldAttr::parse_attributes(&f.attrs)?;

        let span = f.span();

        let name = &f.ident;
        let res_name = format_ident!("_{}", name.as_ref().unwrap());

        if field_attr.ignore {
            field_attribute_pairs.push((
                field_attr,
                quote_spanned! {span=>
                    let #name = Default::default();
                },
                quote_spanned! {span=>
                    #name
                },
            ));
        } else {
            field_attribute_pairs.push((
                field_attr,
                quote_spanned! {span=>
                    let #res_name = Decodable::decode_from_buf(buffer)?;
                    let #name = #res_name.0;
                    buffer = #res_name.1;
                },
                quote_spanned! {span=>
                    #name
                },
            ));
        }
    }

    field_attribute_pairs.sort_by(|(a, _, _), (b, _, _)| a.orderno_cmp(b));

    let recurse = field_attribute_pairs.iter().map(|(_, s, _)| s);
    let fields = field_attribute_pairs.iter().map(|(_, _, v)| v);

    return Ok(quote! {
        #(#recurse)*

        let decoded_res = Self {
            #(#fields),*
        };
    });
}
