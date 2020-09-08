use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(event)]
    struct TestEvent {
        f: f32,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) -> () {
            self.env().emit(TestEvent { f: 1.0 });
        }
    }
}

fn main() {}
