use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    type MyInt = i32;
    type MyReturn = (bool, i32);

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_1(&self, _i: MyInt) {}

        pub fn noop_2(&self) -> MyReturn {
            (false, 0i32)
        }
    }
}

fn main() {}
