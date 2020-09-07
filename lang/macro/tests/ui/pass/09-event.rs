use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod noop {
    use super::*;

    #[liquid(storage)]
    struct Noop {}

    #[derive(liquid::InOut)]
    pub struct Log {
        i: i8,
        s: String,
    }

    #[liquid(event)]
    struct TestEvent {
        i: i8,
        #[liquid(indexed)]
        b: bool,
        x: i16,
        y: i32,
        #[liquid(indexed)]
        z: i64,
        #[liquid(indexed)]
        s: String,
        log: Log,
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
                log: Log {
                    i: 0,
                    s: String::from("456"),
                },
            });
        }
    }
}

fn main() {}
