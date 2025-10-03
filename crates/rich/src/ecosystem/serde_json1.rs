use crate::{Meta, MetaId, StructuralProjection, WrappedMeta};

pub mod value {
  use super::*;
  use crate::{BoolView, Rich};
  use std::borrow::Borrow;
  use std::collections::BTreeMap;
  use std::hash::Hash;

  /// Metadata for [serde_json::Value](serde_json1::value::Value).
  #[derive(Debug)]
  pub enum ValueMeta {
    /// Metadata for [serde_json::Value::Null](serde_json1::value::Value::Null).
    Null(WrappedMeta<Meta<()>>),
    /// Metadata for [serde_json::Value::Bool](serde_json1::value::Value::Bool).
    Bool(WrappedMeta<Meta<bool>>),
    /// Metadata for [serde_json::Value::Number](serde_json1::value::Value::Number).
    Number(MetaId),
    /// Metadata for [serde_json::Value::String](serde_json1::value::Value::String).
    String(WrappedMeta<Meta<String>>),
    /// Metadata for [serde_json::Value::Array](serde_json1::value::Value::Array).
    Array(Vec<WrappedMeta<Meta<::serde_json1::value::Value>>>),
    /// Metadata for [serde_json::Value::Object](serde_json1::value::Value::Object).
    Object(BTreeMap<String, WrappedMeta<Meta<::serde_json1::value::Value>>>),
  }

  impl StructuralProjection for ::serde_json1::value::Value {
    type Meta = Option<ValueMeta>;
  }

  pub struct ValueView<'rich>(Rich<&'rich ::serde_json1::value::Value, &'rich WrappedMeta<Option<ValueMeta>>>);

  impl<'rich> ValueView<'rich> {
    pub fn new(rich: Rich<&'rich ::serde_json1::value::Value, &'rich WrappedMeta<Option<ValueMeta>>>) -> Self {
      Self(rich)
    }

    pub fn value(&self) -> &'rich ::serde_json1::Value {
      &self.0.value
    }

    pub fn meta(&self) -> MetaId {
      self.0.meta.id
    }

    pub fn visit(&self) -> ValueVisit<'_> {
      let rich = &self.0;
      match &rich.value {
        ::serde_json1::value::Value::Bool(value) => {
          const DEFAULT: &WrappedMeta<Meta<bool>> = &WrappedMeta::new((), MetaId::from_usize(0));
          let meta = match rich.meta.nested.as_ref() {
            Some(ValueMeta::Bool(meta)) => meta,
            _ => DEFAULT,
          };
          ValueVisit::Bool(BoolView::new(Rich::new(value, meta)))
        }
        ::serde_json1::value::Value::Array(value) => {
          const DEFAULT: &Vec<WrappedMeta<Meta<serde_json1::Value>>> = &Vec::new();
          let meta = match rich.meta.nested.as_ref() {
            Some(ValueMeta::Array(meta)) => meta,
            _ => DEFAULT,
          };
          ValueVisit::Array(ArrayView::new(Rich::new(value, meta)))
        }
        ::serde_json1::value::Value::Object(value) => {
          const DEFAULT: &BTreeMap<String, WrappedMeta<Meta<::serde_json1::value::Value>>> = &BTreeMap::new();
          let meta = match rich.meta.nested.as_ref() {
            Some(ValueMeta::Object(meta)) => meta,
            _ => DEFAULT,
          };
          ValueVisit::Object(ObjectView::new(Rich::new(value, meta)))
        }
        _ => todo!(),
      }
    }
  }

  pub enum ValueVisit<'rich> {
    Bool(BoolView<'rich>),
    Array(ArrayView<'rich>),
    Object(ObjectView<'rich>),
  }

  #[derive(Debug, Clone, Copy)]
  pub struct ArrayView<'rich>(
    Rich<&'rich Vec<::serde_json1::value::Value>, &'rich Vec<WrappedMeta<Meta<::serde_json1::value::Value>>>>,
  );

  impl<'rich> ArrayView<'rich> {
    pub fn new(
      rich: Rich<&'rich Vec<::serde_json1::value::Value>, &'rich Vec<WrappedMeta<Meta<::serde_json1::value::Value>>>>,
    ) -> Self {
      Self(rich)
    }

    pub fn get(&self, index: usize) -> Option<ValueView<'rich>> {
      const DEFAULT: &WrappedMeta<Option<ValueMeta>> = &WrappedMeta::new(None, MetaId::from_usize(0));
      let rich = &self.0;
      let value = rich.value.get(index)?;
      let meta = match rich.meta.get(index) {
        Some(meta) => meta,
        None => DEFAULT,
      };
      Some(ValueView::new(Rich::new(value, meta)))
    }
  }

  #[derive(Debug, Clone, Copy)]
  pub struct ObjectView<'rich>(
    Rich<
      &'rich ::serde_json1::value::Map<String, ::serde_json1::value::Value>,
      &'rich BTreeMap<String, WrappedMeta<Meta<::serde_json1::value::Value>>>,
    >,
  );

  impl<'rich> ObjectView<'rich> {
    pub fn new(
      rich: Rich<
        &'rich ::serde_json1::value::Map<String, ::serde_json1::value::Value>,
        &'rich BTreeMap<String, WrappedMeta<Meta<::serde_json1::value::Value>>>,
      >,
    ) -> Self {
      Self(rich)
    }

    pub fn value(&self) -> &::serde_json1::value::Map<String, ::serde_json1::value::Value> {
      self.0.value
    }

    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<ValueView<'rich>>
    where
      String: Borrow<Q>,
      Q: ?Sized + Ord + Eq + Hash,
    {
      const DEFAULT: &WrappedMeta<Option<ValueMeta>> = &WrappedMeta::new(None, MetaId::from_usize(0));
      let rich = &self.0;
      let value = rich.value.get(key)?;
      let meta = match rich.meta.get(key) {
        Some(meta) => meta,
        None => DEFAULT,
      };
      Some(ValueView::new(Rich::new(value, meta)))
    }
  }
}

pub use value::ValueMeta;
