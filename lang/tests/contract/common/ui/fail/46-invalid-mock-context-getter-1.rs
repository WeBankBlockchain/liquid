use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        #[liquid(mock_context_getter = something)]
        fn foo(&self);
    }
}

fn main() {}