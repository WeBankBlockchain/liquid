use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod incrementer {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Incrementer {
        value: storage::Value<u128>,
    }

    #[liquid(methods)]
    impl Incrementer {
        pub fn new(&mut self) {
            self.value.initialize(0);
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
