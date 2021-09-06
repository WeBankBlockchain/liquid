use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    #[liquid(contract)]
    pub struct Noop {
        #[liquid(signers)]
        addr: Address,
    }

    #[liquid(rights)]
    impl Noop {
        #[liquid(belongs_to = "addr")]
        pub fn noop_0(&self) {}
    }
}

fn main() {}
