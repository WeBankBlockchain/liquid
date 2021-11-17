use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {
        foo: storage::Value<FixedPointU64F16>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.foo.initialize(Default::default());
        }

        pub fn noop(&self) -> FixedPointU64F16 {
            FixedPointU64F16::from_user("011")
        }
    }
}

fn main() {}
