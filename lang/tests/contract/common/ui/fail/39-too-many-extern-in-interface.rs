use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        fn bar(&self);
    }

    extern "liquid" {
        fn baz(&self);
    }
}

fn main() {}
