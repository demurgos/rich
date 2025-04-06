use crate::dummy;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, Path};
use syn::spanned::Spanned;
use crate::internals::ast::Container;

pub fn expand_derive_meta_type(input: &mut syn::DeriveInput) -> syn::Result<TokenStream> {
  let container = match Container::from_ast(input) {
    Some(cont) => cont,
    None => return Err(syn::Error::new(input.span(), "failed to build `Container` ast")),
  };
  
  let rich: Path = parse_quote!(_rich);
  let ident: Ident = container.ident;

  let impl_block = quote! {
    #[automatically_derived]
    impl #rich::MetaType for #ident {
      type Meta = ();
    }
  };

  Ok(dummy::wrap_in_const(None, &rich, impl_block))
}
