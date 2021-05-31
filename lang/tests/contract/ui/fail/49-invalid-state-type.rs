#![allow(unused_imports)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {
        u: u32,
        f: storage::IterableMapping<u32, f32>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
