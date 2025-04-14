use crate::internals::context::Context;
use crate::internals::symbol::{ATTR, META, NAME};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::ParseBuffer;

#[derive(Debug)]
pub struct ContainerAttributes {
  /// Options for the `MetaType` associated with this container
  pub meta: ContainerMetaAttributes,
}

#[derive(Debug)]
pub struct ContainerMetaAttributes {
  pub name: Option<Ident>,
  /// Extra attributes to attach to this container.
  pub attr: Vec<TokenStream>,
}

impl ContainerAttributes {
  /// Extract out the `#[serde(...)]` attributes from an item.
  pub fn from_ast(cx: &mut Context, item: &syn::DeriveInput) -> Self {
    let mut meta_attr: Vec<TokenStream> = Vec::new();
    let mut meta_name: Option<Ident> = None;

    for attr in &item.attrs {
      if attr.path() != META {
        continue;
      }

      if let syn::Meta::List(meta) = &attr.meta {
        if meta.tokens.is_empty() {
          continue;
        }
      }

      attr
        .parse_nested_meta(|meta| -> Result<(), syn::Error> {
          if meta.path == ATTR {
            let content: ParseBuffer;
            syn::parenthesized!(content in meta.input);
            let content = content.parse::<TokenStream>()?;
            meta_attr.push(content);
          } else if meta.path == NAME {
            let value = meta.value()?.parse::<Ident>()?;
            meta_name = Some(value);
          } else {
            let path = meta.path.to_token_stream().to_string().replace(' ', "");
            return Err(meta.error(format_args!("unknown rich container attribute `{}`", path)));
          }
          Ok(())
        })
        .expect("invalid nested meta");
    }

    Self {
      meta: ContainerMetaAttributes {
        attr: meta_attr,
        name: meta_name,
      },
    }
  }
}
