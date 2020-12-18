use liquid::InOut;
use liquid_lang as liquid;

#[derive(InOut)]
pub enum MyStruct {
    U32(u32),
    S(String),
}

#[liquid::contract]
mod noop {
    use super::MyStruct;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_1(&self, _s: MyStruct) {}

        pub fn noop_2(&self, _s: Vec<MyStruct>) {}

        pub fn noop_3(&self) -> (MyStruct, bool) {
            (MyStruct::U32(0u32), false)
        }
    }
}

fn main() {}
