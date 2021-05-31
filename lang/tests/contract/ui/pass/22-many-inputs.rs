use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(
            &self,
            _a0: u8,
            _a1: u8,
            _a2: u8,
            _a3: u8,
            _a4: u8,
            _a5: u8,
            _a6: u8,
            _a7: u8,
            _a8: u8,
            _a9: u8,
            _a10: u8,
            _a11: u8,
            _a12: u8,
            _a13: u8,
            _a14: u8,
            _a15: u8,
            _a16: u8,
            _a17: u8,
            _a18: u8,
            _a19: u8,
            _a20: u8,
            _a21: u8,
            _a22: u8,
            _a23: u8,
            _a24: u8,
            _a25: u8,
            _a26: u8,
        ) {
        }
    }
}

fn main() {}
