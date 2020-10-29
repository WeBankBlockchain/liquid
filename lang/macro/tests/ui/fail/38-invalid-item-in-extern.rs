use liquid_lang as liquid;

#[liquid::interface(name = auto)]
mod foo {
    extern "liquid" {
        fn bar();

        #[link_name = "environ"]
        static libc_environ: *const *const std::os::raw::c_char;
    }
}

fn main() {}
