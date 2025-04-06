use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn wrap_in_const(extern_path: Option<&Path>, local_path: &Path, code: TokenStream) -> TokenStream {
  let use_rich = match extern_path {
    Some(path) => quote! {
        use #path as #local_path;
    },
    None => quote! {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate rich as #local_path;
    },
  };

  quote! {
      #[doc(hidden)]
      #[allow(
          non_upper_case_globals,
          unused_attributes,
          unused_qualifications,
          clippy::absolute_paths,
      )]
      const _: () = {
          #use_rich
          #code
      };
  }
}
