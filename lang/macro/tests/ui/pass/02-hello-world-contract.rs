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
        fn init(&mut self) {
            self.name.set("Hello, world!".to_owned());
        }

        #[liquid(external)]
        fn get(&self) -> String {
            self.name.get().to_string()
        }

        #[liquid(external)]
        fn set(&mut self, new_name: String) {
            self.name.set(new_name);
        }
    }
}

fn main() {}
