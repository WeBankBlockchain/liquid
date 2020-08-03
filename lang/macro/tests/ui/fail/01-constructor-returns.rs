use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn constructor(&mut self) -> Self {}

        pub fn noop(&self, i: u8, s: String) -> (bool, i32) {}
    }
}

fn main() {}
