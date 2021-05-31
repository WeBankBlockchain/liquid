use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod incrementer {
    use super::*;

    #[liquid(storage)]
    struct Incrementer {
        value: storage::Value<u128>,
    }

    #[liquid(methods)]
    impl Incrementer {
        pub fn new(&mut self, init: u128) {
            self.value.initialize(init);
        }

        pub fn inc_by(&mut self, delta: u128) {
            self.value += delta;
        }

        pub fn get(&self) -> u128 {
            *self.value
        }
    }
}

fn main() {}
