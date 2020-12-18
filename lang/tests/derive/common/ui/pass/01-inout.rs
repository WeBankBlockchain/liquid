use liquid::InOut;
use liquid_lang as liquid;

#[derive(InOut)]
pub struct MyStruct {
    b: bool,
    i: i32,
}

#[derive(InOut)]
pub struct MyStruct2 {
    b: bool,
    i: i32,
}

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_1(&self, _s: MyStruct) -> Vec<MyStruct> {
            vec![]
        }

        pub fn noop_2(&self, _s: Vec<MyStruct>) -> MyStruct {
            MyStruct { b: false, i: 0 }
        }

        pub fn noop_3(&self) -> (MyStruct, MyStruct2) {
            (MyStruct { b: false, i: 0 }, MyStruct2 { b: false, i: 0 })
        }
    }
}

fn main() {}
