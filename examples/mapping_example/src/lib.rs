#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod mapping_example {
    use liquid_core::storage;

    #[liquid(storage)]
    struct MappingExample {
        map: storage::Mapping<String, u32>,
    }

    impl MappingExample {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.map.initialize();
        }

        #[liquid(external)]
        fn register(&mut self, name: String, value: u32) {
            self.map.insert(&name, value);
        }

        #[liquid(external)]
        fn query(&self, name: String) -> u32 {
            if let Some(value) = self.map.get(&name) {
                *value
            } else {
                0
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut contract = MappingExample::init();
            contract.register("Alice".to_string(), 42);
            assert_eq!(contract.query("Alice".to_string()), 42);
        }
    }
}
