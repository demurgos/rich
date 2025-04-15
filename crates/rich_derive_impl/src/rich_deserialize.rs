use crate::dummy;
use crate::internals::ast::{Container, Data, Style};
use crate::internals::context::Context;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{parse_quote, Path};

pub fn expand_derive_rich_deserialize(input: &mut syn::DeriveInput) -> syn::Result<TokenStream> {
  let mut cx = Context::new();
  let container: Container<'_> = match Container::from_ast(&mut cx, input) {
    Some(cont) => cont,
    None => {
      cx.check()?;
      return Err(syn::Error::new(input.span(), "failed to build `Container` ast"));
    }
  };

  let rich: Path = parse_quote!(_rich);
  let ident: &Ident = &container.ident;
  let meta_ident: Ident = match container.attributes.meta.name.as_ref() {
    Some(name) => name.clone(),
    None => Ident::new(&format!("{ident}Meta"), ident.span()),
  };
  let meta_type: TokenStream = match container.data {
    Data::Enum(_) => meta_struct(&meta_ident),
    Data::Struct(Style::Unit, _) => rich_deserialize_unit_struct(&meta_ident, &container),
    Data::Struct(Style::Newtype, _) => meta_newtype(&meta_ident),
    Data::Struct(Style::Tuple, _) => meta_newtype(&meta_ident),
    Data::Struct(Style::Struct, _) => meta_newtype(&meta_ident),
  };

  let impl_block = quote! {
    #meta_type

    #[automatically_derived]
    impl #rich::MetaType for #ident {
      type Meta = #meta_ident;
    }
  };

  cx.check()?;

  Ok(dummy::wrap_in_const(None, &rich, impl_block))
}

#[allow(unused)]
fn meta_enum(meta_ident: &Ident) -> TokenStream {
  quote! {
    #[derive(Default)]
    pub enum #meta_ident {}
  }
}

fn rich_deserialize_unit_struct(meta_ident: &Ident, container: &Container) -> TokenStream {
  let meta = ForwardMeta(&container.attributes.meta.attr);
  quote! {
    #meta
    pub struct #meta_ident;
  }
}

fn meta_newtype(meta_ident: &Ident) -> TokenStream {
  quote! {
    #[derive(Default)]
    pub struct #meta_ident(u32);
  }
}

#[allow(unused)]
fn meta_tuple(meta_ident: &Ident) -> TokenStream {
  quote! {
    #[derive(Default)]
    pub struct #meta_ident(u32, u32);
  }
}

fn meta_struct(meta_ident: &Ident) -> TokenStream {
  quote! {
    #[derive(Default)]
    pub struct #meta_ident {
      foo: u32,
    }
  }
}

#[derive(Debug)]
struct ForwardMeta<'a>(&'a [TokenStream]);

impl<'a> ToTokens for ForwardMeta<'a> {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    for meta in self.0 {
      tokens.append_all(quote! { #[#meta]})
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use syn::parse2;
  use syn::DeriveInput;

  #[test]
  fn unit_struct() {
    let mut input: DeriveInput = parse2(quote! {
      #[meta(name = MetaUnit, attr(derive(Default, Debug)))]
      struct MyUnit;
    })
    .expect("parsing succeeds");

    let actual = expand_derive_rich_deserialize(&mut input).expect("derive succeeds");

    // language=rust
    let expected = quote! {
      #[doc(hidden)]
      #[allow(non_upper_case_globals, unused_attributes, unused_qualifications, clippy::absolute_paths)]
      const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate rich as _rich;

        #[derive(Default, Debug)]
        pub struct MetaUnit;

        #[automatically_derived]
        impl _rich::MetaType for MyUnit {
          type Meta = MetaUnit;
        }
      };
    };

    assert_eq!(actual.to_string(), expected.to_string());
  }
}
