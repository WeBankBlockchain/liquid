use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init_0(&mut self) {}

        #[liquid(constructor)]
        fn init_1(&mut self) {}
    }
}

fn main() {}
