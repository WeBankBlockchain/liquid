use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        pub fn bar() -> u32;
    }
}

fn main() {}
