use liquid::storage;
use liquid_lang as liquid;

/// This trick is just used for testing. **DO NOT** use it in your own code.
type GetterIndexPlaceHolder = liquid_primitives::__Liquid_Getter_Index_Placeholder;

#[liquid::contract]
mod noop {
    use super::*;

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
            self.a(GetterIndexPlaceHolder {})
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
