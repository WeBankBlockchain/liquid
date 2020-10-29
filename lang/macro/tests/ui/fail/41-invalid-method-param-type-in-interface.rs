use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        fn bar(_f: f32);
    }
}

fn main() {}
