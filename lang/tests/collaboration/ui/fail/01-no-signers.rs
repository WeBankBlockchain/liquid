use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    #[liquid(contract)]
    pub struct Noop {
        addr: Address
    }
}

fn main() {}