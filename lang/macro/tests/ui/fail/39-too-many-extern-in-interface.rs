use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        fn bar();
    }

    extern "liquid" {
        fn baz();
    }
}

fn main() {}
