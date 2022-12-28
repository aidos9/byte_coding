//! This crate provides traits and macros which facilitate the encoding of arbitrary data structures into an array of bytes.
//!
//! # Usage
//! ## Add Library
//! Add this library to your Cargo.toml:
//! ```toml
//! byte_coding = { git = "https://github.com/aidos9/byte_coding" }
//! ```
//!
//! ### Features
//! - `derive` - Enables the derive macros for enums and structs (default)
//! - `std` - Enables features which required std (default)
//! - `coder` - Enables the [Coder] struct as a simple front end for decoding multiple objects
//!
//! # Example
//! ```
//! use byte_coding::*;
//!
//! #[derive(Encodable, Decodable, PartialEq, Debug)]
//! struct Example {
//!     field_1: u64,
//!     #[byte_coding(order_no = 0)]
//!     field_2: u32,
//!     field_3: u16,
//!     field_4: u8
//! }
//!
//! fn main() {
//!     let example = Example {
//!         field_1: u64::MAX,
//!         field_2: 0,
//!         field_3: u16::MAX,
//!         field_4: 0    
//!     };
//!
//!     // Encode example sturct into a vector of bytes
//!     let encoded_bytes = example.encoded();
//!     assert_eq!(encoded_bytes, vec![0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0]);
//!
//!     let decoded_example: Example = Decodable::decode(&encoded_bytes).unwrap();
//!     assert_eq!(decoded_example, example);
//! }
//! ```
//!
//! See the [Encodable] and [Decodable] traits for further details

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod decodable;
mod encodable;

#[cfg(feature = "coder")]
mod coder;
#[cfg(feature = "coder")]
pub use coder::Coder;

#[cfg(feature = "derive")]
pub use byte_coding_derive::*;
pub use decodable::*;
pub use encodable::*;
