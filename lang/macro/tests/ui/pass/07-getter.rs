use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    use liquid_core::storage;

    #[liquid(storage)]
    struct Noop {
        pub a: storage::Value<bool>,
        pub b: storage::Vec<bool>,
        pub c: storage::Mapping<String, bool>,
        pub d: storage::IterableMapping<String, bool>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop_0(&self) -> bool {
            #[allow(deprecated)]
            self.a(())
        }

        pub fn noop_1(&self) -> bool {
            #[allow(deprecated)]
            self.b(0)
        }

        pub fn noop_2(&self) -> bool {
            #[allow(deprecated)]
            self.c(String::from(""))
        }

        pub fn noop_3(&self) -> bool {
            #[allow(deprecated)]
            self.d(String::from(""))
        }
    }
}

fn main() {}
