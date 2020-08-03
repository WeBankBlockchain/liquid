use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn constructor(&mut self) {}

        pub fn noop(
            &self,
            a0: u8,
            a1: u8,
            a2: u8,
            a3: u8,
            a4: u8,
            a5: u8,
            a6: u8,
            a7: u8,
            a8: u8,
            a9: u8,
            a10: u8,
            a11: u8,
            a12: u8,
            a13: u8,
            a14: u8,
            a15: u8,
            a16: u8,
        ) {
        }
    }
}

fn main() {}
