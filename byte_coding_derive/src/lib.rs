//! Derive macros for the Decodable and Encodable traits provided by the byte_coding crate.
//!
//! # Usage
//! It is not recommended to include this crate directly, instead use the 'derive' feature
//! of the byte_coding crate. It is enabled by default but can be manually enabled:
//!
//! ### Including Derive Macros
//! Manually enabling the 'derive' feature of byte_coding.
//! ```toml
//! byte_coding = { git = "https://github.com/aidos9/byte_coding", features = ["derive"] }
//! ```

mod byte_coding_attr;
mod decoding;
mod encoding;
mod parsing;

use decoding::decoding;
use encoding::encoding;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Generates an implementation of the Decodable trait for a data type.
///
/// Only structs and enums are supported. Both unit structs/enum variants,
/// tuple struct/enum variants, struct enum variants and regular structs
/// are supported. The `byte_coding` attribute can be used to specify
/// configuration options against structs, enums, fields or enum variants.
///
/// # Available Attribute Values
/// #### Structs and Enums
/// * `pre_dec_func` - A string which contains the name of a function which should perform
/// some operation on the source data before the decoding operations are executed.
/// e.g. `#[byte_coding(pre_dec_func = "my_func")]`
/// * `post_dec_func` - A string which contains the name of a function which should perform
/// some operation on the decoded data before it is returned.
/// e.g. `#[byte_coding(post_dec_func = "my_func")]`
///
/// #### Struct Fields
/// * `order_no` - An integer to indicate the order in which the field should be encoded.
/// There is no requirement for these numbers to be contiguous, or for each field to
/// have a value. However, no value can be repeated and any fields without order numbers
/// will be processed from top to bottom after any fields with order numbers.
/// e.g. `#[byte_coding(order_no = 0)]`
/// * `ignore` - Specify this option to ignore decoding this field, this value must have the
/// Default trait implemented.
/// e.g. `#[byte_coding(ignore)]`
///
/// #### Enums
/// * `encoding_type` - A string which indicates what type the enum variant values are,
/// by default a 'u16' value is used. If a smaller or larger value is required it should
/// be annotated here. Supported values: `["u8", "u16", "u32", "u64", "u128"]`.
/// e.g. `#[byte_coding(encoding_type = "u64")]`
/// * `inferred_values` - A flag which when set indicates the byte_coding can infer values,
/// by default these are numeric values starting at zero and increasing by 1 for each variant.
/// You can override any specific variant by manually providing a value for that variant but
/// any future inferred values will continue from that value.
/// e.g. ``#[byte_coding(inferred_values)]``
///
/// #### Enum Variants
/// * `value` - Each enum variant is assigned a positive integer value. By default this
/// macro will use the discriminant values from each enum variant. However, if this
/// is not provided, instead this attribute must be set to inform the macro the values
/// it should assign to each enum variant. e.g. `#[byte_coding(value = 1)]`
///
/// # Examples
/// ### Simple Examples
/// Most direct serialisation and deserialisation should be able to be achieved without
/// any specific configuration. In this first example, the fields will be serialised in
/// the order from top to bottom.
/// ```
/// use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// struct Example1 {
///     f1: String,
///     f2: u64,
///     f3: Option<u16>,
///     f4: Box<Option<String>>
/// }
///
/// // This struct can now be decoded like any other
/// let encoded_data = vec![0x4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x4, 0, 0, 0, 0, 0, 0, 0, b'c', b'a', b't', b's'];
/// let decoded = Example1::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example1 {
///     f1: "test".to_string(),
///     f2: 0xff,
///     f3: None,
///     f4: Box::new(Some("cats".to_string()))
/// });
/// ```
///
/// The below examples are the 3 possible ways for using the derive macro on an enum
/// ```
/// use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// enum Example2 {
///     E2A = 0,
///     E2B = 1,
///     E2C = 2
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let encoded_data = vec![1, 0];
/// let decoded = Example2::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example2::E2B);
/// ```
///
/// Tuple and struct enums are also supported
/// ```
/// use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// enum Example3 {
///     #[byte_coding(value = 0)]
///     E3A,
///     #[byte_coding(value = 1)]
///     E3B(String, u32),
///     #[byte_coding(value = 2)]
///     E3C{
///         s: String,
///         id: u32
///     },
/// }
///
/// /// // By default enums are identified using 2 bytes
/// let encoded_data = vec![0, 0];
/// let decoded = Example3::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example3::E3A);
///
/// let encoded_data = vec![1, 0, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 32, 0, 0, 0];
/// let decoded = Example3::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example3::E3B("test".to_string(), 32));
///
/// let encoded_data = vec![2, 0, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 32, 0, 0, 0];
/// let decoded = Example3::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example3::E3C {
///     s: "test".to_string(),
///     id: 32
/// });
/// ```
///
/// Inferred values can also be used
/// ```
/// use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// #[byte_coding(inferred_values)]
/// enum Example4 {
///     E2A,
///     E2B,
///     E2C
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let encoded_data = vec![1, 0];
/// let decoded = Example4::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, Example4::E2B);
/// ```
///
/// ### Complex Example
/// If a different ordering is desired for the fields then the following example can be used.
/// In the below example the `f1` field is encoded last, the `f2` field is encoded and the
/// `f3` field is encoded first.
///
/// ```
/// # use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// struct ComplexStruct {
///     f1: String,
///     #[byte_coding(order_no = 1)]
///     f2: u8,
///     #[byte_coding(order_no = 0)]
///     f3: Option<String>,
///     #[byte_coding(ignore)]
///     f4: String,
/// }
///
/// let encoded = vec![0, 82, 3, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b'a'];
/// let decoded = ComplexStruct::decode(&encoded).unwrap();
///
/// assert_eq!(decoded, ComplexStruct{
///     f1: "tea".to_string(),
///     f2: 82,
///     f3: None,
///     f4: Default::default()
/// });
/// ```
///
/// Enums can use a different type for encoding their variant's unique identifiers. Importantly
/// the derive macro does not respect the value of the `repr` macro. This means that they can
/// differ in types.
///
/// ```
/// # use byte_coding::Decodable;
///
/// # #[derive(Clone, Debug, PartialEq)]
/// #[derive(Decodable)]
/// #[byte_coding(encoding_type = "u8")]
/// enum ExampleEnum {
///     EA = 0,
///     EB = 1,
///     EC = 2
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let encoded_data = vec![1, 0, 2];
/// let decoded: [ExampleEnum ; 3] = Decodable::decode(&encoded_data).unwrap();
///
/// assert_eq!(decoded, [ExampleEnum::EB, ExampleEnum::EA, ExampleEnum::EC]);
/// ```
///
/// Additionally if you want to perform some operation on the data or result before or after
/// decoding, you can set a function the `pre_dec_func` key and the `post_dec_func` key.
///
/// The function specified as the `pre_dec_func` must have the following type signature, but
/// can be called any name:
/// ```ignore
/// fn pre_dec_func(buffer: &[u8]) -> Option<&[u8]>;
/// ```
///
/// The function specified as the `post_dec_func` must have the following type signature,
/// where T is the type being decoded and the function name can be chosen as you wish:
/// ```ignore
/// fn post_dec_func(value: T, buffer: &[u8]) -> Option<(T, &[u8])>;
/// ```
///
/// In the below example we change the f3 value to a None value if the decoded struct has a 0
/// length string.
/// ```
/// # use byte_coding::Decodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Decodable)]
/// #[byte_coding(post_dec_func = "modify_result")]
/// struct ComplexStruct {
///     f1: Option<String>
/// }
///
/// fn modify_result(mut value: ComplexStruct, buffer: &[u8]) -> Option<(ComplexStruct, &[u8])>{
///     if value.f1.is_some() {
///         if value.f1.as_ref().unwrap().is_empty() {
///            value.f1 = None;
///         }
///     }
///
///     return Some((value, buffer));
/// }
///
/// let encoded = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
/// let decoded = ComplexStruct::decode(&encoded).unwrap();
///
/// assert_eq!(decoded, ComplexStruct{
///     f1: None
/// });
/// ```
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

