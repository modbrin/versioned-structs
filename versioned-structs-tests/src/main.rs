use versioned_structs::versioned;

fn main() {
    let _vd = VersionsDemo {
        v1: HelloThereV1 {
            field_a: false,
            field_b: 42,
        },
        v2: HelloThereV2 {
            field_a: true,
            field_b: 1337,
            field_c: "test".to_string(),
        },
        v3: HelloThere {
            field_a: false,
            field_b: 0,
            field_c: 1,
        },
    };
}

#[versioned]
#[derive(Debug)]
struct HelloThere {
    field_a: bool,
    #[versioned_field(from = 1, to = 3)]
    field_b: u32,
    #[versioned_field(from = 2, to = 2)]
    field_c: String,
    #[versioned_field(from = 3)]
    field_c: i32,
}

struct VersionsDemo {
    v1: HelloThereV1,
    v2: HelloThereV2,
    v3: HelloThere,
}
