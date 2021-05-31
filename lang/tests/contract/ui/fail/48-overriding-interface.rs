use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        fn foo(&self);
        #[liquid(mock_context_getter = "another_getter")]
        fn foo(&self, i: i8);
    }
}

fn main() {}