#![cfg_attr(not(feature = "std"), no_std)]

use liquid::storage;
use liquid_lang as liquid;

#[liquid::contract]
mod erc20 {
    use super::*;
    type Balance = u128;

    #[liquid(storage)]
    struct Erc20 {
        pub total_supply: storage::Value<Balance>,
        balances: storage::Mapping<Address, Balance>,
        allowances: storage::Mapping<(Address, Address), Balance>,
    }

    #[liquid(event)]
    struct Transfer {
        #[liquid(indexed)]
        from: Address,
        #[liquid(indexed)]
        to: Address,
        value: u128,
    }

    #[liquid(event)]
    struct Approval {
        #[liquid(indexed)]
        owner: Address,
        #[liquid(indexed)]
        spender: Address,
        value: u128,
    }

    #[liquid(methods)]
    impl Erc20 {
        pub fn new(&mut self, initial_supply: Balance) {
            let caller = self.env().get_caller();
            self.total_supply.initialize(initial_supply);
            self.balances.initialize();
            self.balances.insert(caller, initial_supply);
            self.allowances.initialize();
        }

        pub fn balance_of(&self, owner: Address) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        pub fn allowance(&self, owner: Address, spender: Address) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        pub fn transfer(&mut self, to: Address, value: Balance) -> bool {
            let from = self.env().get_caller();
            self.transfer_from_to(from, to, value)
        }

        pub fn approve(&mut self, spender: Address, value: Balance) -> bool {
            let owner = self.env().get_caller();
            self.allowances
                .insert((owner.clone(), spender.clone()), value);
            self.env().emit(Approval {
                owner,
                spender,
                value,
            });
            true
        }

        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            value: Balance,
        ) -> bool {
            let caller = self.env().get_caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return false;
            }

            self.allowances
                .insert((from.clone(), caller.clone()), allowance - value);
            self.transfer_from_to(from, to, value)
        }

        fn balance_of_or_zero(&self, owner: &Address) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &Address, spender: &Address) -> Balance {
            *self
                .allowances
                .get(&(owner.clone(), spender.clone()))
                .unwrap_or(&0)
        }

        fn transfer_from_to(
            &mut self,
            from: Address,
            to: Address,
            value: Balance,
        ) -> bool {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return false;
            }

            self.balances.insert(from.clone(), from_balance - value);
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to.clone(), to_balance + value);
            self.env().emit(Transfer { from, to, value });
            true
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid::env::test;

        #[test]
        fn new_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;

            test::set_caller(alice);
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply, 100);
            assert_eq!(contract.balances.len(), 1);
            assert_eq!(contract.allowances.len(), 0);
        }

        #[test]
        fn balance_of_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;

            test::set_caller(alice.clone());
            let contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice.clone()), 100);
            assert_eq!(contract.balance_of(bob.clone()), 0);
        }

        #[test]
        fn transfer_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;

            test::set_caller(alice.clone());
            let mut contract = Erc20::new(100);

            assert_eq!(contract.balance_of(bob.clone()), 0);
            assert_eq!(contract.transfer(bob.clone(), 10), true);

            let events = test::get_events();
            assert_eq!(events.len(), 1);
            let transfer_event = events.last().unwrap();
            assert_eq!(transfer_event.topics.len(), 3);
            assert_eq!(
                transfer_event.topics[0],
                "0xf099cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                transfer_event.topics[1],
                liquid_primitives::hash::hash(alice.as_bytes()).into(),
            );
            assert_eq!(
                transfer_event.topics[2],
                liquid_primitives::hash::hash(bob.as_bytes()).into(),
            );
            println!("{:?}", transfer_event);
            assert_eq!(transfer_event.decode_data::<u128>(), 10);

            assert_eq!(contract.balance_of(bob.clone()), 10);
            assert_eq!(contract.balance_of(alice.clone()), 90);
        }

        #[test]
        fn not_enough_balance() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;

            test::set_caller(alice.clone());
            let mut contract = Erc20::new(100);

            assert_eq!(contract.balance_of(bob.clone()), 0);
            assert_eq!(contract.transfer(bob.clone(), 1000), false);
            assert_eq!(contract.balance_of(bob), 0);
            assert_eq!(contract.balance_of(alice), 100);
        }

        #[test]
        fn transfer_from_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;
            let charlie = accounts.charlie;

            test::set_caller(alice.clone());
            let mut contract = Erc20::new(100);
            test::pop_execution_context();

            test::set_caller(bob.clone());
            assert_eq!(
                contract.transfer_from(alice.clone(), charlie.clone(), 10),
                false
            );
            test::pop_execution_context();

            test::set_caller(alice.clone());
            assert_eq!(contract.approve(bob.clone(), 10), true);
            test::pop_execution_context();

            let events = test::get_events();
            assert_eq!(events.len(), 1);
            let approval_event = events.last().unwrap();
            assert_eq!(approval_event.topics.len(), 3);
            assert_eq!(
                approval_event.topics[0],
                "0x7134692b230b9e1ffa39098904722134159652b09c5bc41d88d6698779d228ff"
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                approval_event.topics[1],
                liquid_primitives::hash::hash(alice.as_bytes()).into()
            );
            assert_eq!(
                approval_event.topics[2],
                liquid_primitives::hash::hash(bob.as_bytes()).into()
            );
            assert_eq!(approval_event.decode_data::<u128>(), 10);

            test::set_caller(bob.clone());
            assert_eq!(
                contract.transfer_from(alice.clone(), charlie.clone(), 10),
                true
            );
            assert_eq!(contract.balance_of(alice), 90);
            assert_eq!(contract.balance_of(charlie), 10);
            assert_eq!(contract.balance_of(bob), 0);
        }
    }
}
