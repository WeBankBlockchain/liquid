use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    #[liquid(storage)]
    struct Noop<T> {
        t: T,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
