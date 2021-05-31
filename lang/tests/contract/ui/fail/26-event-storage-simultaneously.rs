use liquid_lang as liquid;

#[liquid::contract]
mod noop {
    #[liquid(event)]
    #[liquid(storage)]
    struct Noop {}

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}
