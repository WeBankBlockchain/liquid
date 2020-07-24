#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod vec_example {
    use liquid_core::storage;

    #[liquid(storage)]
    struct VecExample {
        value: storage::Vec<u32>,
    }

    impl VecExample {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.value.initialize();
        }

        #[liquid(external)]
        fn append(&mut self, elem: u32) {
            self.value.push(elem);
        }

        #[liquid(external)]
        fn sum(&self) -> u32 {
            let mut ret = 0u32;
            for elem in self.value.iter() {
                ret += elem;
            }
            ret
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut contract = VecExample::init();
            for i in 0..10 {
                contract.append(i);
            }
            assert_eq!(contract.sum(), 45);
        }

        #[test]
        #[should_panic]
        fn upper_overflow() {
            let mut contract = VecExample::init();
            contract.append(u32::MAX);
            contract.append(u32::MAX);
            let _ = contract.sum();
        }
    }
}
