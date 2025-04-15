use drop_bomb::DropBomb;
use quote::ToTokens;
use std::fmt::Display;

/// A type to collect errors together and format them.
///
/// Dropping this object will cause a panic. It must be consumed using `check`.
///
/// References can be shared since this type uses run-time exclusive mut checking.
pub struct Context {
  // The contents will be set to `None` during checking. This is so that checking can be
  // enforced.
  errors: Vec<syn::Error>,
  bomb: DropBomb,
}

impl Context {
  /// Create a new context object.
  ///
  /// This object contains no errors, but will still trigger a panic if it is not `check`ed.
  pub fn new() -> Self {
    Context {
      errors: Vec::new(),
      bomb: DropBomb::new("`Context::check` must be called"),
    }
  }

  /// Add an error to the context object with a tokenenizable object.
  ///
  /// The object is used for spanning in error messages.
  pub fn error_spanned_by<A, T>(&mut self, obj: A, msg: T)
  where
    A: ToTokens,
    T: Display,
  {
    self.errors.push(syn::Error::new_spanned(obj.into_token_stream(), msg));
  }

  /// Add one of Syn's parse errors.
  pub fn syn_error(&mut self, err: syn::Error) {
    self.errors.push(err);
  }

  /// Consume this object, producing a formatted error string if there are errors.
  pub fn check(mut self) -> syn::Result<()> {
    self.bomb.defuse();

    let mut errors = self.errors.into_iter();

    let mut combined = match errors.next() {
      Some(first) => first,
      None => return Ok(()),
    };

    for rest in errors {
      combined.combine(rest);
    }

    Err(combined)
  }
}
