use rich::{Meta, MetaId, MetaNode, MetaType, Rich, SplitMeta};
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::marker::PhantomData;
use std::num::NonZeroUsize;


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
    Self {
      next_id: usize::MIN,
    }
  }

  pub fn attach<T>(&mut self, value: T) -> Rich<T, MetaId> {
    let id = self.next_id;
    self.next_id = self.next_id.saturating_add(1);
    Rich::new(value, MetaId(id))
  }
}

#[derive(Debug, Clone, Deserialize)]
struct Nested {
  crab: bool,
}

#[derive(Debug)]
struct RichNested {
  crab: Rich<bool, MetaId>,
}

impl MetaType for Nested {
  type Meta = MetaNested;
}

#[derive(Debug)]
struct MetaNested {
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

#[derive(Debug, Clone)]
struct Config {
  str: String,
  num: u32,
  nested: Nested,
}

impl MetaType for Config {
  type Meta = MetaConfig;
}

#[derive(Debug)]
struct MetaConfig {
  str: MetaNode<()>,
  num: MetaNode<()>,
  nested: MetaNode<MetaNested>,
}

#[derive(Debug)]
struct RichConfig {
  str: Rich<String, MetaId>,
  num: Rich<u32, MetaId>,
  nested: Rich<RichNested, MetaId>,
}

impl SplitMeta for RichConfig {
  type Value = Config;

  fn split_meta(self) -> Rich<Self::Value, Meta<Self::Value>> {
    let str = self.str.deep_split_meta();
    let num = self.num.deep_split_meta();
    let nested = self.nested.deep_split_meta();
    Rich::new(
      Config { str: str.value, num: num.value, nested: nested.value },
      MetaConfig {
        str: str.meta,
        num: num.meta,
        nested: nested.meta,
      },
    )
  }
}

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
        let mut index: u64 = 0;

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
          index += 1;
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

impl<'de, 'scope> DeserializeSeed<'de> for RichScopeSerdeSeed<'scope, RichConfig> {
  type Value = Rich<RichConfig, MetaId>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Debug)]
    enum Field {
      Str,
      Num,
      Nested,
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
          "str" => Ok(Field::Str),
          "num" => Ok(Field::Num),
          "nested" => Ok(Field::Nested),
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
      type Value = RichConfig;

      fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("struct RichNested")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: MapAccess<'de>,
      {
        let mut index: u64 = 0;

        let mut str: Option<Rich<String, MetaId>> = None;
        let mut num: Option<Rich<u32, MetaId>> = None;
        let mut nested: Option<Rich<RichNested, MetaId>> = None;

        while let Some(key) = map.next_key::<Field>()? {
          match key {
            Field::Str => {
              if str.is_some() {
                return Err(A::Error::duplicate_field("str"));
              }
              str = Some(self.0.attach(map.next_value()?))
            }
            Field::Num => {
              if num.is_some() {
                return Err(A::Error::duplicate_field("num"));
              }
              num = Some(self.0.attach(map.next_value()?))
            }
            Field::Nested => {
              if nested.is_some() {
                return Err(A::Error::duplicate_field("nested"));
              }
              let value = map.next_value_seed(RichScopeSerdeSeed::<RichNested>::new(self.0))?;
              nested = Some(value)
            }
            _ => {}
          }
          index += 1;
        }

        let str = match str {
          Some(value) => value,
          None => return Err(A::Error::missing_field("str")),
        };
        let num = match num {
          Some(value) => value,
          None => return Err(A::Error::missing_field("num")),
        };
        let nested = match nested {
          Some(value) => value,
          None => return Err(A::Error::missing_field("nested")),
        };

        Ok(RichConfig { str, num, nested })
      }
    }

    deserializer
      .deserialize_struct("Config", &["str", "num", "nested"], RichVisitor(self.scope))
      .map(|v| self.scope.attach(v))
  }
}

pub fn add(left: u64, right: u64) -> u64 {
  left + right
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

#[derive(Debug)]
pub struct MetaHandle(usize);

struct ConfigMeta<M> {
  this: M,
  fields: (M, M, M),
  values: (M, M, NestedMeta<M>),
}

struct NestedMeta<M> {
  this: M,
  fields: (M,),
  values: (M,),
}

#[derive(Debug)]
struct RichConfigArena<'arena>(&'arena mut MetaArena);

impl<'de, 'arena> DeserializeSeed<'de> for RichConfigArena<'arena> {
  type Value = Config;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Debug)]
    enum Field {
      Str,
      Num,
      Nested,
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
          "str" => Ok(Field::Str),
          "num" => Ok(Field::Num),
          "nested" => Ok(Field::Nested),
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

    struct ConfigVisitor<'arena>(&'arena mut MetaArena);

    impl<'de, 'arena> Visitor<'de> for ConfigVisitor<'arena> {
      type Value = Config;

      fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("struct Config")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: MapAccess<'de>,
      {
        let mut index: u64 = 0;

        self.0.positions.push(index);

        let mut str: Option<(u64, String)> = None;
        let mut num: Option<(u64, u32)> = None;
        let mut nested: Option<(u64, Nested)> = None;

        while let Some(key) = map.next_key::<Field>()? {
          match key {
            Field::Str => {
              if str.is_some() {
                return Err(A::Error::duplicate_field("str"));
              }
              str = Some((index, map.next_value()?))
            }
            Field::Num => {
              if num.is_some() {
                return Err(A::Error::duplicate_field("num"));
              }
              num = Some((index, map.next_value()?))
            }
            Field::Nested => {
              if nested.is_some() {
                return Err(A::Error::duplicate_field("nested"));
              }
              nested = Some((index, map.next_value()?))
            }
            _ => {}
          }
          index += 1;
        }

        let (index_str, str) = match str {
          Some(str) => str,
          None => return Err(A::Error::missing_field("str")),
        };
        let (index_num, num) = match num {
          Some(num) => num,
          None => return Err(A::Error::missing_field("num")),
        };
        let (index_nested, nested) = match nested {
          Some(nested) => nested,
          None => return Err(A::Error::missing_field("nested")),
        };

        self.0.positions.push(index_str);
        self.0.positions.push(index_num);
        self.0.positions.push(index_nested);

        Ok(Config { str, num, nested })
      }
    }

    deserializer.deserialize_struct("Config", &["str", "num", "nested"], ConfigVisitor(self.0))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rich::{Meta, MetaNode, Rich};

  use rich_derive::MetaType;

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, MetaType)]
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

    let mut de = serde_json::de::Deserializer::from_str(input);
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

    let mut de = serde_json::de::Deserializer::from_str(input);
    let rich: Rich<RichConfig, MetaId> = seed.deserialize(&mut de).unwrap();

    dbg!(&rich);

    assert_eq!(rich.value.nested.value.crab.value, true);

    let rich: Rich<Config, MetaNode<Meta<Config>>> = rich.deep_split_meta();

    // dbg!(&config);
    // dbg!(&meta);
    //
    // let merged = config.merge_meta(meta.value);
    // dbg!(&merged);

    // assert_eq!(config.nested.crab, true);
  }
}
