#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod sum_1 {
    use super::*;

    #[liquid(storage)]
    struct Sum1 {
        value: storage::Vec<u32>,
    }

    #[liquid(methods)]
    impl Sum1 {
        pub fn new(&mut self) {
            self.value.initialize();
        }

        pub fn append(&mut self, elem: u32) {
            self.value.push(elem);
        }

        pub fn sum(&self) -> u32 {
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
            let mut contract = Sum1::new();
            for i in 0..10 {
                contract.append(i);
            }
            assert_eq!(contract.sum(), 45);
        }

        #[test]
        #[should_panic]
        fn upper_overflow() {
            let mut contract = Sum1::new();
            contract.append(u32::MAX);
            contract.append(u32::MAX);
            let _ = contract.sum();
        }
    }
}
