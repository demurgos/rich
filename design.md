# Rich value and deserialization metadata value

This document describes the design considerations for the crates in this repo.

## Goal

The goal of this repo is to provide a generic mechanism to attach metadata to
Rust values. The use case is to keep track of source locations (metadata) when
parsing configuration files (values). Tracking the source of config values
enables better help messages.

## Metadata id

The core issue of attaching metadata to Rust values can be reduced to **attaching
a unique id to each value**. Once this issue is solved, a general metadata API
can be built on top.

The unique id can be a key into a map of metadata details, the unique id can be
turned into a handle referencing a metadata store, the unique id can be turned
into a generic for inlined metadata, etc. There are multiple possible designs,
but they all rely on being able to attach a unique id in the first place.

We'll start then by defining a `usize` new-type acting as our metadata id, and
see how we can attach it to values.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaId(usize);
```

## Primitive metadata

Let's start with simple values that will act as metadata primitives: `bool`,
`u32`, `String`. From the point of view of our metadata mechanism, these are
opaque atomic pieces of data.

These values are opaque, so the only way to enrich them is to create the
corresponding metadata on the side:

```rust
let is_crab = true;
let is_crab_meta = MetaId(1);

let price = 42;
let price_meta = MetaId(2);
```

An obvious next step is to pair the primitive value and its metadata into a
single rich value. The simplest would be a tuple:

```rust
let rich_is_crab = (is_crab, is_crab_meta);
let rich_price = (price, price_meta);
```

This lets us manipulate the rich value as unit, and extract the base value with
`.0` and metadata with `.1`.

To make it more readable, we can define an equivalent struct, so we can use
named fields:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Rich<T, M> {
  value: T,
  meta: M,
}

impl<T, M> Rich<T, M> { fn new(value: T, meta: M) -> Self { Self { value, meta }} }

let rich_is_crab = Rich::new(is_crab, is_crab_meta);
let rich_price = Rich::new(price, price_meta);

println!("the price is {}", rich_price.value);
println!("the price metadata is {:?}", rich_price.meta);
```

With this `Rich` struct, we can also add a bunch of helper methods or trait
implementations to provide a nicer API. However this is not a priority for now,
we can move to the next step.

## Structure metadata

Attaching metadata to an atomic value is not very interesting as it's fairly
straight forward to define a wrapper type with custom extra fields. The more
relevant scenario for a generic mechanism is how to handle a complex reachability
graph of values.

Let's start slowly with a simple struct that we want to enrich:

```rust
struct Mascot {
    is_crab: bool,
    price: u32,
}
```

### Internal metadata

The first solution is to wrap each field type with our `Rich` struct:

```rust
struct RichMascot {
  is_crab: Rich<bool, MetaId>,
  price: Rich<u32, MetaId>,
}

let is_crab = Rich::new(true, MetaId(1));
let price = Rich::new(42, MetaId(2));
let mascot = Rich::new(RichMascot { is_crab, price }, MetaId(3));
// we wrap `RichMascot` itself in a `Rich` struct, to support metadata at the
// struct level

println!("the mascot metadata is {:?}", mascot.meta);
println!("the price is {}", mascot.value.price.value);
println!("the price metadata is {:?}", mascot.value.price.meta);
```

