use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {
        foo: storage::Value<u256>,
        bar: storage::Value<i256>,
    }

    #[liquid(event)]
    struct Nothing {
        _1: u256,
        _2: i256,
        #[liquid(indexed)]
        _3: u256,
        #[liquid(indexed)]
        _4: i256,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.foo.initialize(0.into());
            self.bar.initialize(0.into());
        }

        pub fn noop(&self) -> (u256, i256) {
            self.env().emit(Nothing {
                _1: 0.into(),
                _2: 0.into(),
                _3: 0.into(),
                _4: 0.into(),
            });
            (0u8.into(), 0i8.into())
        }
    }
}

fn main() {}
