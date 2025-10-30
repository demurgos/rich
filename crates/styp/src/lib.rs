//! # `styp`
//!
//! `styp` is a helper library for structural typing when using Rust.
//!
//! The main feature currently is the [`StructuralProjection`] trait used to
//! implement mapped types.

use core::marker::PhantomData;

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
///   pub fn map<F>(self, f: F) -> Self
///     where F: Fn(u64) -> u64
///   {
///     Self {
///       x: f(self.x),
///       y: f(self.y),
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
/// #   pub fn map<F>(self, f: F) -> Self
/// #     where F: Fn(u64) -> u64
/// #   {
/// #     Self {
/// #       x: f(self.x),
/// #       y: f(self.y),
/// #     }
/// #   }
/// # }
///
/// let base: Vec2D = Vec2D { x: 3, y: 6 };
///
/// fn double(x: u64) -> u64 {
///   x * 2
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
///   pub fn map<F, U>(self, f: F) -> Vec2D<U>
///     where F: Fn(T) -> U
///   {
///     Vec2D {
///       x: f(self.x),
///       y: f(self.y),
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
/// struct Pressure(pub u64);
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Temperature {
///   Cold,
///   Nice,
///   Hot,
/// }
///
/// struct WeatherMeasurement {
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
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
///
/// struct WeatherMeasurementVec {
///   pub temperature: Vec<Temperature>,
///   pub pressure: Vec<Pressure>,
/// }
/// ```
///
/// It would also be useful to have a version using slices to avoid cloning.
///
/// ```rust
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
///
/// struct WeatherMeasurementSlice<'a> {
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
/// use styp::{Project, SliceProjector, VecProjector};
///
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
/// #
/// # struct WeatherMeasurement {
/// #   pub temperature: Temperature,
/// #   pub pressure: Pressure,
/// # }
///
/// // These types are equivalent to the manually written ones.
/// type WeatherMeasurementVec = Project<WeatherMeasurement, VecProjector>;
/// type WeatherMeasurementSlice<'a> = Project<WeatherMeasurement, SliceProjector<'a>>;
/// ```
///
/// Another good use-case it to turn a structure into a "view type" of references.
/// For example, type projection could allow to define the following types
/// automatically:
///
/// ```rust
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
///
/// struct WeatherMeasurementRef<'a> {
///   pub temperature: &'a Temperature,
///   pub pressure: &'a Pressure,
/// }
///
/// struct WeatherMeasurementMut<'a> {
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
/// pub trait Projector<TyInput> {
///   type Output;
/// }
/// ```
///
/// And here is how we can define projectors for the different transformations
/// discussed in the previous section.
///
/// ```
/// # use core::marker::PhantomData;
/// # use styp::Projector;
///
/// pub enum VecProjector {}
///
/// impl<TyInput> Projector<TyInput> for VecProjector {
///   type Output = Vec<TyInput>;
/// }
///
/// pub struct SliceProjector<'a>(PhantomData<&'a ()>, core::convert::Infallible);
///
/// impl<'a, TyInput: 'a> Projector<TyInput> for SliceProjector<'a> {
///   type Output = &'a [TyInput];
/// }
///
/// pub struct RefProjector<'a>(PhantomData<&'a ()>, core::convert::Infallible);
///
/// impl<'a, TyInput: 'a> Projector<TyInput> for RefProjector<'a> {
///   type Output = &'a TyInput;
/// }
///
/// pub struct RefMutProjector<'a>(PhantomData<&'a mut ()>, core::convert::Infallible);
///
/// impl<'a, TyInput: 'a> Projector<TyInput> for RefMutProjector<'a> {
///   type Output = &'a mut TyInput;
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
/// pub trait StructuralProjection<TyProjector> {
///   type Projection;
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
/// # use styp::{Projector, StructuralProjection};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
/// #
/// # struct WeatherMeasurement {
/// #   pub temperature: Temperature,
/// #   pub pressure: Pressure,
/// # }
/// #
/// # pub struct WeatherMeasurementStructure<TyTemperature, TyPressure> {
/// #   pub temperature: TyTemperature,
/// #   pub pressure: TyPressure,
/// # }
///
/// impl<TyProjector> StructuralProjection<TyProjector> for WeatherMeasurement
///   where TyProjector: Projector<Temperature> + Projector<Pressure>
/// {
///   type Projection = WeatherMeasurementStructure<
///     <TyProjector as Projector<Temperature>>::Output,
///     <TyProjector as Projector<Pressure>>::Output,
///   >;
/// }
/// ```
///
/// This implementation can be used as follows:
/// ```rust
/// # use styp::{Projector, StructuralProjection, VecProjector};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
/// #
/// # struct WeatherMeasurement {
/// #   pub temperature: Temperature,
/// #   pub pressure: Pressure,
/// # }
/// #
/// # pub struct WeatherMeasurementStructure<TyTemperature, TyPressure> {
/// #   pub temperature: TyTemperature,
/// #   pub pressure: TyPressure,
/// # }
/// #
/// # impl<TyProjector> StructuralProjection<TyProjector> for WeatherMeasurement
/// #   where TyProjector: Projector<Temperature> + Projector<Pressure>
/// # {
/// #   type Projection = WeatherMeasurementStructure<
/// #     <TyProjector as Projector<Temperature>>::Output,
/// #     <TyProjector as Projector<Pressure>>::Output,
/// #   >;
/// # }
///
/// type WeatherMeasurementVec = <WeatherMeasurement as StructuralProjection<VecProjector>>::Projection;
/// ```
///
/// For ergonomics, we can define a type alias helper:
/// ```
/// # use styp::{StructuralProjection};
/// #
/// # #[expect(type_alias_bounds, reason = "the bounds provide hints about usage")]
/// pub type Project<Base, Projector>
/// where
///   Base: StructuralProjection<Projector>,
/// = <Base as StructuralProjection<Projector>>::Projection;
/// ```
///
/// And finally get the wanted types:
/// ```rust
/// # use styp::{Project, Projector, StructuralProjection, VecProjector};
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # struct Pressure(pub u64);
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// # enum Temperature {
/// #   Cold,
/// #   Nice,
/// #   Hot,
/// # }
/// #
/// # struct WeatherMeasurement {
/// #   pub temperature: Temperature,
/// #   pub pressure: Pressure,
/// # }
/// #
/// # pub struct WeatherMeasurementStructure<TyTemperature, TyPressure> {
/// #   pub temperature: TyTemperature,
/// #   pub pressure: TyPressure,
/// # }
/// #
/// # impl<TyProjector> StructuralProjection<TyProjector> for WeatherMeasurement
/// #   where TyProjector: Projector<Temperature> + Projector<Pressure>
/// # {
/// #   type Projection = WeatherMeasurementStructure<
/// #     <TyProjector as Projector<Temperature>>::Output,
/// #     <TyProjector as Projector<Pressure>>::Output,
/// #   >;
/// # }
///
/// type WeatherMeasurementVec = Project<WeatherMeasurement, VecProjector>;
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

/// Trait representing a structural projection operator.
///
/// This trait allows to map an input type `TyInput` into the output type
/// `Self::Output`. See implementations in this module for examples and they may
/// be used.
///
/// Types implementing this trait should be inhabited as this trait is intended
/// for type-level manipulation only and has no use with runtime values.
/// Inhabited types are usually defined using an empty enum, or a struct
/// containing an empty enum or the `never` type.
pub trait Projector<TyInput> {
  type Output;
}

/// Helper type alias to apply a structural projector to a base type.
#[expect(
  type_alias_bounds,
  reason = "even if it's not enforced yet (see <https://github.com/rust-lang/rust/issues/112792>) the type bound serves as documentation"
)]
pub type Project<Base, Projector>
where
  Base: StructuralProjection<Projector>,
