use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn invalid_return(&mut self) -> Self {}

        #[liquid(message)]
        fn noop(&self, i: u8, s: String) -> (bool, i32) {}
    }
}

fn main() {}
