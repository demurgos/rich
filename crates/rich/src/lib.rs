//! This module defines the [`Data`] struct. It is an internal implementation
//! that should not be relied on by external code.

use std::marker::PhantomData;

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
  pub const fn as_ref<'rich>(&'rich self) -> Rich<&'rich T, &'rich M> {
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

/// Trait marking types supporting structural projection.
///
/// Structural projection is type-level operation creating a new type by mapping
/// the type of each "component" forming the structure of the original type.
///
/// # General structural projection
///
/// ## Mapping values for a simple type
///
/// A good way to understand structural type projection is to compare it to
/// value mapping. Let's say that we have struct type representing a 2D vector.
/// We may add a method to perform component-wise mapping:
///
/// ```rust
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// struct Vec2D {
///   pub x: u64,
///   pub y: u64,
/// }
///
/// impl Vec2D {
///   pub fn map<F>(self f: F) -> Self
///     where F: Fn(u64) -> u64
///     {
///       Self {
///         x: f(self.x),
///         y: f(self.y),
///       }
///     }
///   }
/// }
/// ```
///
/// At this stage, we can define a _base_ value and an operator transforming the
/// value of each component, for example the `double` function defined as
/// doubling the value. We can use the `map` method to apply this operator on
/// all the components and created a _mapped_ value this way.
///
/// ```rust
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Vec2D {
/// #   pub x: u64,
/// #   pub y: u64,
/// # }
/// #
/// # impl Vec2D {
/// #   pub fn map<F>(self f: F) -> Self
/// #     where F: Fn(u64) -> u64
/// #     {
/// #       Self {
/// #         x: f(self.x),
/// #         y: f(self.y),
/// #       }
/// #     }
/// #   }
/// # }
/// let base: Vec2D = Vec2D { x: 3, y: 6 };
///
/// fn double(x: u64) -> u64 {
///  x * 2
/// }
///
/// let mapped: Vec2D = base.map(double);
/// assert_eq!(mapped, Vec2D { x: 6, y: 12 });
/// ```
///
/// ## Mapping values for a higher-order type
///
/// If the base type is generic over a type parameter (called a "higher order
/// type"), the `map` function could even produce values of a different type.
/// You should be familiar with this as this is how the [`Iterator::map`]
/// method works. Continuing from the previous example, we could make the vector
/// generic and map it with an operator checking if the initial value was even,
/// turning each component into a `bool`.
///
/// ```rust
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// struct Vec2D<T> {
///   pub x: T,
///   pub y: T,
/// }
///
/// impl<T> Vec2D<T> {
///   pub fn map<F, U>(self f: F) -> Vec2D<U>
///     where F: Fn(T) -> U
///     {
///       Self {
///         x: f(self.x),
///         y: f(self.y),
///       }
///     }
///   }
/// }
///
/// let base: Vec2D<u64> = Vec2D { x: 3, y: 6 };
///
/// fn is_even(x: u64) -> bool {
///   x.is_multiple_of(2)
/// }
///
/// let mapped: Vec2D<bool> = base.map(is_even);
/// assert_eq!(mapped, Vec2D { x: false, y: true });
/// ```
///
/// So far, all these mappings happen at the value level. The operator receives
/// an input value and transforms it into an output value. Within a `.map` call,
/// the input and output types are always the same, but the value changes.
///
/// In particular, in our last example, all the components of the vector must
/// have the same type.
///
/// ## Type projection motivation
///
/// The benefit of structural projection is to handle mapping of structures with
/// _heterogeneous_ types for each component. To see this in action, we need to
/// define a structure with fields having different types. For example, let's
/// say that we do weather measurements and want to record the temperature and
/// pressure at a given time. These two quantities will be represented by
/// different types.
///
/// ```rust
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// struct Pressure(pub u64),
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Temperature {
///   Cold,
///   Nice,
///   Hot,
/// }
///
/// WeatherMeasurement {
///   pub temperature: Temperature,
///   pub pressure: Pressure,
/// }
/// ```
///
/// Now, let's imagine that we actually want to store a list of measurements.
/// We could use a `Vec<WeatherMeasurement>`, but our program may instead prefer
/// a more column-oriented approach using a [Structure of Arrays](https://en.wikipedia.org/wiki/AoS_and_SoA)
/// representation. Here is how it would look like:
///
/// ```rust
/// WeatherMeasurementVec {
///   pub temperature: Vec<Temperature>,
///   pub pressure: Vec<Pressure>,
/// }
/// ```
///
/// It would also be useful to have a version using slices to avoid cloning.
///
/// ```rust
/// WeatherMeasurementSlice<'a> {
///   pub temperature: &'a [Temperature],
///   pub pressure: &'a [Pressure],
/// }
/// ```
///
/// We wrote the types `WeatherMeasurementVec` and `WeatherMeasurementSlice`
/// manually. However, in some sense these are not primary types, but rather
/// they are derived from the original `WeatherMeasurement` type by applying
/// some _type-level_ transformation. For `WeatherMeasurementVec`, we wrap
/// each component type in a `Vec`, and for `WeatherMeasurementSlice`, we use
/// a slice of the corresponding original type.
///
/// Structural projection enables to define the types above as transformations
/// of the original type.
///
/// ```rust
/// // These types are equivalent to the manually written ones.
/// type WeatherMeasurementVec = Project<WeatherMeasurementVec, IntoVec>;
/// type WeatherMeasurementSlice<'a> = Project<WeatherMeasurementVec, IntoSlice<'a>>;
/// ```
///
/// Another good use-case it to turn a structure into a "view type" of references.
/// For example, type projection could allow to define the following types
/// automatically:
///
/// ```rust
/// WeatherMeasurementRef<'a> {
///   pub temperature: &'a Temperature,
///   pub pressure: &'a Pressure,
/// }
///
/// WeatherMeasurementMut<'a> {
///   pub temperature: &'a mut Temperature,
///   pub pressure: &'a mut Pressure,
/// }
/// ```
///
/// In short, structural projection allows to abstract type-level
/// transformations applied to each sub-component type.
///
/// The fact that the transformation
/// is applied to the type of each sub-component is why it is structural. The
/// projection is transforming the type of each field. It is not transforming
/// the container type directly. This is similar to how `Vec2D::map` was
/// transforming the _value_ of each component, not the `Vec2D` type itself.
///
/// ## Defining an operator
///
/// With value mapping, there were two parts: the `map` method driving the
/// transformation and the `double` or `is_even` functions defining the
/// transformation to apply.
///
/// For structural projection, we also have these two parts. Let's start with
/// defining the operator. This crate calls it a [`Projector`]. Its role is to
/// take an input _type_ and turn it into an output _type_.
///
/// In Rust, the signature of a type-level functions is represented as a trait with
/// an associated type. For our use-case, the associated type should accept a
/// generic type parameter acting as the input and turn it into an output type.
/// The output type is controlled by the concrete projector implementation.
///
/// That's a lot of words, let's see an example. Here is the `Projector` trait:
/// ```rust
/// pub trait Projector {
///   type Apply<TyInput>;
/// }
/// ```
///
/// And here is how we can define projectors for the different transformations
/// discussed in the previous section.
///
/// ```
/// pub enum IntoVec;
///
/// impl Projector for IntoVec {
///   type Apply<TyInput> = Vec<TyInput>;
/// }
///
/// pub struct IntoSlice<'a>(PhantomData<&'a ()>, core::convert::Infallible);
///
/// impl<'a> Projector for IntoSlice<'a> {
///   type Apply<TyInput> = &'a [TyInput];
/// }
///
/// pub struct IntoRef<'a>(PhantomData<&'a ()>, core::convert::Infallible);
///
/// impl<'a> Projector for IntoRef<'a> {
///   type Apply<TyInput> = &'a TyInput;
/// }
///
/// pub struct IntoMut<'a>(PhantomData<&'a mut ()>, core::convert::Infallible);
///
/// impl<'a> Projector for IntoMut<'a> {
///   type Apply<TyInput> = &'a mut TyInput;
/// }
/// ```
///
/// The `IntoVec` projector should be pretty straightforward. The other
/// projectors are a bit more complex as they need to carry a lifetime parameter.
/// The `core::convert::Infallible` type or empty enum definition ensures that
/// these types can't be instantiated as concrete values: they exist purely for
/// type-level transformations.
///
/// ## Implementing structural projection
///
/// Most pieces are now in place, the last remaining part is to actually
/// implement the structural projection. Structural projection is another type-
/// level function so we'll represent it as a trait. The input is a
/// projector and the output is the transformed base type, where each field is
/// projected.
///
/// ```
/// pub trait StructuralProjection {
///   type Project<TyProjector>
///     where TyProjector: Projector;
/// }
/// ```
///
/// Since each field can be projected to an arbitrary type, we need to define a
/// generic struct which has the same shape as `WeatherMeasurement` but where
/// each field is controlled by a generic type parameter.
///
/// ```rust
/// pub struct WeatherMeasurementStructure<TyTemperature, TyPressure> {
///   pub temperature: TyTemperature,
///   pub pressure: TyPressure,
/// }
/// ```
///
/// We can finally implement the `StructuralProjection` trait for `WeatherMeasurement`:
///
/// ```rust
/// impl StructuralProjection for WeatherMeasurement {
///   type Project<TyProjector> = WeatherMeasurementStructure<
///     TyProjector::Apply<Temperature>,
///     TyProjector::Apply<Pressure>
///   >
///     where TyProjector: Projector;
/// }
/// ```
///
/// This implementation can be used as follows:
/// ```rust
/// type WeatherMeasurementVec = <WeatherMeasurement as StructuralProjection>::Project<IntoVec>;
/// ```
///
/// For ergonomics, we can define a type alias helper:
/// ```
/// pub type Project<T: StructuralProjection, P: Projector> = <T as StructuralProjection>::Project<P>;
/// ```
///
/// And finally get the wanted types:
/// ```rust
/// type WeatherMeasurementVec = Project<WeatherMeasurementVec, IntoVec>;
/// ```
///
/// ## Extensions
///
/// With all the scaffolding built in this example, we don't even need the
/// original type anymore. Using `WeatherMeasurementStructure`, we can define
/// the original type as `type WeatherMeasurement = WeatherMeasurementStructure<Temperature, Pressure>;`.
///
/// The `StructuralProjection` trait could also be implemented for `WeatherMeasurementStructure`
/// to enable composition of projection.
///
/// # Structural projection in `rich`
///
/// `rich` uses a restricted form of structural projection where the
/// `Projector` must be constant with regard to the input type. Since type-level
/// functions are pure, this means that the output type is also constant. This
/// restricted form of structural projection means that the output type of the
/// projection no longer needs one generic type parameter per field, instead
/// a single field is enough. Another important property is that it makes it
/// simpler to perform recursive projection.
///
/// In practice, it means that you can replace the types of all the fields
/// inside a struct by a single type. For rich, this is the _metadata_ type
/// for this field in a value of the base type.
///
/// # Structural projection outside structs
///
/// The structural projection section focused on structs since it's easier to
/// understand what's a component of the "structure": a simple field. The
/// meaning of a sub-component can be a bit harder for other kinds of types.
/// Ultimately, the meaning of a component is up to the author of the original
/// type and what kind of model they want to expose, here are some guidelines
/// however.
///
/// - Tuples are equivalent to anonymous structs, each of their component
///   should be projected.
/// - The unit type `()` should always be projected into `()` regardless of
///   the projector.
/// - Primitives have no internal sub-component, as such they are akin to `()`
///   and should always project to `()`.
/// - Collections such as `Vec<T>` should behave as a uniform tuple `(T, T, ..., T)`
///   so they should be projected into `Vec<Projector::Apply<T>>` or similar.
/// - A map is a collection of `(K, V)` pairs.
/// - Sometimes it's up to the author to make a judgement call. For example,
///   should `String` value be treated as primitives or a collection of `char`?
/// - Enums are tricky
/// - Recursion is up to the author. Ideally a type would provide the choice
///   between single-level or recursive projection.
pub trait StructuralProjection<TyProjector> {
  /// Type of extracted metadata.
  ///
  /// If `Self` is not a rich value, this should be `()` (or `EmptyMeta`?)
  type Projection;
}

pub trait Projector<TyInput> {
  type Output;
}

pub struct ConstProjector<T>(PhantomData<T>, core::convert::Infallible);

impl<TyInput, T> Projector<TyInput> for ConstProjector<T> {
  type Output = T;
}

/// Helper type alias allowing to extract the `Meta` type out of a rich value.
#[expect(
  type_alias_bounds,
  reason = "even if it's not enforced yet (see <https://github.com/rust-lang/rust/issues/112792>) the type bound serves as documentation"
)]
pub type Meta<T, M>
where
  T: StructuralProjection<ConstProjector<M>>,
= <T as StructuralProjection<ConstProjector<M>>>::Projection;

impl<TyProjector> StructuralProjection<TyProjector> for () {
  type Projection = ();
}

impl<TyProjector, T0> StructuralProjection<TyProjector> for (T0,)
where
  TyProjector: Projector<T0>,
{
  type Projection = (<TyProjector as Projector<T0>>::Output,);
}

impl<TyProjector, T0, T1> StructuralProjection<TyProjector> for (T0, T1)
where
  TyProjector: Projector<T0> + Projector<T1>,
{
  type Projection = (
    <TyProjector as Projector<T0>>::Output,
    <TyProjector as Projector<T1>>::Output,
  );
}

impl<TyProjector, T0, T1, T2> StructuralProjection<TyProjector> for (T0, T1, T2)
where
  TyProjector: Projector<T0> + Projector<T1> + Projector<T2>,
{
  type Projection = (
    <TyProjector as Projector<T0>>::Output,
    <TyProjector as Projector<T1>>::Output,
    <TyProjector as Projector<T2>>::Output,
  );
}

impl<TyProjector> StructuralProjection<TyProjector> for bool {
  type Projection = ();
}

impl<TyProjector> StructuralProjection<TyProjector> for u32 {
  type Projection = ();
}

impl<TyProjector> StructuralProjection<TyProjector> for String {
  type Projection = ();
}

impl<TyProjector, T> StructuralProjection<TyProjector> for Vec<T>
where
  TyProjector: Projector<T>,
{
  type Projection = Vec<<TyProjector as Projector<T>>::Output>;
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
  type Value: StructuralProjection<TreeMetaProjector<M>>;

  fn split_meta(self) -> Rich<Self::Value, <Self::Value as StructuralProjection<TreeMetaProjector<M>>>::Projection>;
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
  pub fn as_ref(&self) -> &T {
    &self.0
  }
}

impl<T, M> SplitMeta<M> for RichPrimitive<T>
where
  T: StructuralProjection<TreeMetaProjector<M>, Projection = ()>,
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
  pub fn deep_split_meta(
    self,
  ) -> Rich<T::Value, TreeMeta<M, <T::Value as StructuralProjection<TreeMetaProjector<M>>>::Projection>> {
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
  struct MascotStructure<TyIsCrab, TyPrice> {
    is_crab: TyIsCrab,
    price: TyPrice,
  }

  impl<TyProjector> StructuralProjection<TyProjector> for Mascot
  where
    TyProjector: Projector<bool> + Projector<u32>,
  {
    type Projection =
      MascotStructure<<TyProjector as Projector<bool>>::Output, <TyProjector as Projector<u32>>::Output>;
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

    fn split_meta(self) -> Rich<Self::Value, MascotStructure<TreeMeta<MetaId, ()>, TreeMeta<MetaId, ()>>> {
      let is_crab: Rich<bool, _> = self.is_crab.deep_split_meta();
      let price: Rich<u32, _> = self.price.deep_split_meta();
      Rich::new(
        Mascot {
          is_crab: is_crab.value,
          price: price.value,
        },
        MascotStructure {
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
        MascotStructure {
          is_crab: TreeMeta::new(MetaId(1), ()),
          price: TreeMeta::new(MetaId(2), ()),
        },
      ),
    );

    assert_eq!(actual, expected);
  }
}
