//! This crate provides derove macros for `rich` traits.

extern crate proc_macro2;
extern crate quote;
extern crate syn;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod meta_type;
pub(crate) mod dummy;
pub(crate) mod internals;

#[proc_macro_derive(MetaType, attributes(rich))]
pub fn derive_meta_type(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    meta_type::expand_derive_meta_type(&mut input)
      .unwrap_or_else(syn::Error::into_compile_error)
      .into()
}
