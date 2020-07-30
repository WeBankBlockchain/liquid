use liquid_lang as liquid;

use liquid::State;
#[derive(State)]
pub struct MyState {
    b: bool,
    i: i32,
}

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::MyState;
    use liquid_core::storage;

    #[liquid(storage)]
    struct Noop {
        value: storage::Value<MyState>,
    }

    impl Noop {
        #[liquid(constructor)]
        fn init(&mut self) {
            self.value.initialize(MyState { b: false, i: 0i32 });
        }

        #[liquid(external)]
        fn noop(&self) -> bool {
            self.value.b
        }
    }
}

fn main() {}
