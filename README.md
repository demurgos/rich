# Rich value wrapper for Rust

The `Rich` wrapper type represents a value with some attached metadata. This is
a fairly simple type that could be defined locally in your project. The goal
of this library is to provide various helper methods to make it more convenient
to manipulate such values.

An important use-case for this library is to track metadata for config values,
such as the source where a value is coming from. The `serde_rich` crate provides
integration with `serde` to collect deserialization information.
