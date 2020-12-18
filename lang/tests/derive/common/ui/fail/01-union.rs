use liquid::InOut;
use liquid_lang as liquid;

#[derive(InOut)]
#[repr(C)]
pub union MyUnion {
    f1: u32,
    f2: f32,
}

fn main() {}
