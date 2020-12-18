use liquid::State;
use liquid_lang as liquid;

#[derive(State)]
pub struct MyStruct<T> {
    a: T,
}

fn main() {}
