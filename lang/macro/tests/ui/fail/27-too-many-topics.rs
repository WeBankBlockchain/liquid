use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(event)]
    struct TestEvent {
        i: i8,
        #[liquid(indexed)]
        b: bool,
        x: i16,
        #[liquid(indexed)]
        y: i32,
        #[liquid(indexed)]
        z: i64,
        #[liquid(indexed)]
        s: String,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) -> () {
            self.env().emit(TestEvent {
                i: 0,
                b: true,
                x: 0,
                y: 0,
                z: 0,
                s: String::from("123"),
            });
        }
    }
}

fn main() {}
