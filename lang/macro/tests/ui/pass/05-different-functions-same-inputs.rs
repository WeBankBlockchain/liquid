use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop_0(&self, _i: i32, _s: String, _b: bool) -> bool {
            false
        }

        #[liquid(external)]
        fn noop_1(&self, _i: i32, _s: String, _b: bool) -> i32 {
            0i32
        }
    }
}

fn main() {}
