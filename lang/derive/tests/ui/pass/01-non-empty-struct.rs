use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
pub struct MyInOut {
    b: bool,
    i: i32,
}

use liquid::State;
#[derive(State)]
pub struct MyState {
    b: bool,
    i: i32,
}

fn main() {}
