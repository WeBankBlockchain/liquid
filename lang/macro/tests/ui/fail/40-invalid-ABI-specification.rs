use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "C" {
        fn bar(&self);
    }
}

fn main() {}
