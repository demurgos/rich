mod ecosystem;

use rich::{Meta, MetaId, MetaNode, StructuralProjection, Rich, SplitMeta, WrappedMeta};
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::marker::PhantomData;

/// Re-export of the public dependency `rich`.
pub use rich;

/// Trait representing types that can richly deserialized (including
/// deserialization metadata).
trait RichDeserialize<'de>
where
  Self: StructuralProjection,
{
  fn rich_deserialize<'scope, D>(
    scope: &'scope mut RichScope,
    deserializer: D,
  ) -> Result<Rich<Self, WrappedMeta<Self::Meta>>, D::Error>
  where
    Self: Sized,
    D: Deserializer<'de>;
}

impl<'de, 'scope, T> DeserializeSeed<'de> for RichScopeSerdeSeed<'scope, T>
where
  T: RichDeserialize<'de>,
{
  type Value = Rich<T, WrappedMeta<T::Meta>>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    T::rich_deserialize(self.scope, deserializer)
  }
}


// impl MergeMeta<()> for u32 {
//   type Rich = Self;
//
//   fn merge_meta(self, _meta: ()) -> Self::Rich {
//     self
//   }
// }
//
// impl SplitMeta for bool {
//   type Value = Self;
//   type Meta = ();
//
//   fn split_meta(self) -> (Self::Value, Self::Meta) {
//     (self, ())
//   }
// }
//
// impl MergeMeta<()> for bool {
//   type Rich = Self;
//
//   fn merge_meta(self, _meta: ()) -> Self::Rich {
//     self
//   }
// }
//
// impl SplitMeta for String {
//   type Value = Self;
//   type Meta = ();
//
//   fn split_meta(self) -> (Self::Value, Self::Meta) {
//     (self, ())
//   }
// }
//
// impl MergeMeta<()> for String {
//   type Rich = Self;
//
//   fn merge_meta(self, _meta: ()) -> Self::Rich {
//     self
//   }
// }

pub struct RichScope {
  next_id: usize,
}

impl RichScope {
  pub fn new() -> Self {
    Self { next_id: usize::MIN }
  }

  pub fn attach<T>(&mut self, value: T) -> Rich<T, MetaId> {
    let id = self.next_id;
    self.next_id = self.next_id.saturating_add(1);
    Rich::new(value, MetaId(id))
  }

  pub fn wrap<Value, Nested>(&mut self, rich: Rich<Value, Nested>) -> Rich<Value, WrappedMeta<Nested>> {
    let id = self.next_id;
    self.next_id = self.next_id.saturating_add(1);
    Rich::new(rich.value, WrappedMeta::new(rich.meta, MetaId(id)))
  }
}

#[derive(Debug, Clone, Deserialize)]
struct Nested {
  #[allow(unused)]
  crab: bool,
}

#[derive(Debug)]
struct RichNested {
  crab: Rich<bool, MetaId>,
}

impl StructuralProjection for Nested {
  type Meta = MetaNested;
}

#[derive(Debug)]
struct MetaNested {
  #[allow(unused)]
  crab: MetaNode<()>,
}

impl SplitMeta for RichNested {
  type Value = Nested;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
    let crab = self.crab.deep_split_meta();
    Rich::new(Nested { crab: crab.value }, MetaNested { crab: crab.meta })
  }
}

// impl SplitMeta for RichNested {
//   type Value = Nested;
//   type Meta = MetaNested;
//
//   fn split_meta(self) -> (Self::Value, Self::Meta) {
//     let (crab, meta_crab) = self.crab.deep_split_meta();
//     (Nested { crab }, MetaNested { crab: meta_crab })
//   }
// }
//
// impl MergeMeta<MetaNested> for Nested {
//   type Rich = RichNested;
//
//   fn merge_meta(self, meta: MetaNested) -> Self::Rich {
//     RichNested {
//       crab: Rich::new(self.crab, meta.crab.meta),
//     }
//   }
// }

// impl SplitMeta for RichConfig {
//   type Value = Config;
//   type Meta = MetaConfig;
//
//   fn split_meta(self) -> (Self::Value, Self::Meta) {
//     let (str, meta_str) = self.str.deep_split_meta();
//     let (num, meta_num) = self.num.deep_split_meta();
//     let (nested, meta_nested) = self.nested.deep_split_meta();
//     (
//       Config { str, num, nested },
//       MetaConfig {
//         str: meta_str,
//         num: meta_num,
//         nested: meta_nested,
//       },
//     )
//   }
// }
//
// impl MergeMeta<MetaConfig> for Config {
//   type Rich = RichConfig;
//
//   fn merge_meta(self, meta: MetaConfig) -> Self::Rich {
//     RichConfig {
//       str: Rich::new(self.str.merge_meta(meta.str.value), meta.str.meta),
//       num: Rich::new(self.num.merge_meta(meta.num.value), meta.num.meta),
//       nested: Rich::new(self.nested.merge_meta(meta.nested.value), meta.nested.meta),
//     }
//   }
// }

pub struct RichScopeSerdeSeed<'scope, T> {
  scope: &'scope mut RichScope,
  phantom: PhantomData<fn() -> T>,
}

impl<'scope, T> RichScopeSerdeSeed<'scope, T> {
  pub fn new(scope: &'scope mut RichScope) -> Self {
    Self {
      scope,
      phantom: PhantomData,
    }
  }
}