/// Generates an implementation of the Encodable trait for a data type.
///
/// Only structs and enums are supported. Both unit structs/enum variants,
/// tuple struct/enum variants, struct enum variants and regular structs
/// are supported. The `byte_coding` attribute can be used to specify
/// configuration options against structs, enums, fields or enum variants.
///
/// # Available Attribute Values
/// #### Structs and Enums
/// * `pre_enc_func` - A string which contains the name of a function which should perform
/// some operation on the source object before the encoding operations are executed.
/// e.g. `#[byte_coding(pre_enc_func = "my_func")]`
/// * `post_enc_func` - A string which contains the name of a function which should perform
/// some operation on the encoded data before it is returned.
/// e.g. `#[byte_coding(post_enc_func = "my_func")]`
///
/// #### Struct Fields
/// * `order_no` - An integer to indicate the order in which the field should be encoded.
/// There is no requirement for these numbers to be contiguous, or for each field to
/// have a value. However, no value can be repeated and any fields without order numbers
/// will be processed from top to bottom after any fields with order numbers.
/// e.g. `#[byte_coding(order_no = 0)]`
/// * `ignore` - Specify this option to ignore encoding this field.
/// e.g. `#[byte_coding(ignore)]`
///
/// #### Enums
/// * `encoding_type` - A string which indicates what type the enum variant values are,
/// by default a 'u16' value is used. If a smaller or larger value is required it should
/// be annotated here. Supported values: `["u8", "u16", "u32", "u64", "u128"]`.
/// e.g. `#[byte_coding(encoding_type = "u64")]`
/// * `inferred_values` - A flag which when set indicates the byte_coding can infer values,
/// by default these are numeric values starting at zero and increasing by 1 for each variant.
/// You can override any specific variant by manually providing a value for that variant but
/// any future inferred values will continue from that value.
/// e.g. ``#[byte_coding(inferred_values)]``
///
/// #### Enum Variants
/// * `value` - Each enum variant is assigned a positive integer value. By default this
/// macro will use the discriminant values from each enum variant. However, if this
/// is not provided, instead this attribute must be set to inform the macro the values
/// it should assign to each enum variant. e.g. `#[byte_coding(value = 1)]`
///
/// # Examples
/// ### Simple Examples
/// Most direct serialisation and deserialisation should be able to be achieved without
/// any specific configuration. In this first example, the fields will be serialised in
/// the order from top to bottom.
/// ```
/// use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// struct Example1 {
///     f1: String,
///     f2: u64,
///     f3: Option<u16>,
///     f4: Box<Option<String>>
/// }
///
/// // This struct can now be decoded like any other
/// let comparison_encoded = vec![0x4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0x4, 0, 0, 0, 0, 0, 0, 0, b'c', b'a', b't', b's'];
/// let encoded = Example1 {
///     f1: "test".to_string(),
///     f2: 0xff,
///     f3: None,
///     f4: Box::new(Some("cats".to_string()))
/// }.encoded();
///
/// assert_eq!(comparison_encoded, encoded);
/// ```
///
/// The below examples are the 3 possible ways for using the derive macro on an enum
/// ```
/// use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// enum Example2 {
///     E2A = 0,
///     E2B = 1,
///     E2C = 2
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let comparison_encoded = vec![1, 0];
/// let encoded = Example2::E2B.encoded();
///
/// assert_eq!(comparison_encoded, encoded);
/// ```
///
/// Tuple and struct enums are also supported
/// ```
/// use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// enum Example3 {
///     #[byte_coding(value = 0)]
///     E3A,
///     #[byte_coding(value = 1)]
///     E3B(String, u32),
///     #[byte_coding(value = 2)]
///     E3C{
///         s: String,
///         id: u32
///     },
/// }
///
/// /// // By default enums are identified using 2 bytes
/// let comparison_encoded = vec![0, 0];
/// let encoded = Example3::E3A.encoded();
///
/// assert_eq!(encoded, comparison_encoded);
///
/// let comparison_encoded = vec![1, 0, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 32, 0, 0, 0];
/// let encoded = Example3::E3B("test".to_string(), 32).encoded();
///
/// assert_eq!(encoded, comparison_encoded);
///
/// let comparison_encoded = vec![2, 0, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't', 32, 0, 0, 0];
/// let encoded = Example3::E3C {
///     s: "test".to_string(),
///     id: 32
/// }.encoded();
///
/// assert_eq!(encoded, comparison_encoded);
/// ```
///
/// Inferred values can also be used
/// ```
/// use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// #[byte_coding(inferred_values)]
/// enum Example4 {
///     E2A,
///     E2B,
///     E2C
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let comparison_encoded = vec![1, 0];
/// let encoded = Example4::E2B.encoded();
///
/// assert_eq!(comparison_encoded, encoded);
/// ```
///
/// ### Complex Example
/// If a different ordering is desired for the fields then the following example can be used.
/// In the below example the `f1` field is encoded last, the `f2` field is encoded and the
/// `f3` field is encoded first.
///
/// ```
/// # use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// struct ComplexStruct {
///     f1: String,
///     #[byte_coding(order_no = 1)]
///     f2: u8,
///     #[byte_coding(order_no = 0)]
///     f3: Option<String>,
///     #[byte_coding(ignore)]
///     f4: String,
/// }
///
/// let comparison_encoded = vec![0, 82, 3, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b'a'];
/// let encoded = ComplexStruct{
///     f1: "tea".to_string(),
///     f2: 82,
///     f3: None,
///     f4: "test".to_string()
/// }.encoded();
///
/// assert_eq!(encoded, comparison_encoded);
/// ```
///
/// Enums can use a different type for encoding their variant's unique identifiers. Importantly
/// the derive macro does not respect the value of the `repr` macro. This means that they can
/// differ in types.
///
/// ```
/// # use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq)]
/// #[derive(Encodable)]
/// #[byte_coding(encoding_type = "u8")]
/// enum ExampleEnum {
///     EA = 0,
///     EB = 1,
///     EC = 2
/// }
///
/// // By default enums are encoded using 2 bytes for the ID
/// let comparison_encoded = vec![1, 0, 2];
/// let encoded = [ExampleEnum::EB, ExampleEnum::EA, ExampleEnum::EC].encoded();
///
/// assert_eq!(encoded, comparison_encoded);
/// ```
///
/// Additionally if you want to perform some operation on the data or result before or after
/// encoding, you can set a function the `pre_enc_func` key and the `post_enc_func` key.
///
/// The function specified as the `pre_enc_func` must have the following type signature
/// where T is the type being decoded and the function name can be chosen as you wish:
/// ```ignore
/// fn pre_enc_func(value: &T) -> T
/// ```
///
/// The function specified as the `post_enc_func` must have the following type signature,
/// but the function name can be chosen as you wish:
/// ```ignore
/// fn post_enc_func(buffer: &mut Vec<u8>);
/// ```
///
/// In the below example we change the f3 value to a None value if the struct has a 0
/// length string.
/// ```
/// # use byte_coding::Encodable;
///
/// # #[derive(Debug, PartialEq, Clone)]
/// #[derive(Encodable)]
/// #[byte_coding(pre_enc_func = "validate_length")]
/// struct ComplexStruct {
///     f1: Option<String>
/// }
///
/// fn validate_length(value: &ComplexStruct) -> ComplexStruct{
///     let mut res = value.clone();
///
///     if res.f1.is_some() {
///         if res.f1.as_ref().unwrap().is_empty() {
///            res.f1 = None;
///         }
///     }
///
///     return res;
/// }
///
/// let comparison_encoded = vec![0];
/// let encoded = ComplexStruct{
///     f1: None
/// }.encoded();
///
/// assert_eq!(encoded, comparison_encoded);
/// ```
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
