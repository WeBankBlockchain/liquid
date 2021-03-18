use liquid_lang as liquid;

#[liquid::contract(version = "1.0.0-rc1", invalid_key = "whatever")]
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
