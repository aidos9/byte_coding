use std::{fmt::Display, str::FromStr};

use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, Attribute, DeriveInput, Lit, Meta, MetaNameValue, NestedMeta};

pub const BYTE_CODING_BASE_IDENT: &'static str = "byte_coding";

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ByteCodingAttr {
    pub pre_enc_func: Option<String>,
    pub post_enc_func: Option<String>,
    pub pre_dec_func: Option<String>,
    pub post_dec_func: Option<String>,
    pub enum_options: Option<ByteCodingEnumAttr>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ByteCodingEnumAttr {
    pub encoding_type: Option<EnumEncodingType>,
    pub inferred_values: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ByteCodingEnumVariantAttr {
    pub value: Option<u128>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ByteCodingStructFieldAttr {
    pub order_no: Option<usize>,
    pub ignore: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EnumEncodingType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

macro_rules! merge_optionals {
    ($a:expr, $b:expr) => {
        if $b.is_some() {
            $a = $b;
        }
    };
}

impl ByteCodingAttr {
    pub fn new() -> Self {
        return Self::default();
    }

    fn lit_to_string(literal: &Lit) -> Result<String, TokenStream> {
        return match literal {
            Lit::Str(s) => Ok(s.value()),
            _ => Err(quote_spanned! {
                literal.span() =>
                compile_error!("Expected a string.");
            }),
        };
    }

    pub fn from_data(input: &DeriveInput) -> Result<(Self, Option<&Attribute>), TokenStream> {
        let mut toplevel_attrs: Vec<ByteCodingAttr> = Vec::new();
        let mut first_enum_attr = None;

        for attr in input.attrs.iter().filter(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == BYTE_CODING_BASE_IDENT
        }) {
            let n_attr = ByteCodingAttr::try_from(
                attr.parse_meta()
                    .expect("Unable to parse meta in a top level attribute"),
            )?;

            if n_attr.enum_options.is_some() && first_enum_attr.is_none() {
                first_enum_attr = Some(attr);
            }

            toplevel_attrs.push(n_attr);
        }

        return Ok((
            toplevel_attrs
                .into_iter()
                .reduce(|running, e| running.merged(e))
                .unwrap_or_default(),
            first_enum_attr,
        ));
    }

    pub fn merged(mut self, other: Self) -> Self {
        merge_optionals!(self.pre_enc_func, other.pre_enc_func);
        merge_optionals!(self.pre_dec_func, other.pre_dec_func);
        merge_optionals!(self.post_enc_func, other.post_enc_func);
        merge_optionals!(self.post_dec_func, other.post_dec_func);

        if let Some(dest_enum_opts) = self.enum_options.as_mut() {
            if let Some(src_enum_opts) = other.enum_options.as_ref() {
                merge_optionals!(dest_enum_opts.encoding_type, src_enum_opts.encoding_type);
                dest_enum_opts.inferred_values =
                    dest_enum_opts.inferred_values || src_enum_opts.inferred_values;
            }
        } else {
            self.enum_options = other.enum_options;
        }

        return self;
    }

    pub fn set_name_value(&mut self, name_value: &MetaNameValue) -> Result<(), TokenStream> {
        if name_value.path.segments.len() != 1 {
            quote_spanned! {
                name_value.path.span() =>
                compile_error!("Unknown attribute name");
            };
        }

        match name_value.path.segments[0].ident.to_string().as_str() {
            "pre_enc_func" => self.pre_enc_func = Some(Self::lit_to_string(&name_value.lit)?),
            "pre_dec_func" => self.pre_dec_func = Some(Self::lit_to_string(&name_value.lit)?),
            "post_enc_func" => self.post_enc_func = Some(Self::lit_to_string(&name_value.lit)?),
            "post_dec_func" => self.post_dec_func = Some(Self::lit_to_string(&name_value.lit)?),
            "encoding_type" => {
                let variant = match Self::lit_to_string(&name_value.lit)?.as_str() {
                    "u8" => EnumEncodingType::U8,
                    "u16" => EnumEncodingType::U16,
                    "u32" => EnumEncodingType::U32,
                    "u64" => EnumEncodingType::U64,
                    "u128" => EnumEncodingType::U128,
                    _ => {
                        return Err(quote_spanned! {
                            name_value.lit.span() =>
                            compile_error!("Unknown encoding type.");
                        });
                    }
                };

                if self.enum_options.is_none() {
                    self.enum_options = Some(ByteCodingEnumAttr {
                        encoding_type: Some(variant),
                        inferred_values: false,
                    });
                } else {
                    self.enum_options.as_mut().unwrap().encoding_type = Some(variant);
                }
            }
            _ => {
                return Err(quote_spanned! {
                    name_value.path.span() =>
                    compile_error!("Unknown attribute name");
                });
            }
        }

        return Ok(());
    }

    fn set_path(&mut self, path: &syn::Path) -> Result<(), TokenStream> {
        match path.segments[0].ident.to_string().as_str() {
            "inferred_values" => {
                if self.enum_options.is_none() {
                    self.enum_options = Some(ByteCodingEnumAttr {
                        encoding_type: None,
                        inferred_values: true,
                    });
                } else {
                    self.enum_options.as_mut().unwrap().inferred_values = true;
                }
            }
            _ => {
                return Err(quote_spanned! {
                    path.span() =>
                        compile_error!("Unknown attribute name");
                });
            }
        }

        return Ok(());
    }
}

impl Default for ByteCodingAttr {
    fn default() -> Self {
        return Self {
            pre_enc_func: None,
            post_enc_func: None,
            pre_dec_func: None,
            post_dec_func: None,
            enum_options: None,
        };
    }
}

impl TryFrom<Meta> for ByteCodingAttr {
    type Error = TokenStream;

    fn try_from(value: Meta) -> Result<Self, Self::Error> {
        return Self::try_from(&value);
    }
}

impl TryFrom<&Meta> for ByteCodingAttr {
    type Error = TokenStream;

    fn try_from(value: &Meta) -> Result<Self, Self::Error> {
        let mut a = Self::new();

        match value {
            Meta::List(ls) => {
                if ls.path.segments[0].ident != BYTE_CODING_BASE_IDENT {
                    panic!("Unexpected meta list path");
                }

                for nested in &ls.nested {
                    match nested {
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            a.set_name_value(name_value)?;
                        }
                        NestedMeta::Meta(Meta::Path(p)) => {
                            a.set_path(p)?;
                        }
                        NestedMeta::Meta(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute type");
                            });
                        }
                        NestedMeta::Lit(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute literal");
                            });
                        }
                    }
                }
            }
            _ => panic!("Unexpected attribute type"),
        }

        return Ok(a);
    }
}

