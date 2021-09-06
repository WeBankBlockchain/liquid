use liquid_lang as liquid;

#[liquid::contract]
mod noop1 {
    use super::*;

    #[liquid::interface(name = auto)]
    mod foo {
        extern "liquid" {
            fn foo(&self);
        }
    }

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
