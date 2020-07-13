use liquid_macro::seq;

struct Proc {
    id: usize,
}

impl Proc {
    const fn new(id: usize) -> Self {
        Self { id }
    }
}

const PROCS: [Proc; 256] = seq!(N in 0..256 {
    [
        #(Proc::new(N as usize),)*
    ]
});

fn main() {
    assert_eq!(PROCS[32].id, 32);
}
