//! This module defines the [`Data`] struct. It is an internal implementation
//! that should not be relied on by external code.

use std::marker::PhantomData;
use styp::{ConstProjector, Projector, StructuralProjection};

pub mod ecosystem;

/// Placeholder type for [`Data`] values where there is no metadata associated
/// with the value.
///
/// This is equivalent to `()` but allows to add extra methods without breaking
/// the orphan rule.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmptyMeta;

/// Opaque identifier for metadata.
///
/// This allows indirect attachment of information to Rust values. All values
/// can cheaply receive a metadata id, and the actual information can be linked
/// to the `MetaId`.
///
/// Internally, this is represented using a `usize`. The value is unique within
/// a scope that must be documented by functions issuing these metadata ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaId(usize);

impl MetaId {
  /// Wrap the provided `usize` into a `MetaId`.
  pub const fn into_usize(self) -> usize {
    self.0
  }

  /// Retrieve the inner `usize` from this `MetaId`.
  pub const fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

/// A rich value of type `T`, attached to metadata of type `M`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rich<T, M> {
  /// The primary data for this `Rich` value.
  pub value: T,
  /// Metadata providing additional information about `value`.
  pub meta: M,
}

impl<T, M> Rich<T, M> {
  /// Create a [`Rich`] value, by attaching metadata to a value.
  pub const fn new(value: T, meta: M) -> Self {
    Self { value, meta }
  }

  /// Map both the value and metadata to references
  pub const fn as_ref(&self) -> Rich<&T, &M> {
    Rich {
      value: &self.value,
      meta: &self.meta,
    }
  }

  /// Create a new `Rich`, by mapping the `value` field.
  ///
  /// The `meta` field is kept unchanged.
  pub fn map_value<U, F>(self, f: F) -> Rich<U, M>
  where
    F: FnOnce(T) -> U,
  {
    Rich {
      value: f(self.value),
      meta: self.meta,
    }
  }