impl<'de, 'scope> DeserializeSeed<'de> for RichScopeSerdeSeed<'scope, RichNested> {
  type Value = Rich<RichNested, MetaId>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Debug)]
    enum Field {
      Crab,
      Other,
    }

    struct FieldVisitor;

    impl<'de> Visitor<'de> for FieldVisitor {
      type Value = Field;

      fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("field identifier")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: Error,
      {
        match v {
          "crab" => Ok(Field::Crab),
          _ => Ok(Field::Other),
        }
      }
    }

    impl<'de> Deserialize<'de> for Field {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        deserializer.deserialize_identifier(FieldVisitor)
      }
    }

    struct RichVisitor<'scope>(&'scope mut RichScope);

    impl<'de, 'scope> Visitor<'de> for RichVisitor<'scope> {
      type Value = RichNested;

      fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("struct RichNested")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: MapAccess<'de>,
      {
        let mut crab: Option<Rich<bool, MetaId>> = None;

        while let Some(key) = map.next_key::<Field>()? {
          match key {
            Field::Crab => {
              if crab.is_some() {
                return Err(A::Error::duplicate_field("crab"));
              }
              crab = Some(self.0.attach(map.next_value()?))
            }
            _ => {}
          }
        }

        let crab = match crab {
          Some(crab) => crab,
          None => return Err(A::Error::missing_field("crab")),
        };

        Ok(RichNested { crab })
      }
    }

    deserializer
      .deserialize_struct("Nested", &["crab"], RichVisitor(self.scope))
      .map(|v| self.scope.attach(v))
  }
}

#[derive(Debug)]
pub struct MetaArena {
  pub positions: Vec<u64>,
}

impl MetaArena {
  pub fn new() -> Self {
    Self { positions: Vec::new() }
  }
}

#[allow(unused)]
struct ConfigMeta<M> {
  this: M,
  fields: (M, M, M),
  values: (M, M, NestedMeta<M>),
}

#[allow(unused)]
struct NestedMeta<M> {
  this: M,
  fields: (M,),
  values: (M,),
}

#[cfg(test)]
mod tests {
  use super::*;
  use rich::{Meta, MetaNode, Rich};
  use rich_derive::MetaType;
  use ::serde_json1;
  use rich::ecosystem::serde_json1::value::{ValueView, ValueVisit};

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, MetaType)]
  #[meta(attr(derive(Default, Debug)))]
  struct Opaque;

  #[test]
  fn rich_parse_nested() {
    // language=json
    let input = r#"{
  "crab": true
}"#;
    let mut scope = RichScope::new();
    let seed = RichScopeSerdeSeed::<RichNested> {
      scope: &mut scope,
      phantom: PhantomData,
    };

    let mut de = serde_json1::de::Deserializer::from_str(input);
    let rich: Rich<RichNested, MetaId> = seed.deserialize(&mut de).unwrap();

    dbg!(&rich);

    assert_eq!(rich.value.crab.value, true);

    let result = add(2, 2);
    assert_eq!(result, 4);
  }

  #[test]
  fn rich_parse_config() {
    // language=json
    let input = r#"{
  "num": 42,
  "str": "Hello, World!",
  "nested": {
    "crab": true
  }
}"#;
    let mut scope = RichScope::new();
    let seed = RichScopeSerdeSeed::<RichConfig> {
      scope: &mut scope,
      phantom: PhantomData,
    };

    let mut de = serde_json1::de::Deserializer::from_str(input);
    let rich: Rich<RichConfig, MetaId> = seed.deserialize(&mut de).unwrap();

    dbg!(&rich);

    assert_eq!(rich.value.nested.value.crab.value, true);

    let rich: Rich<Config, MetaNode<Meta<Config>>> = rich.deep_split_meta();

    let foo: Meta<Opaque> = Meta::<Opaque>::default();

    // dbg!(&config);
    // dbg!(&meta);
    //
    // let merged = config.merge_meta(meta.value);
    // dbg!(&merged);

    // assert_eq!(config.nested.crab, true);
  }

  #[test]
  fn rich_parse_serde_json_value() {
    // language=json
    let input = r#"{
  "foo": true,
  "message": "Hello, World!",
  "list": [true, false]
}"#;
    let mut scope = RichScope::new();
    let seed = RichScopeSerdeSeed::<serde_json1::Value> {
      scope: &mut scope,
      phantom: PhantomData,
    };

    let mut de = serde_json1::de::Deserializer::from_str(input);
    let rich: Rich<serde_json1::Value, WrappedMeta<Option<rich::ecosystem::serde_json1::ValueMeta>>> =
      seed.deserialize(&mut de).unwrap();

    dbg!(&rich);

    assert_eq!(
      rich.value,
      serde_json1::Value::Object({
        let mut obj = serde_json1::value::Map::new();
        obj.insert(String::from("foo"), serde_json1::Value::Bool(true));
        obj.insert(String::from("message"), serde_json1::Value::String(String::from("Hello, World!")));
        obj.insert(String::from("list"), serde_json1::Value::Array(vec![serde_json1::Value::Bool(true),serde_json1::Value::Bool(false) ]));
        obj
      })
    );

    let view = ValueView::new(rich.as_ref());
    assert_eq!(view.meta(), MetaId::from_usize(9));

    let ValueVisit::Object(view) = view.visit() else { panic!("expected view visit to return `Object` variant"); };

    let foo: ValueView<'_> = view.get("foo").expect("`foo` view is available");

    dbg!(foo.value());
    assert_eq!(foo.meta(), MetaId::from_usize(1));

    // match view.visit() {
    //   ValueVisit::Bool(view) => {
    //     dbg!(view);
    //   }
    //   ValueVisit::Array(view) => {
    //     dbg!(view);
    //   }
    //   ValueVisit::Object(view) => {
    //     dbg!(view);
    //   }
    // }

    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
