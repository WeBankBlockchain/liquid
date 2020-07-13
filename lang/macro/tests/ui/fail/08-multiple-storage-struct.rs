use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop_1 {}

    #[liquid(storage)]
    struct Noop_2 {}

    impl Noop_1 {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(&self) {}
    }

    impl Noop_2 {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(&self) {}
    }
}

fn main() {}
