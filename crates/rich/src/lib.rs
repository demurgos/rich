//! This module defines the [`Data`] struct. It is an internal implementation
//! that should not be relied on by external code.

pub mod ecosystem;

/// Placeholder type for [`Data`] values where there is no metadata associated
/// with the value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmptyMeta;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaId(pub usize);

impl MetaId {
  pub const fn into_usize(self) -> usize {
    self.0
  }

  pub const fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

/// A rich value of type `T`, attached to metadata of type `M`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rich<T, M = MetaId> {
  pub value: T,
  pub meta: M,
}

impl<T, M> Rich<T, M> {
  /// Create a [`Rich`] value, by attaching metadata to a value.
  pub const fn new(value: T, meta: M) -> Self {
    Self { value, meta }
  }

  /// Map both the value and metadata to references
  pub const fn as_ref<'rich>(&'rich self) -> Rich<&'rich T, &'rich M> {
    Rich {
      value: &self.value,
      meta: &self.meta,
    }
  }

  // /// Wrap a value inside a [`Rich`], using default metadata.
  // pub fn simple(value: T) -> Self {
  //     Self {
  //         value,
  //         meta: M::default(),
  //     }
  // }
  //
  // /// Create a new `Data`, by mapping the value.
  // ///
  // /// The metadata is kept unchanged.
  // pub fn map<U, F>(self, f: F) -> Rich<U, M>
  // where
  //   F: FnOnce(T) -> U,
  // {
  //     Rich {
  //         value: f(self.value),
  //         meta: self.meta,
  //     }
  // }
  //
  // /// Create a new `Data`, by mapping the metadata.
  // ///
  // /// The value is kept unchanged.
  // pub fn map_meta<N, F>(self, f: F) -> Rich<T, N>
  // where
  //   F: FnOnce(M) -> N,
  // {
  //     Rich {
  //         value: self.value,
  //         meta: f(self.meta),
  //     }
  // }
  //
  // pub fn into_pair(self) -> (T, M) {
  //     (self.value, self.meta)
  // }
}

pub trait MetaType {
  type Meta;
}

#[expect(
  type_alias_bounds,
  reason = "even if it's not enforced yet (see <https://github.com/rust-lang/rust/issues/112792>) the type bound serves as documentation"
)]
pub type Meta<T: MetaType> = <T as MetaType>::Meta;

impl MetaType for () {
  type Meta = ();
}

impl MetaType for bool {
  type Meta = ();
}

impl MetaType for u32 {
  type Meta = ();
}

impl MetaType for String {
  type Meta = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaNode<N> {
  id: MetaId,
  nested: N,
}

impl<N> MetaNode<N> {
  /// Create a [`Rich`] value, by attaching metadata to a value.
  pub const fn new(id: MetaId, nested: N) -> Self {
    Self { id, nested }
  }
}

pub trait SplitMeta {
  type Value: MetaType;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>>;
}

impl SplitMeta for bool {
  type Value = Self;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
    Rich::new(self, ())
  }
}

impl SplitMeta for u32 {
  type Value = Self;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
    Rich::new(self, ())
  }
}

impl SplitMeta for String {
  type Value = Self;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
    Rich::new(self, ())
  }
}

pub trait MergeMeta<TyMeta>
where
  Self: MetaType<Meta = TyMeta>,
  Self::Rich: SplitMeta<Value = Self>,
{
  type Rich;

  fn merge_meta(self, meta: TyMeta) -> Self::Rich;
}

impl<T> Rich<T>
where
  T: SplitMeta,
{
  pub fn deep_split_meta(self) -> Rich<T::Value, MetaNode<Meta<T::Value>>> {
    let rich = self.value.split_meta();
    Rich::new(rich.value, MetaNode::new(self.meta, rich.meta))
  }
}

/// Represents a metadata node in a structured hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WrappedMeta<N> {
  /// Metadata id for this level of the hierarchy
  id: MetaId,
  /// Nested metadata
  nested: N,
}

impl<N> WrappedMeta<N> {
  /// Create a new [WrappedMeta].
  pub const fn new(nested: N, id: MetaId) -> Self {
    Self { nested, id }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoolView<'rich>(Rich<&'rich bool, &'rich WrappedMeta<Meta<bool>>>);

impl<'rich> BoolView<'rich> {
  pub fn new(rich: Rich<&'rich bool, &'rich WrappedMeta<Meta<bool>>>) -> Self {
    Self(rich)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct Mascot {
    is_crab: bool,
    price: u32,
  }

  impl MetaType for Mascot {
    type Meta = MascotMeta;
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct MascotMeta {
    is_crab: MetaNode<Meta<bool>>,
    price: MetaNode<Meta<u32>>,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct RichMascot {
    is_crab: Rich<bool, MetaId>,
    price: Rich<u32, MetaId>,
  }

  impl SplitMeta for RichMascot {
    type Value = Mascot;

    fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
      let is_crab = self.is_crab.deep_split_meta();
      let price = self.price.deep_split_meta();
      Rich::new(
        Mascot {
          is_crab: is_crab.value,
          price: price.value,
        },
        MascotMeta {
          is_crab: is_crab.meta,
          price: price.meta,
        },
      )
    }
  }

  #[test]
  fn deep_split_meta_mascot() {
    let config = Rich::new(
      RichMascot {
        is_crab: Rich::new(true, MetaId(1)),
        price: Rich::new(42, MetaId(2)),
      },
      MetaId(3),
    );

    let actual = config.deep_split_meta();

    let expected = Rich::new(
      Mascot {
        is_crab: true,
        price: 42,
      },
      MetaNode::new(
        MetaId(3),
        MascotMeta {
          is_crab: MetaNode::new(MetaId(1), ()),
          price: MetaNode::new(MetaId(2), ()),
        },
      ),
    );

    assert_eq!(actual, expected);
  }
}
