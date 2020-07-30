use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
struct MyInOut {
    b: bool,
    i: i32,
}

use liquid::State;
#[derive(State)]
struct MyState {
    b: bool,
    i: i32,
}

fn main() {}
