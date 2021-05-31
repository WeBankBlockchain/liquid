use liquid::{storage, InOut};
use liquid_lang as liquid;

#[derive(InOut)]
pub struct Null {
    s: String,
    i: i32,
}

impl Default for Null {
    fn default() -> Self {
        Self {
            s: Default::default(),
            i: Default::default(),
        }
    }
}

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {
        foo: storage::Value<[String; 2]>,
        bar: storage::Value<[u8; 2]>,
    }

    #[liquid(event)]
    struct Nothing {
        _1: [Null; 2],
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.foo.initialize(Default::default());
            self.bar.initialize(Default::default());
        }

        pub fn noop(&self, a: [String; 2]) -> [Null; 2] {
            self.env().emit(Nothing {
                _1: [
                    Null {
                        s: a[0].clone(),
                        i: Default::default(),
                    },
                    Null {
                        s: a[1].clone(),
                        i: Default::default(),
                    },
                ],
            });
            Default::default()
        }
    }
}

fn main() {}
