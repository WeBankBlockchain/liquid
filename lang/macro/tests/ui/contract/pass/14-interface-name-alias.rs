use liquid_lang as liquid;

#[liquid::interface(name = "Bar")]
mod foo {
    extern "liquid" {
        fn foo(&self);
    }
}

#[liquid::contract(version = "0.2.0")]
mod noop {
    use super::foo::*;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {
            let _bar = Bar::at(Default::default());
        }
    }
}

fn main() {}