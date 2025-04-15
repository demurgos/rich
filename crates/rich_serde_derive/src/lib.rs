//! This crate provides derive macros for `rich_serde` traits.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MetaType, attributes(meta, serde, rich))]
pub fn derive_meta_type(input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as DeriveInput);
  let stream: proc_macro2::TokenStream =
    rich_derive_impl::rich_deserialize::expand_derive_rich_deserialize(&mut input).unwrap_or_else(syn::Error::into_compile_error);
  proc_macro::TokenStream::from(stream)
}
