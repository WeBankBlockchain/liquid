use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Noop {
        foo: storage::Value<bytes>,
    }

    #[liquid(event)]
    struct Nothing {
        _1: bytes,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.foo.initialize(Default::default());
        }

        pub fn noop_1(&self) -> bytes {
            self.env().emit(Nothing {
                _1: Default::default(),
            });
            bytes::new()
        }

        pub fn noop_2(&self, _b: bytes) {}
    }
}

fn main() {}
