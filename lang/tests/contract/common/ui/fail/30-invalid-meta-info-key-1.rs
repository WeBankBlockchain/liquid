use liquid_lang as liquid;

#[liquid::contract(version = "0.3.0", invalid_key = "whatever")]
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
