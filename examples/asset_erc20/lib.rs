#![cfg_attr(not(feature = "std"), no_std)]
#![feature(trace_macros)]

trace_macros!(true);
use liquid_lang as liquid;

/// This declaration aims at importing `format!` macro used by `get` method.
/// If you don't need that, just remove this declaration freely.
#[macro_use]
extern crate alloc;

#[liquid::contract(version = "0.2.0")]
mod asset_erc20 {
    use liquid_lang::storage;

    /// Defines the storage of your contract.
    #[liquid(storage)]
    struct AssetErc20 {
        allowances: storage::Mapping<(address, address), u64>,
    }

    #[liquid(asset(issuer="0x83309d045a19c44dc3722d15a6abd472f95866ac", total=1000000000, description="这是一个erc20测试"))]
    struct Erc20Token;

    /// Defines the methods of your contract.
    #[liquid(methods)]
    impl AssetErc20 {
        /// Constructor that initializes your storage.
        /// # Note
        /// 1. The name of constructor must be `new`;
        /// 2. The receiver of constructor must be `&mut self`;
        /// 3. The visibility of constructor must be `pub`.
        /// 4. The constructor should return nothing.
        pub fn new(&mut self) {
            self.allowances.initialize();
        }

        pub fn total_supply(&self) -> u64 {
            Erc20Token::total_supply()
        }
        pub fn balance_of(&self, owner: address) -> u64 {
            Erc20Token::balance_of(&owner)
        }
        pub fn erc20_description(&self) -> String {
            Erc20Token::description().into()
        }
        pub fn issue_to(&mut self, owner: address, amount: u64) -> bool {
            Erc20Token::issue_to(&owner, amount)
        }
        pub fn transfer(&mut self, recipient: address, amount: u64) -> bool {
            match Erc20Token::withdraw_from_caller(amount) {
                None => false,
                Some(token) => {
                    token.deposit(&recipient);
                    true
                }
            }
        }
        pub fn allowance(&mut self, owner: address, spender: address) -> u64 {
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }
        pub fn approve(&mut self, spender: address, amount: u64) -> bool {
            match Erc20Token::withdraw_from_caller(amount) {
                None => false,
                Some(token) => {
                    token.deposit(&self.env().get_address());
                    let caller = self.env().get_caller();
                    let mut allowance : u64 = 0;
                    {
                        allowance = *self.allowances.get(&(caller, spender)).unwrap_or(&0);
                    }
                    self.allowances.insert(&(caller, spender), allowance + amount);
                    true
                }
            }
        }

        pub fn transfer_from(
            &mut self,
            sender: address,
            recipient: address,
            amount: u64,
        ) -> bool {
            let caller = self.env().get_caller();
            let allowance = *self.allowances.get(&(sender, caller)).unwrap_or(&0);
            if allowance >= amount {
                let balance = allowance - amount;
                self.allowances.insert(&(sender, caller), 0);
                return match Erc20Token::withdraw_from_self(amount) {
                    None => false,
                    Some(token) => {
                        token.deposit(&recipient);
                        true
                    }
                };
            }
            false
        }
    }

}
