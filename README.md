# derive-from-ext
A derive macro that auto implements 'std::convert::From' for structs. The default behaviour is to create an instance of the structure by calling .into() on each of the source structure properties. The source struct properties can be mapped using different names and methods by using field attributes to override the default behaviour.

## Installing
Include in your Cargo.toml file:
```toml
[dependencies]
derive-from-ext = "0.1"
```

## Examples
```rust
use derive_from_ext::From;

struct A {
    prop1: String,
}

#[derive(From, Debug)]
#[from(A)]
struct B {
    prop1: String,
}

let a = A { prop1: "Test".to_string() };
let b: B = a.into();
dbg!(b); //automatically converted into type B and can use implementations on this type
```

## Defaults
If a source structure has few properties than the current structure then the property can be skipped by tagging with 'skip':
```rust
#[from(A)]
struct B [
    #[from(skip)]
    other_prop: String,
]
```

Note: this only works where a default can be assigned to the property value and is equivalent to setting the skip method to 'std::default::Default::default' as below.

```rust
#[from(A)]
struct B [
    #[from(skip, default="String::from(\"New value\")")]
    other_prop: String,
]
```

## Alternative method
To use an alternative method to create the value for the structure property, a property attribute can be used with the path of the met:
```rust
fn lowercase(str: String) -> String {
    str.to_lowercase()
}

#[from(A)]
struct B {
    #[from(map="lowercase")]
    other_prop: String,
}
```

## Different property names
To map from a different property on the source structure, a property attribute can be used:
```rust
#[from(A)]
struct B {
    #[from(rename="prop1")]
    other_prop: String,
}
```

## Multiple structs
To support multiple source structures, the 'from' attribute can be extended to include the other structures. The attributes can then also override source-specific options for each source structure:
```rust
#[from(A, B)]
struct C {
    #[from(overrides=( A=(skip=true), B=(map="lowercase") ))]
    other_prop: String,
}
```

## Alternatives
If you do not require the features above and only want to convert a struct into matching struct you may be better off using [derive_more](https://github.com/JelteF/derive_more) instead.
