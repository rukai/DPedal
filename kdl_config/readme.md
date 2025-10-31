# kdl-config

Serde style (not serde compatible) derive macros for deserializing a subset of KDL into rust structs and enums.
It is focused around the use case of application configuration.
It is assumed that your config flow will look something like:

1. Use the kdl crate as a first layer of deserialization
2. Apply any version upgrade logic on the KDL document
3. Use kdl-config to parse the document into rust structures.
4. Apply validation on top of the rust structures
5. Convert the config into a structure usable by your application.

Because of this assumption of step 2, we take the upstream KDL document instead of a raw string.
Because of this assumption of step 4, the format of the rust `struct`s/`enum`s must include `Parsed<T>` on every field to include span information, allowing for rich diagnostics.

For example:

```rust
#[derive(KdlConfig, Default, Debug)]
pub struct ProfileParsed {
    pub activation_combination: Parsed<Vec<Parsed<PhysicalButtonParsed>>>,
    pub logic: Parsed<BaseLogicParsed>,
    pub socd: Parsed<SocdTypeParsed>,
}

// Parse the KDL using the upstream kdl crate.
let input = NamedSource::new("foo.kdl", "...");
let kdl: KdlDocument = input.inner().parse()?;

// parse the KDL document into structs and enums
let (profile, error): (Parsed<ConfigParsed>, ParseError) = kdl_config::parse(input, kdl);
// Result is purposely not returned to allow validation to continue on in the case of partial failure

// insert any extra validation into error.diagnostics here.

if !error.diagnostics.is_empty() {
    return Err(error.into());
}
```

## Why KDL?

KDL is much closer to XML than JSON.
However this project restricts itself to a subset of KDL that is semantically much closer to JSON.
This results in an API that maps a bit more naturally to rust and is more intuitive to those unfamiliar with the full extent of KDL.
This comes at the cost of missing out on some of KDL's cool features like arguments and properties. If this is important to you consider using [knus](https://github.com/TheLostLambda/knus) or the [upstream KDL project](https://github.com/kdl-org/kdl-rs).

This then raises the question:

* Why not just use JSON?
  * JSON doesnt allow comments
  * JSON is a little verbose to write with all the string quotes.
* Why not just use YAML?
  * YAML is [full of footguns](https://ruudvanasseldonk.com/2023/01/11/the-yaml-document-from-hell)

## Goals

* Robust deserialization into user defined structs and enums.
  * missing and extra fields will always fail the deserialization.
* Make it trivial to report issues on spans post deserialization.

## Non-goals

* Serialization
* Support for arguments/properties
  * properties will never be supported
  * leaf level arguments deserializing into a list might happen one day after analysis and discussion
* highest performance
  * The use case is configuration, so the expectation is kdl-config is run very rarely.
  * While the project should be fast within its own design, the design choices taken do limit that somewhat.
