use liquid_macro::seq;

macro_rules! expand_to_nothing {
    ($arg: literal) => {
        // nothing
    };
}

seq!(N in (0)..(4) {
    expand_to_nothing!(N);
});

fn main() {}