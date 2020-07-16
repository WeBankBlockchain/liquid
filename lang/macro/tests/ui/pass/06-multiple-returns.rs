use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(&self) -> (bool, i32, i8, String) {
            (true, 0i32, 0i8, String::new())
        }
    }
}

fn main() {}
