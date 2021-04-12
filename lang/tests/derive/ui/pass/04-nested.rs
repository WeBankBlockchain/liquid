use liquid::InOut;
use liquid_lang as liquid;

#[derive(InOut)]
pub struct MyInOut {
    b: bool,
    i: i32,
}

#[derive(InOut)]
pub struct MyInOut2 {
    in_out: MyInOut,
}

#[liquid::contract]
mod noop {
    use super::MyInOut2;

    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self, _s: MyInOut2) -> MyInOut2 {
            unreachable!();
        }
    }
}

fn main() {}
