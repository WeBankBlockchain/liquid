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

        pub fn noop_0(&self) -> MyStruct {
            MyStruct { b: true, i: 0 }
        }

        pub fn noop_1(&self) -> Vec<MyStruct> {
            let mut ret = Vec::new();
            ret.push(MyStruct { b: true, i: 0 });
            ret
        }

        pub fn noop_2(&self) -> (MyStruct, bool) {
            (MyStruct { b: true, i: 0 }, true)
        }
    }
}

fn main() {}
