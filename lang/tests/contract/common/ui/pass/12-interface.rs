#![feature(unboxed_closures, fn_traits)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod iface1 {
    extern "solidity" {
        fn getInt(&self, key: String) -> i256;

        fn set(&mut self, key: String, value: address);
        fn set(&mut self, key: String, value: String);
    }
}

#[liquid::interface(name = auto)]
mod iface2 {
    extern "liquid" {
        fn get(&self) -> bytes2;
        fn set(&mut self);
    }
}

#[liquid::contract]
mod noop {
    use super::{iface1::*, iface2::*, *};

    #[liquid(storage)]
    struct Noop {
        iface1: storage::Value<Iface1>,
        iface2: storage::Value<Iface2>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.iface1.initialize(Iface1::at(Default::default()));
            self.iface2.initialize(Iface2::at(Default::default()));

            let _ = self.iface1.getInt(String::from("noop"));
            (self.iface1.set)(String::from("noop"), address::default());
            (self.iface1.set)(String::from("noop"), String::from("noop"));

            let _ = (*self.iface2).get();
            (*self.iface2).set();
        }

        pub fn noop(&self) {
            let iface1 = Iface1::at(Default::default());
            let iface2 = Iface2::at(Default::default());

            let _ = iface1.getInt(String::from("noop"));
            (iface1.set)(String::from("noop"), address::default());
            (iface1.set)(String::from("noop"), String::from("noop"));

            let _ = iface2.get();
            iface2.set();
        }
    }
}

fn main() {}
