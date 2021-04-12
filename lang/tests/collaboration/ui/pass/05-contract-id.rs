use liquid_lang as liquid;

#[liquid::collaboration]
mod auction {
    #[liquid(contract)]
    pub struct Noop {
        #[liquid(signers)]
        addr: address,
    }

    #[liquid(rights_belong_to = "addr")]
    impl Noop {
        pub fn noop(&self, _id: ContractId<Noop>) -> ContractId<Noop> {
            unreachable!();
        }
    }
}

fn main() {}
