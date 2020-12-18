use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) -> Self {}

        pub fn noop(&self, i: u8, s: String) -> (bool, i32) {}
    }
}

fn main() {}
