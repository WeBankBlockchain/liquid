use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) -> (bool, i32, i8, String) {
            (true, 0i32, 0i8, String::new())
        }
    }
}

fn main() {}
