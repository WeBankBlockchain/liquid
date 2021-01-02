use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    #[liquid(contract)]
    pub struct Noop {
        #[liquid(signers)]
        addr: u8,
    }
}

fn main() {}
