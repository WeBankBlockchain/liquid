use liquid::State;
use liquid_lang as liquid;

#[derive(State)]
pub struct MyState {
    b: bool,
    i: i32,
}

#[derive(State)]
pub struct MyState2 {
    in_out: MyState,
}

fn main() {}