In this approach, we modify the struct to place metadata directly inside. This
is the approach used by [serde_spanned](https://crates.io/crates/serde_spanned).

This solution has the benefit of being straight-forward: it's obviously clear
how to retrieve values and their corresponding metadata. The downside is that
metadata is now strongly coupled with our base value. We may only care about
metadata in the part of our code responsible for help messages, but now the
intertwining means that it's harder to pass a simpler `Mascot` value to deeper
levels of our program.

### External metadata

An alternative solution to internal metadata is to keep the value struct as-is,
and define a side-struct dedicated to metadata.

```rust
struct Mascot {
  is_crab: bool,
  price: u32,
}

struct MascotMeta {
    is_crab: MetaId,
    price: MetaId,
}

let value = Mascot { is_crab: true, price: 42 };
let meta = MascotMeta { is_crab: MetaId(1), price: MetaId(2) }
let mascot = Rich::new(value, meta);
let mascot_meta = MetaId(3);
// (we'll see how to attach metadata at the struct level in the next section)

println!("the price is {}", mascot.value.price);
println!("the price metadata is {:?}", mascot.meta.price);
```

I call this representation "external metadata" as the metadata now lives on the
side of the data. It has the benefit of keeping the value struct untouched. This
allows to use the rich mascot for helper messages, but use the simpler value
struct for our program internals.

The downside is that the metadata is further apart from the value. The values
and metadata are split at the start, and you then need to follow the same path
in "value side" and "metadata side" to get the corresponding data. With this
simple example it's not bad, but it may get more difficulty as the complexity
increases or the paths are no longer simple field look-ups.

Keeping the data and metadata separate is still a very nice feature despite the
downsides as it allows to add metadata incrementally, without major disruption.
The struct traversal and correlation of values with their metadata can probably
be improved through some helper view structs.

### Nesting

Let's increase the nesting level:

```rust
struct Language {
  name: String,
  mascot: Mascot,
}

struct Mascot {
  is_crab: bool,
  price: u32,
}
```

Here is how it would look like with internal metadata:

```rust
struct RichLanguage {
  name: Rich<String, MetaId>,
  mascot: Rich<RichMascot, MetaId>,
}

struct RichMascot {
  is_crab: Rich<bool, MetaId>,
  price: Rich<u32, MetaId>,
}

let is_crab = Rich::new(true, MetaId(1));
let price = Rich::new(42, MetaId(2));
let mascot = Rich::new(RichMascot { is_crab, price }, MetaId(3));
let name = Rich::new(String::from("Rust"), MetaId(4));
let language = Rich::new(RichLanguage { name, mascot }, MetaId(5));

println!("the language metadata is {:?}", language.meta);
println!("the mascot metadata is {:?}", language.value.mascot.meta);
println!("the price is {}", language.value.mascot.value.price.value);
println!("the price metadata is {:?}", language.value.mascot.value.price.meta);
```

How would we represent it using external metadata? The main issue is with the
`Language.mascot` field. We want to have access both to the overall mascot
metadata and to the inner metadata (for `is_crab` and `price`). We can try to
use a pair:

```rust
struct LanguageMeta {
  name: MetaId,
  mascot: (MascotMeta, MetaId),
}

struct MascotMeta {
    is_crab: MetaId,
    price: MetaId,
}
```

This design works, but it introduces an inconsistency between the metadata for
primitives and for structs. For a general mechanism, we would like a consistent
mechanism supporting both primitives and structs.

A possible solution would be to have a mechanism to get the struct for nested
metadata. For primitives it would return `()`, and for `Mascot` it would return
`MascotMeta`. Let's imagine that we have higher-kinded type function `NestedMeta`
that implements such mapping. We'll see later how to actually implement such
mapping. This mapper would let us define our external metadata as:

```rust
struct LanguageMeta {
  name: (NestedMeta<String>, MetaId),
  mascot: (NestedMeta<Mascot>, MetaId),
}

struct MascotMeta {
    is_crab: (NestedMeta<bool>, MetaId),
    price: (NestedMeta<u32>, MetaId),
}
```

Which would be equivalent to:

```rust
struct LanguageMeta {
  name: ((), MetaId),
  mascot: (MascotMeta, MetaId),
}

struct MascotMeta {
    is_crab: ((), MetaId),
    price: ((), MetaId),
}
```

For readability, we can replace the pair with an equivalent named struct:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct WrappedMeta<N> {
  nested: N,
  id: MetaId,
}

impl<N> WrappedMeta<N> { fn new(nested: N, id: MetaId) -> Self { Self { nested, id }} }
```

The full code for external metadata would then be:

```rust
struct Language {
  name: String,
  mascot: Mascot,
}

struct Mascot {
  is_crab: bool,
  price: u32,
}

struct LanguageMeta {
  name: WrappedMeta<()>,
  mascot: WrappedMeta<MascotMeta>,
}

struct MascotMeta {
  is_crab: WrappedMeta<()>,
  price: WrappedMeta<()>,
}

let value = Language {
  name: String::from("Rust"),
  mascot: Mascot { is_crab: true, price: 42 },
};
let meta = Wrapped::new(
  LanguageMeta {
    name: WrappedMeta::new((), 4),
    mascot: WrappedMeta::new(
      MascotMeta {
        is_crab: WrappedMeta::new((), 1),
        price: WrappedMeta::new((), 2),
      },
      MetaId(3),
    ),
  },
  MetaId(5),
);
let language = Rich::new(value, meta);

println!("the language metadata is {:?}", language.meta.id);
println!("the mascot metadata is {:?}", language.meta.mascot.id);
println!("the price is {}", language.value.mascot.price);
println!("the price metadata is {:?}", language.meta.mascot.price.id);
```

## Internal/External equivalence

So far, our internal and external representations are equivalent. They both
hold the same data, it's just organized differently. We have to axis to organize
our data: value/metadata and the field hierarchy.

In the internal representation, the primary axis is the field hierarchy, and
the secondary axis is the value/metadata split. In the external representation,
the primary axis is the value/metadata split and the secondary axis the field
hierarchy.

This is somewhat reminiscent of the [Array of Structures and Structure of Arrays](https://en.wikipedia.org/wiki/AoS_and_SoA)
equivalence.

It would be nice to provide some helpers to convert between one representation
and the other.

### Split metadata

Let's start with easier direction, going from an internal metadata
representation to an external metadata representation. In our example, this
means converting a `Rich<RichLanguage, MetaId>` into a
`Rich<Language, WrappedMeta<LanguageMeta>>`.

We are swapping the axis of our representation, but keeping the same field
hierarchy. This implies that a recursive approach should work well. We can
define a trait to extra nested metadata at a single leve:

```rust
/// Split a rich value into its simple value and nested metadata.
pub trait SplitMeta {
  type Value;
  type NestedMeta;
  
  fn split_meta(self) -> (Self::Value, Self::NestedMeta);
}
```

For primitives, there's no nested metadata:

```rust
impl SplitMeta for u32 {
  type Value = Self;
  type NestedMeta = ();

  fn split_meta(self) -> (Self::Value, Self::NestedMeta) {
    (self, ())
  }
}
// similar impl for `bool` and `String`
```

And for structs, we can implement it as the following, assuming `Rich::<T, MetaId>::deep_split_meta`
handles recursion.

```rust
impl SplitMeta for RichLanguage {
  type Value = Language;
  type NestedMeta = LanguageMeta;

  fn split_meta(self) -> (Self::Value, Self::NestedMeta) {
    let split_name: Rich<String, WrappedMeta<()>> = self.name.deep_split_meta();
    let split_mascot: Rich<Mascot, WrappedMeta<MascotMeta>> = self.mascot.deep_split_meta();
    let value = Language { name: split_name.value, mascot: split_mascot.meta };
    let meta = LanguageMeta { name: split_name.value, mascot: split_mascot.meta };
    (value, meta)
  }
}
```

And here is how we can implement `Rich::<T, MetaId>::deep_split_meta`:

```rust
impl<T> Rich<T, MetaId>
  where T: SplitMeta
{
  fn deep_split_meta(self) -> Rich<T::Value, WrappedMeta<T::Meta>> {
    let (value, nested) = self.value.split_meta();
    Rich::new(
      value,
      WrappedMeta::new(nested, self.meta)
    )
  }
}
```

### Merge metadata

We can now do the reverse transformation, where we go from an external metadata
representation to an internal one, by merging the base value and metadata into
a hierarchy of rich values.

Similarly, we can use a recursive approach. For consistency, we can require
that merging returns a value that can be split back (to ensure round-tripping).

```rust
trait MergeMeta<M>
where
{
  type Rich;
  
  fn merge_meta(self, meta: M) -> Self::Rich;
}
```

## Collections

Now that we have a good grasp over struct metadata, we can look into more
complex cases such as collections like `Vec<_>` or `HashMap<_>`.

```rust
struct Zoo {
  animal_names: Vec<String>,
  food_stock: HashMap<String, u32>,
}
```

We can attempt a fairly simple translation:

```rust
// Internal metadata
struct RichZoo {
  animal_names: Rich<Vec<Rich<String, MetaId>>, MetaId>,
  food_stock: Rich<HashMap<String, Rich<u32, MetaId>>, MetaId>,
}

// External metadata
struct Zoo {
  animal_names: Vec<String>,
  food_stock: HashMap<String, u32>,
}
struct ZooMeta {
  animal_names: WrapedMeta<Vec<WrappedMeta<()>>>,
  food_stock: WrapedMeta<HashMap<String, WrappedMeta<()>>>,
}
```

The mapping applied to the `animal_names` vec is fairly straight-forward. For
the internal metadata, there's nothing special to say about it: it just fits
with the pattern. For external metadata however, we lost an important property:
the type system no longer ensures that `Zoo.animal_names` has the same length
as `ZooMeta.animal_names`. It's a fundamental issues splitting this way. The
consequence is that matching the value and metadata is a fallible operation when
dynamically sized collections are involved.

The `food_stock` map suffers from the same risk of mismatch in the external
representation; but it also has another flaw: there is no support for attaching
metadata to the key.

In the internal case, we could wrap the key too in `Rich` (`food_stock: Rich<HashMap<Rich<String, MetaId>, Rich<u32, MetaId>>, MetaId>,`)
however this assumes that the `Hash` impl for `Rich` ignores the metadata and
delegates to the value. This is not the default behavior when deriving `Hash`
and does not match the behavior the anonymous pair `(value, meta)` that we
started with. For these reasons, we should probably use a representation closer
to a map of entries for the use-case of attaching metadata to keys:

```rust
struct RichEntry<V> {
  key_meta: MetaId,
  value: V,
}

struct RichZoo {
  animal_names: Rich<Vec<Rich<String, MetaId>>, MetaId>,
  food_stock: Rich<HashMap<String, RichEntry<Rich<u32, MetaId>>>, MetaId>,
}
```

We need a similar approach for the external representation to support key metadata:

```rust
struct EntryMeta<KeyMeta, ValueMeta> {
  key: KeyMeta,
  value: ValueMeta
}

struct ZooMeta {
  animal_names: WrapedMeta<Vec<WrappedMeta<()>>>,
  food_stock: WrapedMeta<HashMap<String, EntryMeta<WrappedMeta<()>, WrappedMeta<()>>>>,
}
```

There's also the alternative approach of only supporting metadata attached to
the value, and linking the key metadata to the `MetaId` of the value. This feels
too me that it would break the data hierarchy a bit too much as the map value
would become strongly attached to a specific key and couldn't be moved to a
different key without changing semantics. If the map is readonly, it may be
an acceptable approach.

## Enums

The last big category of data to cover are enums. The main question here is
if we want to attach data to the discriminant or not. The discriminant is mostly
an internal detail so it probably makes more sense to apply a simpler mapping
working on data fields only:

```rust
enum Operation {
  Read,
  Update { username: String, rate_limit: u32  },
  Delete(bool), // if true, `force-delete`
}

enum RichOperation {
  Read,
  Update { username: Rich<String, MetaId>, rate_limit: Rich<u32, MetaId> },
  Delete(Rich<bool, MetaId>)
}

enum OperationMeta {
  Read,
  Update { username: WrappedMeta<()>, rate_limit: WrappedMeta<()> },
  Delete(WrappedMeta<()>),
}
```

## Optionality

Because of the fallibility of the conversion from external to internal metadata,
it may be a good idea to make metadata optional. This enables two things:
- the external metadata struct can be defaulted to an empty state
- using values with and without metadata is easier

You obviously lose the guarantee that a metadata field will be present, but it
feels like overall the benefit outweigh this downside.

## Serde integration

We now have two representation for metadata; so which one is the best for
deserialization? `serde_toml` promotes the internal representation through
`serde_spanned`, however I'm not a big fan of it as it requires explicit
support by the deserializer and can't be added ina backwards-compatible way.

The main issue is that the `serde_spanned::Spanned` is a bit magic since it
appears during deserialization but does not come from the input. This means
that serializing it is either impossible (because it's skipped) or does not
round-trip (because it ends up in the output). This inconsistency makes it
harder to reify the metadata-aware state of the application.

The external metadata representation feels like a better solution since it
does not break regular deserialization without metadata and provides a clearer
separation of what comes from the data and what comes from the deserializer.
This enables a more consistent deserialize/serialize behavior.

### Serialization

Using the external metadata representation, there is nothing to do for
serialization. You can easily decide between serializing the rich value or
just the simple data part.

### Deserialization

Deserialization is the interesting part: this where the values are created,
unique ids are assigned and metadata (such as source location) is attached.

Let's immediately start with an obvious observation: serde does not support
attaching deserialization metadata currently. This means that we'll have to
do add some custom implementation. Hopefully they remain compatible and can
be added into a potential future version of `serde`.

To generate a new unique id for each value, we need some shared counter that
is incremented as the parser deserializes the input. There are two solution
for such shared state. We can attach it to the `Deserializer` or use
stateful deserialization with [serde::de::DeserializeSeed](https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html).

The deserializer does not know how struct fields are assembled into a struct
by the `Deserialize::deserialize` implementation. This means that it would
require some heuristics to build the matching `NestedMeta`. This means that
the most promising approach is to use `DeserializeSeed`.

`DeserializeSeed` is a generalization of `Deserialize`, there's actually a
blanket impl `DeserializeSeed` for `PhantomData<T> where T: Deserialize`. This
is promising since it means that we could derive a separate impl that is
metadata aware and add it as a feature without breaking compat.

All we need is:

```rust
/// struct holding the state for metadata support
struct MetaScope {
  next_id: usize,
}

pub struct SerdeMetaSeed<'de, 'scope, T> {
  meta_scope: &'scope mut MetaScope,
  phantom: PhantomData<T>  
}

impl <'de, T> DeserializeSeed<'de> for SerdeMetaSeed<'de, T>
// where T: ???
{
  type Value = T;
  
  // ...
}
```

### Regular struct deserialization

Before we look into metadata support for deserialization, let's see how regular
deserialization is derived.

```rust
#[derive(Deserialize)]
struct Mascot {
  is_crab: bool,
  price: u32,
}
```

This expands to something like the following code. I simplified the actual
output.

```rust
struct Mascot {
    is_crab: bool,
    price: u32,
}

const _: () = {
    impl<'de> Deserialize<'de> for Mascot {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            enum Field {
                IsCrab,
                Price,
                Other,
            }
            struct FieldVisitor;
            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = Field;
                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    Formatter::write_str(formatter, "field identifier")
                }
                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match value {
                        "is_crab" => Ok(Field::IsCrab),
                        "price" => Ok(Field::Price),
                        _ => Ok(Field::Other),
                    }
                }
            }
            impl<'de> Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    serde::Deserializer::deserialize_identifier(
                        deserializer,
                        FieldVisitor,
                    )
                }
            }

            struct Visitor<'de> {
                marker: PhantomData<Mascot>,
                lifetime: PhantomData<&'de ()>,
            }
            impl<'de> Visitor<'de> for Visitor<'de> {
                type Value = Mascot;
                fn expecting(
                    &self,
                    formatter: &mut Formatter,
                ) -> fmt::Result {
                    Formatter::write_str(formatter, "struct Mascot")
                }
                #[inline]
                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    let mut is_crab: Option<bool> = None;
                    let mut price: Option<u32> = None;
                    while let Some(key) = map.next_key::<Field>()? {
                        match key {
                            Filed::IsCrab => {
                                if is_crab.is_some() {
                                    return Err(A::Error::duplicate_field("is_crab"));
                                }
                                is_crab = Some(map.next_value()?);
                            }
                            Filed::Price => {
                                if price.is_some() {
                                    return Err(A::Error::duplicate_field("price"));
                                }
                                price = Some(map.next_value()?);
                            }
                            _ => {
                              let _ = map.next_value::<IgnoredAny>()?;
                            }
                        }
                    }
                    let is_crab = match is_crab {
                        Some(is_crab) => is_crab,
                        None => D::Error::missing_field("is_crab")?,
                    };
                    let price = match price {
                        Some(is_crab) => price,
                        None => D::Error::missing_field("price")?,
                    };
                    Ok(Mascot {
                        is_crab,
                        price,
                    })
                }
            }
            const FIELDS: &'static [&'static str] = deserializer.deserialize_struct(
                "Mascot",
                &["is_crab", "price"],
                Visitor {
                    marker: PhantomData::<Mascot>,
                    lifetime: PhantomData,
                },
            )
        }
    }
};
```

### Metadata-aware struct deserialization

---

## Minimal example

The rest of this document will be based around the following example config:

```json
{
  "str": "Hello, World!",
  "num": 42,
  "nested": {
    "crab": true
  }
}
```

This config corresponds to the following simple Rust struct:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
  str: String,
  num: u32,
  nested: Nested,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Nested {
  crab: bool,
}
```

