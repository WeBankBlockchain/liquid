# Liquid - Makes Smart Contract Smarter

[![GitHub license](https://img.shields.io/badge/%20license-Apache%202.0-green)](https://github.com/vita-dounai/liquid/blob/dev/LICENSE)
[![Code Lines](https://tokei.rs/b1/github/WeBankBlockchain/Liquid/)](https://github.com/WeBankBlockchain/Liquid)
[![Latest release](https://img.shields.io/github/release/WebankBlockchain/liquid.svg)](https://github.com/WebankBlockchain/liquid/releases/latest)
[![Language](https://img.shields.io/badge/Language-Rust-blue.svg)](https://www.rust-lang.org/)

Liquid 由微众银行区块链团队开发并完全开源，是一种[嵌入式领域特定语言](http://wiki.haskell.org/Embedded_domain_specific_language)（ embedded Domain Specific Language，eDSL），能够用来编写运行于区块链底层平台[FISCO BCOS](https://github.com/FISCO-BCOS/FISCO-BCOS)的智能合约。

## 关键特性

### 安全（Security)

-   内置[线性资产模型](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/asset/asset.html)，对安全可控、不可复制的资产类型进行了高级抽象，确保链上资产类应用具备金融级安全性；

-   支持在智能合约内部便捷地编写单元测试用例，可通过内嵌的区块链模拟环境直接在本地执行；

-   算数溢出及内存越界安全检查；

-   能够结合模糊测试等工具进行深度测试；

-   未来将进一步集成形式化验证及数据隐私保护技术。

### 性能（Performance）

-   配合 LLVM 优化器，支持将智能合约代码编译为可移植、体积小、加载快 Wasm 格式字节码；

-   对 Wasm 执行引擎进行了深度优化，并支持交易并行化等技术；

-   结合 Tree-Shaking 等技术，进一步压缩智能合约体积。

### 体验（Experience）

-   支持使用大部分现代语言特性（如移动语义及自动类型推导等）；

-   提供专有开发工具及编辑器插件辅助开发；

-   丰富的标准库及第三方组件。

### 可定制（Customization）

-   能够根据业务需求对编程模型、语言文法的进行深度定制。目前已集成[可编程分布式协作编程模型](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/pdc/introduction.html)；

-   未来还将进一步探索如何与隐私保护、跨链协同等功能相结合。

## 合约示例

使用 Liquid 编写的 HelloWorld 合约如下所示：

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

## 技术文档

阅读[Liquid 在线技术文档](https://liquid-doc.readthedocs.io/zh_CN/latest/index.html)，详细了解如何使用 Liquid。

-   [快速开始](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/quickstart/prerequisite.html)

-   [基础语法](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/contract/contract_mod.html)

-   [开发与测试](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/dev_testing/development.html)

-   [线性资产模型](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/asset/asset.html)

-   [可编程分布式协作](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/pdc/introduction.html)

-   [参考](https://liquid-doc.readthedocs.io/zh_CN/latest/docs/advance/metaprogramming.html)

## 架构设计

![](https://liquid-doc.readthedocs.io/zh_CN/latest/_static/images/advance/liquid_arch.png)

## 社区

-   [智能合约编译技术专项兴趣小组](https://mp.weixin.qq.com/s/NfBZtPWxXdnP0XLLGrQKow)

## License

Liquid 的开源协议为 Apache License 2.0，详情请参考[LICENSE](./LICENSE)。
