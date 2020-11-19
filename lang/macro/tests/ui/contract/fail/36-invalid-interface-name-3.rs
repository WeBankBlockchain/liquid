use liquid_lang as liquid;

#[liquid::interface(name = inherited)]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

fn main() {}