We'll represent the source location as simply the 1-indexed line number of the
start of the value (as `usize`), the real implementation will provide more
advanced information. The point of this document is to see how to attach
metadata in the first place.

The ultimate goal is to load the config and retrieve the source locations:

```rust
let config_text = std::fs::read_to_string("config.json").expect("reading the config succeed");
let config: ??? = parse_with_source_location(&config_text).expect("parsing succeeds");

let num_value: u32 = ???;
let num_source: usize = ???;
let nested_source: usize = ???;

assert_eq!(num_value, 42);
assert_eq!(num_source, 3);
assert_eq!(nested_source, 4);
```

The `Rich` type is used to represent a value with metadata. Its exact shape
depends on context, but you should think of it as:

```
struct Rich<Ty, Meta> {
  value: Ty,
  meta: Meta,
}
```

## Design axis

### Serde compatibility

Is it possible to reuse existing Rust libraries for `serde` without
modifications? In particular, we should distinguish format libraries providing
deserializer implementations and type libraries providing `Deserialize` impls.

A better compatibility with existing libraries is better.

### Serialization transparency

Once a value is parsed, it should be obviously clear if serializing it back
will emit the original input or if it will emit the data decorated with source
information.

### Internal/External metadata

Internal metadata corresponds to a design where the config struct explicitly
contains rich values:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
  str: Rich<String, usize>,
  num: Rich<u32, usize>,
  nested: Rich<Nested, usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Nested {
  crab: Rich<bool, usize>,
}