  /// Create a new `Rich`, by mapping the `meta` field.
  ///
  /// The `value` field is kept unchanged.
  pub fn map_meta<N, F>(self, f: F) -> Rich<T, N>
  where
    F: FnOnce(M) -> N,
  {
    Rich {
      value: self.value,
      meta: f(self.meta),
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

/// Trait marking types supporting structural projection into a container for
/// metadata of type `M` for each subcomponent.
///
/// This is equivalent to `StructuralProjection<ConstProjector<M>>`. For types
/// implementing `StructuralProjection` too, it is recommended to have
/// `Meta` equal to `<Self as StructuralProjection<ConstProjector<M>>>::Projection`.
///
/// TODO: remove this trait
pub trait MetaProjection<M> {
  /// Projection result: type holding nested metadata.
  ///
  /// This should be a structural projection of `Self` where each descendent
  /// component is mapped such that leaf components have type `M`.
  type Meta;
}

/// Helper type alias allowing to extract the `Meta` type out of a rich value.
#[expect(
  type_alias_bounds,
  reason = "even if it's not enforced yet (see <https://github.com/rust-lang/rust/issues/112792>) the type bound serves as documentation"
)]
pub type MetaFor<T, M>
where
  T: MetaProjection<M>,
= <T as MetaProjection<M>>::Meta;

impl<M> MetaProjection<M> for () {
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M, T0> MetaProjection<M> for (T0,) {
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M, T0, T1> MetaProjection<M> for (T0, T1) {
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M, T0, T1, T2> MetaProjection<M> for (T0, T1, T2) {
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M> MetaProjection<M> for bool {
  // type Meta = ()
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M> MetaProjection<M> for u32 {
  // type Meta = ()
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M> MetaProjection<M> for String {
  // type Meta = ()
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

impl<M, T> MetaProjection<M> for Vec<T> {
  // type Meta = Vec<M>
  type Meta = <Self as StructuralProjection<ConstProjector<M>>>::Projection;
}

/// Structure holding both overall metadata for a value, and for its child sub-components.
///
/// This is equivalent to [`Rich`], the only difference is semantics.
/// - `meta` has the same meaning: it's the metadata for the main value
/// - The second field has different semantics:
///   - In [`Rich`], it's the value being described
///   - In [`TreeMeta`], it's more metadata: for sub-components of the main value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeMeta<MainMeta, NestedMeta> {
  /// Metadata id for this level of the hierarchy
  meta: MainMeta,
  /// Nested metadata
  nested: NestedMeta,
}

impl<MainMeta, NestedMeta> TreeMeta<MainMeta, NestedMeta> {
  /// Create a [`TreeMeta`] value, by linking metadata for the main value and for its sub-components.
  pub const fn new(meta: MainMeta, nested: NestedMeta) -> Self {
    Self { meta, nested }
  }
}

/// Projector recursively converting each component type into a `TreeMeta` for holding metadata of type `M`.
pub struct TreeMetaProjector<M>(PhantomData<M>, core::convert::Infallible);

impl<TyInput, M> Projector<TyInput> for TreeMetaProjector<M>
where
  TyInput: StructuralProjection<TreeMetaProjector<M>>,
{
  type Output = TreeMeta<M, <TyInput as StructuralProjection<TreeMetaProjector<M>>>::Projection>;
}

/// Trait marking types supporting structural projection into a recursive
/// `TreeMeta` container for metadata of type `M` for each subcomponent.
///
/// This is equivalent to `StructuralProjection<TreeMetaProjector<M>>`. For types
/// implementing `StructuralProjection` too, it is recommended to have
/// `Meta` equal to `<Self as StructuralProjection<TreeMetaProjector<M>>>::Projection`.
///
// TODO: rename to `MetaProjection`
pub trait TreeMetaProjection<M> {
  /// Projection result: type holding recursive nested metadata.
  ///
  /// This should be a structural projection of `Self` where each subcomponent
  /// is a `TreeMeta` holding metadata of type M for the component itself, and
  /// a subtree of metadata for nested components.
  type TreeMeta;
}

impl<M> TreeMetaProjection<M> for () {
  // type TreeMeta = ()
  type TreeMeta = <Self as StructuralProjection<TreeMetaProjector<M>>>::Projection;
}

impl<M> TreeMetaProjection<M> for bool {
  // type TreeMeta = ()
  type TreeMeta = <Self as StructuralProjection<TreeMetaProjector<M>>>::Projection;
}

impl<M> TreeMetaProjection<M> for u32 {
  // type TreeMeta = ()
  type TreeMeta = <Self as StructuralProjection<TreeMetaProjector<M>>>::Projection;
}

impl<M, T> TreeMetaProjection<M> for Vec<T>
where
  T: TreeMetaProjection<M>,
{
  type TreeMeta = Vec<TreeMeta<M, T::TreeMeta>>;
}

/// Convert a rich value using (potentially nested) internal metadata
/// representation into a pair of pure data and pure external metadata.
///
/// The shape of the external metadata must be the same as the primary
/// value, except the each child component should store metadata for the
/// corresponding sub-component in the primary value. In other words, the
/// metadata type must be a structural projection of the primary alue through
/// the `TreeMetaProjector` operation.
///
/// For convenience, this trait can also be implemented on regular values that
/// don't contain any metadata. In this case, the semantics should be to produce
/// a unit `()` for the metadata.
pub trait SplitMeta<M> {
  type Value: TreeMetaProjection<M>;

  fn split_meta(self) -> Rich<Self::Value, <Self::Value as TreeMetaProjection<M>>::TreeMeta>;
}

/// Wrapper for structural projection primitives used inside rich values with
/// internal metadata.
///
/// Structural projection primitives are types which always produce `()` when
/// structurally projected (no inner structure).
///
/// In practice, those types could implement `SplitMeta` directly. This wrapper
/// is present for some uniform handling without having to impl the trait
/// manually. It may not be needed in practice (it will be removed then).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RichPrimitive<T>(pub T);

impl<T> RichPrimitive<T> {
  pub const fn as_ref(&self) -> &T {
    &self.0
  }
}

impl<T, M> SplitMeta<M> for RichPrimitive<T>
where
  T: TreeMetaProjection<M, TreeMeta = ()>,
{
  type Value = T;

  fn split_meta(self) -> Rich<Self::Value, ()> {
    Rich::new(self.0, ())
  }
}

impl<M> SplitMeta<M> for bool {
  type Value = Self;

  fn split_meta(self) -> Rich<Self::Value, ()> {
    Rich::new(self, ())
  }
}

impl<M> SplitMeta<M> for u32 {
  type Value = Self;

  fn split_meta(self) -> Rich<Self::Value, ()> {
    Rich::new(self, ())
  }
}

// impl<M> SplitMeta<M> for String {
//   type Value = Self;
//
//   fn split_meta(self) -> Rich<Self::Value, ()> {
//     Rich::new(self, ())
//   }
// }
//
// pub trait IsA<T> {}
//
// impl<T> IsA<T> for T {}
//
// pub trait MergeMeta<TyMeta>
// where
//   Self: StructuralProjection<ConstProjector<MetaId>>,
//   Self::Rich: SplitMeta<Value=Self>,
// {
//   type Rich;
//
//   fn merge_meta(self, meta: TyMeta) -> Self::Rich;
// }
//
impl<T, M> Rich<T, M>
where
  T: SplitMeta<M>,
{
  /// Convert a rich holding a `T` with internal metadata into an external
  /// metadata representation with pure data and pure metadata.
  // #[expect(
  //   clippy::type_complexity,
  //   reason = "keeping the signature self-contained is valuable"
  // )]
  pub fn deep_split_meta(self) -> Rich<T::Value, TreeMeta<M, <T::Value as TreeMetaProjection<M>>::TreeMeta>> {
    let value_and_nested_meta: Rich<T::Value, _> = self.value.split_meta();
    Rich::new(
      value_and_nested_meta.value,
      TreeMeta::new(self.meta, value_and_nested_meta.meta),
    )
  }
}

// impl<T, M> SplitMeta<M> for Rich<T, M>
// where
//   T: StructuralProjection<TreeMetaProjector<M>, Projection = ()>, // <Self::Value as StructuralProjection<TreeMetaProjector<M>>>::Projection>
// {
//   type Value = T;
//
//   fn split_meta(self) -> Rich<T, ()> {
//     Rich::new(self.value, ())
//   }
// }

// // /// Represents a metadata node in a structured hierarchy
// // #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// // pub struct WrappedMeta<N> {
// //   id: MetaId,
// //   nested: N,
// // }
// //
// // impl<N> WrappedMeta<N> {
// //   /// Create a new [WrappedMeta].
// //   pub const fn new(nested: N, id: MetaId) -> Self {
// //     Self { nested, id }
// //   }
// // }
// //
// // #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// // pub struct BoolView<'rich>(Rich<&'rich bool, &'rich WrappedMeta<Meta<bool, MetaId>>>);
// //
// // impl<'rich> BoolView<'rich> {
// //   pub fn new(rich: Rich<&'rich bool, &'rich WrappedMeta<Meta<bool, MetaId>>>) -> Self {
// //     Self(rich)
// //   }
// // }
//
#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct Mascot {
    is_crab: bool,
    price: u32,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct MascotMeta<M> {
    is_crab: TreeMeta<M, ()>,
    price: TreeMeta<M, ()>,
  }

  impl<M> TreeMetaProjection<M> for Mascot {
    type TreeMeta = MascotMeta<M>;
  }

  // #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  // struct MascotMeta {
  //   is_crab: TreeMeta<MetaId, Meta<bool, MetaId>>,
  //   price: TreeMeta<MetaId, Meta<u32, MetaId>>,
  // }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct RichMascot {
    is_crab: Rich<bool, MetaId>,
    price: Rich<u32, MetaId>,
  }

  impl SplitMeta<MetaId> for RichMascot {
    type Value = Mascot;

    fn split_meta(self) -> Rich<Self::Value, MascotMeta<MetaId>> {
      let is_crab: Rich<bool, _> = self.is_crab.deep_split_meta();
      let price: Rich<u32, _> = self.price.deep_split_meta();
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
      TreeMeta::new(
        MetaId(3),
        MascotMeta {
          is_crab: TreeMeta::new(MetaId(1), ()),
          price: TreeMeta::new(MetaId(2), ()),
        },
      ),
    );

    assert_eq!(actual, expected);
  }
}
