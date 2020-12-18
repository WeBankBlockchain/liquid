# Liquid - Makes smart contract smarter

[![GitHub license](https://img.shields.io/badge/%20license-Apache%202.0-green)](https://github.com/vita-dounai/liquid/blob/dev/LICENSE)
[![Code Lines](https://tokei.rs/b1/github/vita-dounai/liquid)](https://github.com/vita-dounai/liquid)

This project is based on an earlier project named [ink!](https://github.com/paritytech/ink) which is maintained by [Parity Technologies](https://www.parity.io/), special thanks to them for their impressive work.

**Liquid** is an Rust-based e-DSL programming language to write smart contract for [FISCO BCOS](https://github.com/FISCO-BCOS/FISCO-BCOS). A simple hello-world contract written in **Liquid** is as following:

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract]
mod hello_world {
    use super::*;
    use liquid::storage;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    #[liquid(methods)]
    impl HelloWorld {
        pub fn new(&mut self) {
            self.name.initialize(String::from("Hello, World!"));
        }

        pub fn get(&self) -> String {
            self.name.clone()
        }

        pub fn set(&mut self, name: String) {
            *self.name = name;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_works() {
            let contract = HelloWorld();
            assert_eq!(contract.get(), "Hello, World!".to_owned());
        }

        #[test]
        fn set_works() {
            let new_name = "Bye, world!".to_owned();
            let mut contract = HelloWorld();
            contract.set(new_name.clone());
            assert_eq!(contract.get(), new_name);
        }
    }
}
```
