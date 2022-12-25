mod byte_coding_attr;
mod decoding;
mod encoding;
mod parsing;

use decoding::decoding;
use encoding::encoding;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Decodable, attributes(byte_coding))]
pub fn decodable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let decoding_calls = decoding(&input);
    let name = input.ident;

    let expanded = quote! {
        impl Decodable for #name {
            fn decode_from_buf(mut buffer: &[u8]) -> Option<(Self, &[u8])> {
                #decoding_calls
            }
        }
    };

    return proc_macro::TokenStream::from(expanded);
}

#[proc_macro_derive(Encodable, attributes(byte_coding))]
pub fn encodable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let encoding_calls = encoding(&input);
    let name = input.ident;

    let expanded = quote! {
        impl Encodable for #name {
            fn encode_to_buf(&self, buf: &mut Vec<u8>) {
                #encoding_calls
            }
        }
    };

    return proc_macro::TokenStream::from(expanded);
}
