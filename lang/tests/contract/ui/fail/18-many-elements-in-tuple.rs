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
            _a: (
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
                u8,
                u8,
            ),
        ) {
        }
    }
}

fn main() {}
