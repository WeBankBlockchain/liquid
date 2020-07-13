use liquid_macro::seq;

seq!(N in 0..4 {
    enum Cat {
        #(Cat#N,)*
    }
});

fn main() {
    let _cat = Cat::Cat0;
    let _cat = Cat::Cat1;
    let _cat = Cat::Cat2;
    let _cat = Cat::Cat3;
}
