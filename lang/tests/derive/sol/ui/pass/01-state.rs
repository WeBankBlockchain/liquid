use liquid::{storage, State};
use liquid_lang as liquid;

#[derive(State)]
pub struct MyState {
    b: bool,
    i: i32,
}

#[derive(State)]
pub struct MyState2 {
    b: bool,
    i: i32,
}

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {
        _0: storage::Value<MyState>,
        _1: storage::Vec<MyState>,
        _2: storage::Mapping<MyState, MyState2>,
        _3: storage::IterableMapping<MyState, MyState2>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self._0.initialize(MyState { b: false, i: 0i32 });
            self._1.initialize();
            self._2.initialize();
            self._3.initialize();
        }

        pub fn noop_1(&self) -> bool {
            self._0.b
        }

        pub fn noop_2(&mut self) {
            self._1.push(MyState { b: false, i: 0i32 });
            self._2.insert(
                &MyState { b: false, i: 0i32 },
                MyState2 { b: false, i: 0i32 },
            );
            self._3.insert(
                MyState { b: false, i: 0i32 },
                MyState2 { b: false, i: 0i32 },
            );
        }
    }
}

fn main() {}
