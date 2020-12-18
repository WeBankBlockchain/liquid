use liquid::{InOut, State};
use liquid_lang as liquid;

#[derive(InOut, State)]
enum MyEnum {
    U32(u32),
    S(String),
}

fn main() {}
