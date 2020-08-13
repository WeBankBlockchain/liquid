#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod hello_world {
    use liquid_core::storage;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    #[liquid(methods)]
    impl HelloWorld {
        pub fn new(&mut self, name: String) {
            self.name.initialize(name);
        }

        pub fn get(&self) -> String {
            self.name.clone()
        }

        pub fn set(&mut self, name: String) {
            *self.name = name;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_works() {
            let new_name = "Hello, world!".to_owned();
            let contract = HelloWorld::new(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }

        #[test]
        fn set_works() {
            let old_name = "Hello, world!".to_owned();
            let mut contract = HelloWorld::new(old_name);

            let new_name = "Bye, world!".to_owned();
            contract.set(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }
    }
}
