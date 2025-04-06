/// A source data structure annotated with `derive`, parsed into an internal representation.
pub struct Container<'a> {
  /// The struct or enum name (without generics).
  pub ident: syn::Ident,
  /// Original input.
  pub original: &'a syn::DeriveInput,
}

impl<'a> Container<'a> {
  /// Convert the raw Syn ast into a parsed container object, collecting errors in `cx`.
  pub fn from_ast(item: &'a syn::DeriveInput) -> Option<Container<'a>> {
    let item = Container {
      ident: item.ident.clone(),
      original: item,
    };
    Some(item)
  }
}
