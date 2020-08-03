use liquid_lang as liquid;

#[derive(Clone, Copy)]
pub struct I(i32);

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn constructor(&mut self) {}

        pub fn noop(&self) -> I {}
    }
}

fn main() {}
