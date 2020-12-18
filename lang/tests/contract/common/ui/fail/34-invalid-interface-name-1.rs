use liquid_lang as liquid;

#[liquid::interface(name = "r#fn")]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

fn main() {}