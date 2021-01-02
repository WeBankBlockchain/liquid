use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    #[liquid(contract)]
    pub struct Noop {
        #[liquid(signers = "$[..][..][1,2,3]@(?true)[..][..]")]
        addr: u8,
    }
}


fn main() {}
