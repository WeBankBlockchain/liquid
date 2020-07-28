use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
pub struct MyStruct {
    b: bool,
    i: i32,
}

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::MyStruct;

    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {}

        #[liquid(external)]
        fn noop(&self, _s: MyStruct) {}
    }
}

fn main() {}
