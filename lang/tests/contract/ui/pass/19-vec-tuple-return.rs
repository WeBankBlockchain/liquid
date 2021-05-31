use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_0(&self) -> (bool, i32) {
            (false, 0)
        }

        pub fn noop_1(&self) -> Vec<(bool, i32)> {
            vec![(false, 0)]
        }
    }
}

fn main() {}
