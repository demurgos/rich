use std::fmt;
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const ATTR: Symbol = Symbol("attr");
pub const META: Symbol = Symbol("meta");
pub const NAME: Symbol = Symbol("name");
pub const RICH: Symbol = Symbol("rich");

impl PartialEq<Symbol> for Ident {
  fn eq(&self, other: &Symbol) -> bool {
    self == other.0
  }
}

impl PartialEq<Symbol> for &Ident {
  fn eq(&self, symbol: &Symbol) -> bool {
    *self == symbol.0
  }
}

impl PartialEq<Symbol> for Path {
  fn eq(&self, symbol: &Symbol) -> bool {
    self.is_ident(symbol.0)
  }
}

impl PartialEq<Symbol> for &Path {
  fn eq(&self, symbol: &Symbol) -> bool {
    self.is_ident(symbol.0)
  }
}

impl fmt::Display for Symbol {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str(self.0)
  }
}
