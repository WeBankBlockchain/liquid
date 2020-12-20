use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    use super::*;

    #[liquid(contract)]
    pub struct Noop {
        addr: address
    }
}

fn main() {}