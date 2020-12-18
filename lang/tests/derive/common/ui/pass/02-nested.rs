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

fn main() {}
