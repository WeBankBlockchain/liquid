use liquid_lang as liquid;

#[liquid::interface(name = "")]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

fn main() {}