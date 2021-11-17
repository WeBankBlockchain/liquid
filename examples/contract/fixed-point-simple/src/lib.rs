#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod fixed_point_simple {
    use super::*;

    #[liquid(storage)]
    struct FixedPoint {
        name: storage::Value<FixedPointU64F16>,
    }

    #[liquid(methods)]
    impl FixedPoint {
        pub fn new(&mut self) {
            self.name.initialize(FixedPointU64F16::from_user("16.61"));
        }

        pub fn get(&self) -> FixedPointU64F16 {
            self.name.clone()
        }

        pub fn set(&mut self, name: FixedPointU64F16) {
            self.name.set(name)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_works() {
            let contract = FixedPoint::new();
            let check = FixedPointU64F16::from_user("16.61");
            assert_eq!(contract.get(), check);
        }

        #[test]
        fn set_works() {
            let mut contract = FixedPoint::new();

            let new_name = FixedPointU64F16::from_user("4402.24");
            contract.set(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }
    }
}
