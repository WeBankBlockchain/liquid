use liquid_macro::seq;

trait A {
    fn len() -> usize;
}

macro_rules! impl_a {
    ($(($f1:tt, $f2:tt),)*) => {
        impl<$($f1, $f2),*> A for ($(($f1, $f2)),*) {
            fn len() -> usize {
                42
            }
        }
    };
}

seq!(N in 0..2 {
    impl_a!{
        #((T#N, S#N),)*
    }
});

fn main() {
    let _ = <((u8, u8), (u8, u8)) as A>::len();
}
