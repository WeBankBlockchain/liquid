use liquid_macro::seq;

seq!(N in 0..4 {
    enum Cat {
        #(Cat#+1#N,)*
    }
});

fn main() {
    let _cat = Cat::Cat1;
    let _cat = Cat::Cat2;
    let _cat = Cat::Cat3;
    let _cat = Cat::Cat4;
}
