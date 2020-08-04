use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    type MyInt = i32;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self, _i: MyInt) {}
    }
}

fn main() {}
