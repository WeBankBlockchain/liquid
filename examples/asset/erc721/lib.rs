#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract]
mod asset_erc20 {
    use liquid_lang::storage;

    /// Defines the storage of your contract.
    #[liquid(storage)]
    struct Erc721 {
        allowances: storage::Mapping<u64, (Address, Address)>,
        ownership: storage::Mapping<Address, Vec<u64>>,
        operators: storage::Mapping<(Address, Address), bool>,
    }

    #[liquid(asset(
        issuer = "0x83309d045a19c44dc3722d15a6abd472f95866ac",
        fungible = false,
        description = "这是一个erc721测试"
    ))]
    struct Erc721Token;

    /// Defines the methods of your contract.
    #[liquid(methods)]
    impl Erc721 {
        /// Constructor that initializes your storage.
        /// # Note
        /// 1. The name of constructor must be `new`;
        /// 2. The receiver of constructor must be `&mut self`;
        /// 3. The visibility of constructor must be `pub`.
        /// 4. The constructor should return nothing.
        pub fn new(&mut self) {
            self.allowances.initialize();
            self.ownership.initialize();
            self.operators.initialize();
        }

        pub fn total_supply(&self) -> u64 {
            Erc721Token::total_supply()
        }
        pub fn balance_of(&self, owner: Address) -> u64 {
            Erc721Token::balance_of(&owner)
        }
        pub fn erc721_description(&self) -> String {
            Erc721Token::description().into()
        }
        pub fn issue_to(&mut self, owner: Address, uri: String) -> u64 {
            Erc721Token::issue_to(&owner, &uri).unwrap()
        }
        pub fn owner_of(&self, token_id: u64) -> Address {
            match self.allowances.get(&token_id).as_ref() {
                Some(&(owner, _)) => owner.clone(),
                None => Default::default(),
            }
        }

        pub fn safe_transfer_from(
            &mut self,
            owner: Address,
            recipient: Address,
            token_id: u64,
        ) -> bool {
            let caller = self.env().get_caller();

            if self.is_approved_or_owner(caller.clone(), token_id) {
                return match Erc721Token::withdraw_from_self(token_id) {
                    None => false,
                    Some(token) => {
                        token.deposit(&recipient);
                        self.allowances.remove(&token_id);
                        true
                    }
                };
            } else if self.is_approved_for_all(owner.clone(), caller)
                && self.own_token(owner.clone(), token_id)
            {
                return match Erc721Token::withdraw_from_self(token_id) {
                    None => false,
                    Some(token) => {
                        token.deposit(&recipient);
                        // operators
                        let tokens = self.ownership.get_mut(&owner).unwrap();
                        for i in 0..tokens.len() {
                            if tokens[i] == token_id {
                                tokens.swap_remove(i);
                                break;
                            }
                        }
                        true
                    }
                };
            }
            false
        }
        pub fn safe_transfer(&mut self, recipient: Address, token_id: u64) -> bool {
            match Erc721Token::withdraw_from_caller(token_id) {
                None => false,
                Some(token) => {
                    token.deposit(&recipient);
                    true
                }
            }
        }
        // Only a single account can be approved at a time, so approving the zero address clears previous approvals.
        pub fn approve(&mut self, spender: Address, token_id: u64) -> bool {
            // if spender is 0x0, return token to owner
            if spender == Default::default() {
                return match Erc721Token::withdraw_from_self(token_id) {
                    None => false,
                    Some(token) => {
                        let (owner, _) = self.allowances.get(&token_id).unwrap();
                        token.deposit(&owner);
                        self.allowances.remove(&token_id);
                        true
                    }
                };
            }
            let caller = self.env().get_caller();
            let mut has_token = false;
            if let Some((owner, _)) = self.allowances.get(&token_id) {
                require(caller == *owner, "only owner can approve");
                has_token = true;
            }
            if has_token {
                self.allowances.insert(token_id, (caller, spender));
                return true;
            }
            match Erc721Token::withdraw_from_caller(token_id) {
                None => false,
                Some(token) => {
                    token.deposit(&self.env().get_address());
                    self.allowances.insert(token_id, (caller, spender));
                    true
                }
            }
        }
        pub fn get_approved(&self, token_id: u64) -> Address {
            match self.allowances.get(&token_id).as_ref() {
                Some(&(_, operator)) => operator.clone(),
                None => Default::default(),
            }
        }
        pub fn set_approval_for_all(&mut self, operator: Address, approval: bool) {
            let caller = self.env().get_caller();
            require(caller != operator, "approve to caller");
            if approval {
                let token_ids = Erc721Token::tokens_of(&caller);
                for token_id in token_ids.iter() {
                    let token = Erc721Token::withdraw_from_caller(*token_id).unwrap();
                    token.deposit(&self.env().get_address());
                }
                if !token_ids.is_empty() {
                    self.ownership.insert(caller.clone(), token_ids.clone());
                    self.operators
                        .insert((caller.clone(), caller.clone()), approval);
                }
            }
            self.operators.insert((caller.clone(), operator), approval);
        }

        pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
            match self.operators.get(&(owner, operator)) {
                Some(approved) => *approved,
                None => false,
            }
        }
        fn own_token(&self, owner: Address, token_id: u64) -> bool {
            let tokens = self.ownership.get(&owner).unwrap();
            tokens.iter().any(|id| *id == token_id)
        }
        fn is_owner(&self, spender: Address, token_id: u64) -> bool {
            let default_address: (Address, Address) =
                (Default::default(), Default::default());
            let (owner, _) = self.allowances.get(&token_id).unwrap_or(&default_address);
            *owner == spender
        }
        fn is_approved(&self, spender: Address, token_id: u64) -> bool {
            let default_address: (Address, Address) =
                (Default::default(), Default::default());
            let (_, operator) =
                self.allowances.get(&token_id).unwrap_or(&default_address);
            *operator == spender
        }
        fn is_approved_or_owner(&self, spender: Address, token_id: u64) -> bool {
            if self.is_owner(spender.clone(), token_id) {
                return true;
            }
            if self.is_approved(spender, token_id) {
                return true;
            }
            false
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use liquid::env::test;
//     use crate::asset_erc20::__liquid_private::Erc721Token;

//     #[test]
//     fn issue_to_works() {
//         let success = asset_erc20::Erc721Token::register();
//         assert_eq!(success, true);
//         let accounts = test::default_accounts();
//         let alice = accounts.alice;

//         test::set_caller(alice);
//         let mut contract = asset_erc20::Erc721::new();
//         let id = contract.issue_to(accounts.bob, String::from("token 1"));
//         assert_eq!(success, 1u64);
//         let id = contract.issue_to(accounts.bob, String::from("token 2"));
//         assert_eq!(success, 2u64);
//         let id = contract.issue_to(accounts.bob, String::from("token 3"));
//         assert_eq!(success, 3u64);
//         let id = contract.issue_to(accounts.charlie, String::from("token 4"));
//         assert_eq!(success, 4u64);
//         let id = contract.issue_to(accounts.charlie, String::from("token 5"));
//         assert_eq!(success, 5u64);
//         let id = contract.issue_to(accounts.charlie, String::from("token 6"));
//         assert_eq!(success, 6u64);
//     }
// }
