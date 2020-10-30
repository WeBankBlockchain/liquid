use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    pub struct Baz {
        a: i32,
    }

    extern "liquid" {
        fn bar() -> u32;
    }
}

fn main() {}
