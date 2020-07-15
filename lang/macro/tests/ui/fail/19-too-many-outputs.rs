use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(
            &self,
        ) -> (
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
            u8,
        ) {
        }
    }
}

fn main() {}
