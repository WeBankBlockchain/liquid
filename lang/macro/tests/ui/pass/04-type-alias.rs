use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    type MyInt = i32;

    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(&self, _i: MyInt) {}
    }
}

fn main() {}
