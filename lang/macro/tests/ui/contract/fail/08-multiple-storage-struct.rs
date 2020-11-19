use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    #[liquid(storage)]
    struct Noop_1 {}

    #[liquid(storage)]
    struct Noop_2 {}

    #[liquid(methods)]
    impl Noop_1 {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }

    #[liquid(methods)]
    impl Noop_2 {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
