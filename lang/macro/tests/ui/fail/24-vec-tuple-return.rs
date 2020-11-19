use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_0(&self) -> (bool, i32) {}

        pub fn noop_1(&self) -> Vec<(bool, i32)> {}
    }
}

fn main() {}
