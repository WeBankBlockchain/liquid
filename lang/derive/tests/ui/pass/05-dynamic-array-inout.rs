use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
pub struct MyStruct {
    b: bool,
    i: i32,
}

#[liquid::contract(version = "0.2.0")]
mod noop {
    use super::MyStruct;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self, _s: Vec<MyStruct>) -> Vec<MyStruct> {
            Vec::new()
        }
    }
}

fn main() {}
