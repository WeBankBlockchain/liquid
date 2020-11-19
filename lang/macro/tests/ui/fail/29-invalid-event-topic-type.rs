use liquid_lang as liquid;

#[liquid::contract(version = "0.2.0")]
mod noop {
    #[liquid(storage)]
    struct Noop {}

    #[liquid(event)]
    struct TestEvent {
        i: i32,
        v: Vec<i32>,
        #[liquid(indexed)]
        tv: Vec<i32>,
    }

    #[liquid(methods)]
    impl Noop {
        pub fn new(&mut self) {}

        pub fn noop(&self) -> () {
            self.env().emit(TestEvent {
                i: 1,
                v: Vec::new(),
                tv: Vec::new(),
            });
        }
    }
}

fn main() {}
