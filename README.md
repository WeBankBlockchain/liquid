# Liquid - Makes Smart Contract Smarter

[![GitHub license](https://img.shields.io/badge/%20license-Apache%202.0-green)](https://github.com/vita-dounai/liquid/blob/dev/LICENSE)
[![Code Lines](https://tokei.rs/b1/github/WeBankBlockchain/Liquid/dev)](https://github.com/WeBankBlockchain/Liquid)

Liquid 由微众银行区块链团队开发并完全开源，是一种基于[Rust 语言](https://www.rust-lang.org/)的[嵌入式领域特定语言](http://wiki.haskell.org/Embedded_domain_specific_language>)（ embedded Domain Specific Language，eDSL），能够用于编写在[FISCO BCOS](https://github.com/FISCO-BCOS/FISCO-BCOS)区块链底层平台上运行的智能合约。使用 Liquid 编写的 HelloWorld 合约如下所示：

```rust
#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod hello_world {
    use super::*;

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
    }

    #[liquid(methods)]
    impl HelloWorld {
        pub fn new(&mut self) {
            self.name.initialize(String::from("Alice"));
        }

        pub fn get(&self) -> String {
            self.name.clone()
        }

        pub fn set(&mut self, name: String) {
            self.name.set(name)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_works() {
            let contract = HelloWorld::new();
            assert_eq!(contract.get(), "Alice");
        }

        #[test]
        fn set_works() {
            let mut contract = HelloWorld::new();

            let new_name = String::from("Bob");
            contract.set(new_name.clone());
            assert_eq!(contract.get(), "Bob");
        }
    }
}
```
