#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod iterable_mapping_example {
    use liquid_core::storage;

    #[liquid(storage)]
    struct IterableMappingExample {
        values: storage::IterableMapping<String, u32>,
    }

    impl IterableMappingExample {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.values.initialize();
        }

        #[liquid(external)]
        fn register(&mut self, key: String, val: u32) {
            self.values.insert(key, val);
        }

        #[liquid(external)]
        fn sum(&self) -> u32 {
            let mut ret = 0u32;
            for (_, v) in self.values.iter() {
                ret += v;
            }
            ret
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut contract = IterableMappingExample::init();
            for i in 0..10 {
                contract.register(i.to_string(), i);
            }
            assert_eq!(contract.sum(), 45);
        }
    }
}
