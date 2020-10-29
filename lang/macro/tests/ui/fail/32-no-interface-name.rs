use liquid_lang as liquid;

#[liquid::interface]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

fn main() {}