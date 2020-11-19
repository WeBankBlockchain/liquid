use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Noop {
        foo: storage::Value<bytes1>,
        bar: storage::Value<bytes2>,
    }

    #[liquid(event)]
    struct Nothing {
        _1: bytes3,
        _2: bytes4,
        #[liquid(indexed)]
        _3: bytes5,
        #[liquid(indexed)]
        _4: bytes6,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.foo.initialize(Default::default());
            self.bar.initialize(Default::default());
        }

        pub fn noop_1(&self) -> (bytes7, bytes8) {
            self.env().emit(Nothing {
                _1: Default::default(),
                _2: Default::default(),
                _3: Default::default(),
                _4: Default::default(),
            });
            (Default::default(), Default::default())
        }

        pub fn noop_2(&self, _b: bytes9) {}
    }
}

fn main() {}
