use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    impl Noop {
        pub fn constructor(&mut self) {}

        pub fn noop(&self) {}
    }
}

fn main() {}