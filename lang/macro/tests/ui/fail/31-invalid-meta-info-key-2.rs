use liquid_lang as liquid;

#[liquid::interface(name = auto, invalid_key = "whatever")]
mod foo {
    extern "liquid" {
        fn bar();
    }
}

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
