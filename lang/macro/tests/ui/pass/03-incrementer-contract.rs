use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod incrementer {
    use liquid_storage as storage;

    #[liquid(storage)]
    struct Incrementer {
        value: storage::Value<u128>,
    }

    impl Incrementer {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.value.set(0);
        }

        #[liquid(external)]
        fn inc_by(&mut self, delta: u128) {
            self.value += delta;
        }

        #[liquid(external)]
        fn get(&self) -> u128 {
            *self.value
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn init_works() {
            let contract = Incrementer::init();
            assert_eq!(contract.get(), 0);
        }

        #[test]
        fn inc_by_works() {
            let mut contract = Incrementer::init();
            contract.inc_by(42);
            assert_eq!(contract.get(), 42);
            contract.inc_by(42);
            assert_eq!(contract.get(), 84);
        }
    }
}

fn main() {}