impl ByteCodingEnumVariantAttr {
    pub fn new() -> Self {
        return Self::default();
    }

    fn merge(&mut self, other: Self) {
        merge_optionals!(self.value, other.value);
    }

    fn lit_to_num<N>(literal: &Lit) -> Result<N, TokenStream>
    where
        N: FromStr,
        N::Err: Display,
    {
        return match literal {
            Lit::Int(n) => n.base10_parse().map_err(|_| {
                quote_spanned! {literal.span()=>
                    compile_error!("Invalid integer");
                }
            }),
            _ => Err(quote_spanned! {
                literal.span() =>
                compile_error!("Expected a number.");
            }),
        };
    }

    fn set_name_value(&mut self, name_value: &MetaNameValue) -> Result<(), TokenStream> {
        if name_value.path.segments.len() != 1 {
            quote_spanned! {
                name_value.path.span() =>
                compile_error!("Unknown attribute name");
            };
        }

        match name_value.path.segments[0].ident.to_string().as_str() {
            "value" => self.value = Some(Self::lit_to_num(&name_value.lit)?),
            _ => {
                return Err(quote_spanned! {
                    name_value.path.span() =>
                    compile_error!("Unknown attribute name");
                });
            }
        }

        return Ok(());
    }

    pub fn parse_attributes(attributes: &Vec<Attribute>) -> Result<Self, TokenStream> {
        let mut working = Self::new();

        for attr in attributes.iter().filter(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == BYTE_CODING_BASE_IDENT
        }) {
            let n_attr = Self::try_from(
                attr.parse_meta()
                    .expect("Unable to parse meta in an enum variant attribute"),
            )?;

            working.merge(n_attr);
        }

        return Ok(working);
    }
}

impl Default for ByteCodingEnumVariantAttr {
    fn default() -> Self {
        return Self { value: None };
    }
}

impl TryFrom<Meta> for ByteCodingEnumVariantAttr {
    type Error = TokenStream;

    fn try_from(value: Meta) -> Result<Self, Self::Error> {
        return Self::try_from(&value);
    }
}

impl TryFrom<&Meta> for ByteCodingEnumVariantAttr {
    type Error = TokenStream;

