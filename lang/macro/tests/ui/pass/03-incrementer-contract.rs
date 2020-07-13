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
            let old_value = *self.value.get();
            self.value.set(old_value + delta);
        }

        #[liquid(external)]
        fn get(&self) -> u128 {
            *self.value.get()
        }
    }
}

fn main() {}
