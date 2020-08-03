#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod sum_2 {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Sum2 {
        values: storage::IterableMapping<String, u32>,
    }

    impl Sum2 {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.values.initialize();
        }

        #[liquid(external)]
        fn insert(&mut self, key: String, val: u32) {
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
            let mut contract = Sum2::init();
            for i in 0..10 {
                contract.insert(i.to_string(), i);
            }
            assert_eq!(contract.sum(), 45);
        }
    }
}
