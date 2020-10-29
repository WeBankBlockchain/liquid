use liquid_lang as liquid;
use liquid::InOut;

#[derive(InOut)]
pub struct MyInOut {
    b: bool,
    i: i32,
}

#[derive(InOut)]
pub struct MyInOut2 {
    in_out: MyInOut,
}

fn main() {}
