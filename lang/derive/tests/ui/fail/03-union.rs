use liquid_lang as liquid;

use liquid::InOut;
#[derive(InOut)]
#[repr(C)]
union MyUnion {
    f1: u32,
    f2: f32,
}

fn main() {}
