use crate::RichDeserialize;
use crate::RichScope;
use crate::RichScopeSerdeSeed;
use rich::{Meta, WrappedMeta};
use rich::{Rich};
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserializer;
use std::collections::BTreeMap;

pub mod value {
  use super::*;

  impl<'de> RichDeserialize<'de> for serde_json1::Value {
    fn rich_deserialize<'scope, D>(
      scope: &'scope mut RichScope,
      deserializer: D,
    ) -> Result<Rich<Self, WrappedMeta<Self::Meta>>, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct RichVisitor<'scope>(&'scope mut RichScope);

      impl<'de, 'scope> Visitor<'de> for RichVisitor<'scope> {
        type Value = Rich<serde_json1::Value, Option<rich::ecosystem::serde_json1::ValueMeta>>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
          formatter.write_str("struct serde_json::Value")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
          E: Error,
        {
          let value0 = v;
          let meta0 = ();
          let rich0 = Rich::new(value0, meta0);
          let wrapped0 = self.0.wrap(rich0);

          let value = serde_json1::Value::Bool(wrapped0.value);
          let meta = Some(rich::ecosystem::serde_json1::ValueMeta::Bool(wrapped0.meta));

          Ok(Rich::new(value, meta))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
          A: MapAccess<'de>,
        {
          let mut value = serde_json1::value::Map::new();
          let mut meta = BTreeMap::<String, WrappedMeta<Meta<::serde_json1::value::Value>>>::new();

          while let Some(key) = map.next_key::<String>()? {
            let rich = map.next_value_seed(RichScopeSerdeSeed::<serde_json1::Value>::new(self.0))?;
            value.insert(key.clone(), rich.value);
            meta.insert(key, rich.meta);
          }

          let value = serde_json1::Value::Object(value);
          let meta = Some(rich::ecosystem::serde_json1::ValueMeta::Object(meta));

          Ok(Rich::new(value, meta))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
          A: SeqAccess<'de>,
        {
          let mut value = Vec::<serde_json1::Value>::new();
          let mut meta = Vec::<WrappedMeta<Meta<::serde_json1::value::Value>>>::new();

          while let Some(rich) = seq.next_element_seed(RichScopeSerdeSeed::<serde_json1::Value>::new(self.0))? {
            value.push(rich.value);
            meta.push(rich.meta);
          }

          let value = serde_json1::Value::Array(value);
          let meta = Some(rich::ecosystem::serde_json1::ValueMeta::Array(meta));

          Ok(Rich::new(value, meta))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
          E: Error,
        {
          let value0 = String::from(v);
          let meta0 = ();
          let rich0 = Rich::new(value0, meta0);
          let wrapped0 = self.0.wrap(rich0);

          let value = serde_json1::Value::String(wrapped0.value);
          let meta = Some(rich::ecosystem::serde_json1::ValueMeta::String(wrapped0.meta));

          Ok(Rich::new(value, meta))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
          E: Error,
        {
          let value0 = v;
          let meta0 = ();
          let rich0 = Rich::new(value0, meta0);
          let wrapped0 = self.0.wrap(rich0);

          let value = serde_json1::Value::String(wrapped0.value);
          let meta = Some(rich::ecosystem::serde_json1::ValueMeta::String(wrapped0.meta));

          Ok(Rich::new(value, meta))
        }
      }

      deserializer.deserialize_any(RichVisitor(scope)).map(|v| scope.wrap(v))
    }
  }
}
