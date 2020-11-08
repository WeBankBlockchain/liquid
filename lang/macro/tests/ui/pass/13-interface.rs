#![feature(unboxed_closures, fn_traits)]

use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod iface {
    extern "liquid" {
        fn get(key: String) -> i256;

        fn set(key: String, value: address);
        fn set(key: String, value: String);
    }
}

#[liquid::contract(version = "0.2.0")]
mod noop {
    use super::iface::*;
    use liquid_core::storage;

    #[liquid(storage)]
    struct Noop {
        iface: storage::Value<Iface>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {
            self.iface.initialize(Iface::at(Default::default()));

            let _ = self.iface.get().get(String::from("noop"));
            (self.iface.set)(String::from("noop"), address::default());
            (self.iface.set)(String::from("noop"), String::from("noop"));
        }

        pub fn noop(&self) {
            let iface = Iface::at(Default::default());

            let _ = iface.get(String::from("noop"));
            (iface.set)(String::from("noop"), address::default());
            (iface.set)(String::from("noop"), String::from("noop"));
        }
    }
}

fn main() {}
