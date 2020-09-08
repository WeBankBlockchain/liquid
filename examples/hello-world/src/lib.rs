#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[macro_use]
extern crate alloc;

#[liquid::contract(version = "0.1.0")]
mod hello_world {
    use liquid_core::storage;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    #[liquid(event)]
    struct NewName {
        old_name: String,
        #[liquid(indexed)]
        new_name: String,
    }

    #[liquid(methods)]
    impl HelloWorld {
        pub fn new(&mut self) {
            self.name.initialize(String::from("Alice"));
        }

        pub fn get(&self) -> String {
            format!("Hello, {}!", self.name.clone())
        }

        pub fn set(&mut self, name: String) {
            let old_name = self.name.clone();
            *self.name = name.clone();
            self.env().emit(NewName {
                old_name,
                new_name: name,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid_core::env::test;

        #[test]
        fn get_works() {
            let contract = HelloWorld::new();
            assert_eq!(contract.get(), String::from("Hello, Alice!"));
        }

        #[test]
        fn set_works() {
            let mut contract = HelloWorld::new();

            let new_name = String::from("Bob");
            contract.set(new_name.clone());
            assert_eq!(contract.get(), format!("Hello, {}!", new_name));

            let events = test::get_events();
            assert_eq!(events.len(), 1);
            let new_name_event = events.last().unwrap();
            assert_eq!(new_name_event.topics.len(), 2);
            assert_eq!(
                new_name_event.topics[0],
                "0x1be2d150ed559c350b05f7dfa5a74669ec8d2ce63bb14c134730ffa02d2d111c"
                    .into()
            );
            assert_eq!(
                new_name_event.topics[1],
                "0x28cac318a86c8a0a6a9156c2dba2c8c2363677ba0514ef616592d81557e679b6"
                    .into()
            );
            assert_eq!(new_name_event.decode_data::<String>(), "Alice");
        }
    }
}
