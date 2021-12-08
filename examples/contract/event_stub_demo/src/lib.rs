#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod event_stub_demo {
    use super::*;

    /// Defines the state variables of your contract.
    #[liquid(storage)]
    struct EventStubDemo {
        hello: storage::Value<String>,
    }

    #[liquid(event)]
    struct Transfer {
        #[liquid(indexed)]
        from_account: String,
        #[liquid(indexed)]
        to_account: String,
        #[liquid(indexed)]
        amount: u256,
    }

    #[liquid(event)]
    struct TransferAccount {
        #[liquid(indexed)]
        from_account: String,
        #[liquid(indexed)]
        to_account: String,
    }

    #[liquid(event)]
    struct TransferAmount {
        #[liquid(indexed)]
        amount: u256,
    }

    #[liquid(event)]
    struct EchoU {
        #[liquid(indexed)]
        u: u256,
    }

    #[liquid(event)]
    struct EchoI {
        #[liquid(indexed)]
        i: i256,
    }

    #[liquid(event)]
    struct EchoS {
        #[liquid(indexed)]
        s: String,
    }

    #[liquid(event)]
    struct EchoUIS {
        #[liquid(indexed)]
        u: u256,
        #[liquid(indexed)]
        i: i256,
        #[liquid(indexed)]
        s: String,
    }

    #[liquid(event)]
    struct EchoBn {
        #[liquid(indexed)]
        bsn: bytes32,
    }

    #[liquid(event)]
    struct EchoI8 {
        #[liquid(indexed)]
        i: i8,
    }

    #[liquid(event)]
    struct Echo {
        #[liquid(indexed)]
        bsn: bytes32,
        #[liquid(indexed)]
        i: i8,
    }

    /// Defines the methods of your contract.
    #[liquid(methods)]
    impl EventStubDemo {
        /// Defines the constructor which will be executed automatically when the contract is
        /// under deploying. Usually constructor is used to initialize state variables.
        ///
        /// # Note
        /// 1. The name of constructor must be `new`;
        /// 2. The receiver of constructor must be `&mut self`;
        /// 3. The visibility of constructor must be `pub`.
        /// 4. The constructor should return nothing.
        /// 5. If you forget to initialize state variables, you
        ///    will be trapped in an runtime-error for attempting
        ///    to visit uninitialized storage.
        pub fn new(&mut self) {
            self.hello.initialize(String::from("hello"))
        }

        pub fn transfer(
            &mut self,
            from_account: String,
            to_account: String,
            amount: u256,
        ) {
            self.env().emit(Transfer {
                from_account: from_account.clone(),
                to_account: to_account.clone(),
                amount: amount.clone(),
            });
            self.env().emit(TransferAccount {
                from_account: from_account.clone(),
                to_account: to_account.clone(),
            });
            self.env().emit(TransferAmount {
                amount: amount.clone(),
            });
        }

        pub fn echo1(&mut self, u: u256, i: i256, s: String) -> (u256, i256, String) {
            self.env().emit(EchoU { u: u.clone() });
            self.env().emit(EchoI { i: i.clone() });
            self.env().emit(EchoS { s: s.clone() });
            self.env().emit(EchoUIS {
                u: u.clone(),
                i: i.clone(),
                s: s.clone(),
            });
            return (u, i, s);
        }

        pub fn echo2(&mut self, bsn: bytes32, i: i8) -> (bytes32, i8) {
            self.env().emit(EchoBn { bsn });
            self.env().emit(EchoI8 { i });
            self.env().emit(Echo { bsn, i });
            return (bsn, i);
        }
    }
}
