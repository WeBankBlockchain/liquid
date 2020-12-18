use liquid_lang as liquid;

#[liquid::interface(name = "fn")]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

fn main() {}