    fn try_from(value: &Meta) -> Result<Self, Self::Error> {
        let mut a = Self::new();

        match value {
            Meta::List(ls) => {
                if ls.path.segments[0].ident != BYTE_CODING_BASE_IDENT {
                    panic!("Unexpected meta list path");
                }

                for nested in &ls.nested {
                    match nested {
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            a.set_name_value(name_value)?;
                        }
                        NestedMeta::Meta(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute type");
                            });
                        }
                        NestedMeta::Lit(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute literal");
                            });
                        }
                    }
                }
            }
            _ => panic!("Unexpected attribute type"),
        }

        return Ok(a);
    }
}

impl ByteCodingStructFieldAttr {
    pub fn new() -> Self {
        return Self::default();
    }

    fn merge(&mut self, other: Self) {
        merge_optionals!(self.order_no, other.order_no);

        self.ignore = self.ignore || other.ignore;
    }

    fn lit_to_num<N>(literal: &Lit) -> Result<N, TokenStream>
    where
        N: FromStr,
        N::Err: Display,
    {
        return match literal {
            Lit::Int(n) => n.base10_parse().map_err(|_| {
                quote_spanned! {literal.span()=>
                    compile_error!("Invalid integer");
                }
            }),
            _ => Err(quote_spanned! {
                literal.span() =>
                compile_error!("Expected a number.");
            }),
        };
    }

    fn set_name_value(&mut self, name_value: &MetaNameValue) -> Result<(), TokenStream> {
        if name_value.path.segments.len() != 1 {
            quote_spanned! {
                name_value.path.span() =>
                compile_error!("Unknown attribute name");
            };
        }

        match name_value.path.segments[0].ident.to_string().as_str() {
            "order_no" => self.order_no = Some(Self::lit_to_num(&name_value.lit)?),
            _ => {
                return Err(quote_spanned! {
                    name_value.path.span() =>
                    compile_error!("Unknown attribute name");
                });
            }
        }

        return Ok(());
    }

    fn set_path(&mut self, path: &syn::Path) -> Result<(), TokenStream> {
        match path.segments[0].ident.to_string().as_str() {
            "ignore" => self.ignore = true,
            _ => {
                return Err(quote_spanned! {
                    path.span() =>
                        compile_error!("Unknown attribute name");
                });
            }
        }

        return Ok(());
    }

    pub fn parse_attributes(attributes: &Vec<Attribute>) -> Result<Self, TokenStream> {
        let mut working = Self::new();

        for attr in attributes.iter().filter(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == BYTE_CODING_BASE_IDENT
        }) {
            let n_attr = Self::try_from(
                attr.parse_meta()
                    .expect("Unable to parse meta in an enum variant attribute"),
            )?;

            working.merge(n_attr);
        }

        return Ok(working);
    }

    pub fn orderno_cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        if let Some(s_ono) = self.order_no {
            if let Some(o_ono) = other.order_no {
                return s_ono.cmp(&o_ono);
            }

            return Ordering::Less;
        } else if other.order_no.is_some() {
            return Ordering::Greater;
        }

        return Ordering::Equal;
    }
}

impl Default for ByteCodingStructFieldAttr {
    fn default() -> Self {
        return Self {
            order_no: None,
            ignore: false,
        };
    }
}

impl TryFrom<Meta> for ByteCodingStructFieldAttr {
    type Error = TokenStream;

    fn try_from(value: Meta) -> Result<Self, Self::Error> {
        return Self::try_from(&value);
    }
}

impl TryFrom<&Meta> for ByteCodingStructFieldAttr {
    type Error = TokenStream;

    fn try_from(value: &Meta) -> Result<Self, Self::Error> {
        let mut a = Self::new();

        match value {
            Meta::List(ls) => {
                if ls.path.segments[0].ident != BYTE_CODING_BASE_IDENT {
                    panic!("Unexpected meta list path");
                }

                for nested in &ls.nested {
                    match nested {
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            a.set_name_value(name_value)?;
                        }
                        NestedMeta::Meta(Meta::Path(p)) => {
                            a.set_path(p)?;
                        }
                        NestedMeta::Meta(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute type");
                            });
                        }
                        NestedMeta::Lit(_) => {
                            return Err(quote_spanned! {
                                nested.span() =>
                                compile_error!("Unexpected attribute literal");
                            });
                        }
                    }
                }
            }
            _ => panic!("Unexpected attribute type"),
        }

        return Ok(a);
    }
}
