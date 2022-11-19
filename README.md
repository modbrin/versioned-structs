# Versioned Structs

Library which allows to keep older versions of same struct with version suffixes in name.

> Currently at proof-of-concept stage.

Example:
```rust
use versioned_structs::versioned;

#[versioned]
struct HelloThere {
    field_a: bool,
    #[versioned_field(from = 1, to = 3)]
    field_b: u32,
    #[versioned_field(from = 2, to = 2)]
    field_c: String,
    #[versioned_field(from = 3)]
    field_c: i32,
}
```
will expand into:
```rust
struct HelloThereV1 {
    field_a: bool,
    field_b: u32,
}

struct HelloThereV2 {
    field_a: bool,
    field_b: u32,
    field_c: String,
}

struct HelloThere {
    field_a: bool,
    field_b: u32,
    field_c: i32,
}
```