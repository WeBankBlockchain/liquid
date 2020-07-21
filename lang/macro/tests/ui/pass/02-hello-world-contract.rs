use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod hello_world {
    use liquid_core::storage;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    impl HelloWorld {
        #[liquid(constructor)]
        fn new(&mut self) {
            self.name.initialize(String::from("Hello, World!"));
        }

        #[liquid(external)]
        fn get(&self) -> String {
            self.name.clone()
        }

        #[liquid(external)]
        fn set(&mut self, name: String) {
            *self.name = name;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_works() {
            let contract = HelloWorld::new();
            assert_eq!(contract.get(), "Hello, World!".to_owned());
        }

        #[test]
        fn set_works() {
            let new_name = "Bye, world!".to_owned();
            let mut contract = HelloWorld::new();
            contract.set(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }
    }
}

fn main() {}
