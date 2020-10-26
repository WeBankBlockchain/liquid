use liquid_lang as liquid;

#[liquid::interface(name = "Bar")]
mod foo {
    extern "liquid" {
        fn foo();
    }
}

#[liquid::contract(version = "0.1.0")]
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