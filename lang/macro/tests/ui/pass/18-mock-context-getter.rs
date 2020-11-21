use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        #[liquid(mock_context_getter = "another_getter")]
        fn foo(&self);
    }
}

fn main() {}