let config: Rich<Config, usize> = parse_with_source_location(&config_text).expect("parsing succeeds");
```

Internal metadata means that the metadata is inlined withing the data struct.
This makes it trivial to link data with metadata, and keep it synchronized in
presence of mutations or merges. You can specify the exact fields for which you
care about metadata and ignore others. The downside is that it requires updating
all structs and makes it fairly viral. It creates a high coupling between the
pure data and the config.

In contrast, external metadata uses a side-struct to store metadata:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
  str: String,
  num: u32,
  nested: Nested,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Nested {
  crab: bool,
}

struct ConfigMeta {
  r#self: usize,
  str: usize,
  num: usize,
  nested: NestedMeta,
}

struct Nested {
  r#self: usize,
  crab: usize,
}

let config: Rich<Config, Nested> = parse_with_source_location(&config_text).expect("parsing succeeds");
```

The benefit is that you  get a strong separation between the data and metadata.
The coupling is low and the rest of the program can use the regular data.
The downside is that it's harder to match values, especially when collections
such as maps and vecs are involved. There is also an important downside that
it requires the definition of a side-struct for metadata.

External metadata seems like a more promising approach as in supports weaker
coupling. It also provides a clear answer to serialization transparency.
When serializing the value, there are not metadata fields emitted.

## Prior art

### `serde_spanned`

[The `serde_spanned` crate] adds support for inline metadata. The serialization
transparency is not good. If a format is aware of this type (e.g. `toml`) then
it will silently strip the metadata when serializing back; while an unaware
serializer (e.g. `serde_json`) will emit it.

Internally, this is implemented by emitting record calls with special keys
that are recognized by supported parsers.

### `pin-project`

[The `pin-project` crate](https://crates.io/crates/pin-project) (and
[its sister crate `pin-project-lite`](https://crates.io/crates/pin-project-lite))
provide an example of field projection.

### `core::cell::Ref`

[The `core::cell::Ref` type](https://doc.rust-lang.org/core/cell/struct.Ref.html)
is an example from the standard library providing an example of field projection
through [its `map_split` method](https://doc.rust-lang.org/core/cell/struct.Ref.html#method.map_split). 

## Target

The final API should probably look something like:

```
let input = "...";

let output: SerdeRich<Config> = serde_json::deserialize_meta(input).unwrap();

let value: Rich<&Config, &LocationMeta> = output.inject_meta();
println!("str = {}", value.str);
let rich_value: RichConfig<&LocationMeta> = value.inject_meta();
```

