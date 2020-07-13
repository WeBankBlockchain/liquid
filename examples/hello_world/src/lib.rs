#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod hello_world {
    use liquid_storage as storage;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    impl HelloWorld {
        #[liquid(constructor)]
        fn new(&mut self) {
            self.name.set("Hello, World!".to_owned());
        }

        #[liquid(external)]
        fn get(&self) -> String {
            self.name.get()
        }

        #[liquid(external)]
        fn set(&mut self, String name) -> bool {
            self.name.set(name);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut contract = HelloWorld::new();
            assert_eq!(contract.get(), "Hello, World!".to_owned());
            contract.set("Bye, World!".to_owned());
            assert_eq!(contract.get(), "Bye, World!".to_owned());
        }
    }
}