= <Base as StructuralProjection<Projector>>::Projection;

/// Structural projector that maps all component types to a unique constant
/// type.
///
/// # Example
///
/// ```rust
/// use styp::{ConstProjector, Project, Projector, StructuralProjection};
///
/// pub struct WeatherMeasurementStructure<TyTemperature, TyPressure> {
///   pub temperature: TyTemperature,
///   pub pressure: TyPressure,
/// }
///
/// impl<TyProjector, TyTemperature, TyPressure> StructuralProjection<TyProjector> for WeatherMeasurementStructure<TyTemperature, TyPressure>
///   where TyProjector: Projector<TyTemperature> + Projector<TyPressure>
/// {
///   type Projection = WeatherMeasurementStructure<
///     <TyProjector as Projector<TyTemperature>>::Output,
///     <TyProjector as Projector<TyPressure>>::Output,
///   >;
/// }
///
/// type WeatherMeasurement = WeatherMeasurementStructure<i16, u32>;
/// let measure = WeatherMeasurement {
///   temperature: 23i16,
///   pressure: 101250u32,
/// };
///
/// type WeatherTime = Project<WeatherMeasurement, ConstProjector<std::time::SystemTime>>;
/// let time = WeatherTime {
///   temperature: std::time::SystemTime::now(),
///   pressure: std::time::SystemTime::now(),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstProjector<T>(PhantomData<T>, core::convert::Infallible);

impl<TyInput, T> Projector<TyInput> for ConstProjector<T> {
  type Output = T;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VecProjector {}

impl<TyInput> Projector<TyInput> for VecProjector {
  type Output = Vec<TyInput>;
}

pub struct SliceProjector<'a>(PhantomData<&'a ()>, core::convert::Infallible);

impl<'a, TyInput: 'a> Projector<TyInput> for SliceProjector<'a> {
  type Output = &'a [TyInput];
}

pub struct RefProjector<'a>(PhantomData<&'a ()>, core::convert::Infallible);

impl<'a, TyInput: 'a> Projector<TyInput> for RefProjector<'a> {
  type Output = &'a TyInput;
}

pub struct RefMutProjector<'a>(PhantomData<&'a mut ()>, core::convert::Infallible);

impl<'a, TyInput: 'a> Projector<TyInput> for RefMutProjector<'a> {
  type Output = &'a mut TyInput;
}

impl<TyProjector> StructuralProjection<TyProjector> for () {
  type Projection = ();
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