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
        pub fn constructor(&mut self) {
            self.name.initialize(String::from("Hello, World!"));
        }

        pub fn get(&self) -> String {
            self.name.clone()
        }

        pub fn set(&mut self, name: String) {
            *self.name = name;
        }

        pub fn test_require(&self) {
            require(false, "test");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        #[should_panic]
        fn require_works() {
            let contract = HelloWorld();
            contract.test_require();
        }

        #[test]
        fn get_works() {
            let contract = HelloWorld();
            assert_eq!(contract.get(), "Hello, World!".to_owned());
        }

        #[test]
        fn set_works() {
            let new_name = "Bye, world!".to_owned();
            let mut contract = HelloWorld();
            contract.set(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }
    }
}
