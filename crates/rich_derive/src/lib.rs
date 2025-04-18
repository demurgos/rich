//! This crate provides derive macros for `rich` traits.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MetaType, attributes(meta, rich))]
pub fn derive_meta_type(input: TokenStream) -> TokenStream {
  let mut input = parse_macro_input!(input as DeriveInput);
  let stream: proc_macro2::TokenStream =
    rich_derive_impl::meta_type::expand_derive_meta_type(&mut input).unwrap_or_else(syn::Error::into_compile_error);
  proc_macro::TokenStream::from(stream)
}
