use liquid::InOut;
use liquid_lang as liquid;

#[liquid::collaboration]
mod noop {
    use super::*;

    #[derive(InOut)]
    pub struct A {
        a: address,
        b: bool,
    }

    #[derive(InOut)]
    pub struct B {
        a: address,
        b: u8,
        c: bool,
    }

    fn select(bs: &Vec<B>) -> impl IntoIterator<Item = &address> {
        bs.iter().filter(|b| b.c).map(|b| &b.a)
    }

    #[liquid(contract)]
    pub struct Noop {
        #[liquid(signers = "$")]
        _0: address,
        #[liquid(signers = "$[..]")]
        _1: Vec<address>,
        #[liquid(signers = "$[..][..][..][..]")]
        _2: Vec<address>,
        #[liquid(signers = "$[1,2,3]")]
        _3: Vec<address>,
        #[liquid(signers = "$[..](?@.b)[-2..-1].a")]
        _4: Vec<A>,
        #[liquid(signers = "$[..](?@.b > 42)[1,2](?@.b <= 1024 && @.c).a")]
        _5: Vec<B>,
        #[liquid(signers = "$[..](?(false || 1 == 2) || (true && false) && !(1 != 1))")]
        _6: Vec<address>,
        #[liquid(signers = "crate::noop::select")]
        _7: Vec<B>,
    }
}

fn main() {}
