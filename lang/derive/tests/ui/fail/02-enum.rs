use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
enum MyStruct {
    U32(u32),
    S(String),
}

fn main() {}
