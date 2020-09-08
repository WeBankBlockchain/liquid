#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::contract(version = "0.1.0")]
mod erc20 {
    type Balance = u128;
    use liquid_core::storage;

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
            self.balances.insert(&caller, initial_supply);
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
            self.allowances.insert(&(owner, spender), value);
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

            self.allowances.insert(&(from, caller), allowance - value);
            self.transfer_from_to(from, to, value)
        }

        fn balance_of_or_zero(&self, owner: &Address) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &Address, spender: &Address) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
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

            self.balances.insert(&from, from_balance - value);
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(&to, to_balance + value);
            self.env().emit(Transfer { from, to, value });
            true
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid_core::env::test;

        #[test]
        fn new_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;

            test::push_execution_context(alice);
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

            test::push_execution_context(alice);
            let contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice), 100);
            assert_eq!(contract.balance_of(bob), 0);
        }

        #[test]
        fn transfer_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;

            test::push_execution_context(alice);
            let mut contract = Erc20::new(100);

            assert_eq!(contract.balance_of(bob), 0);
            assert_eq!(contract.transfer(bob, 10), true);

            let events = test::get_events();
            assert_eq!(events.len(), 1);
            let transfer_event = events.last().unwrap();
            assert_eq!(transfer_event.topics.len(), 3);
            assert_eq!(
                transfer_event.topics[0],
                "0x27772adc63db07aae765b71eb2b533064fa781bd57457e1b138592d8198d0959"
                    .into()
            );
            assert_eq!(
                transfer_event.topics[1],
                "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .into()
            );
            assert_eq!(
                transfer_event.topics[2],
                "0x0000000000000000000000000101010101010101010101010101010101010101"
                    .into()
            );
            assert_eq!(transfer_event.decode_data::<u128>(), 10);

            assert_eq!(contract.balance_of(bob), 10);
            assert_eq!(contract.balance_of(alice), 90);
        }

        #[test]
        fn not_enough_balance() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;

            test::push_execution_context(alice);
            let mut contract = Erc20::new(100);

            assert_eq!(contract.balance_of(bob), 0);
            assert_eq!(contract.transfer(bob, 1000), false);
            assert_eq!(contract.balance_of(bob), 0);
            assert_eq!(contract.balance_of(alice), 100);
        }

        #[test]
        fn transfer_from_works() {
            let accounts = test::default_accounts();
            let alice = accounts.alice;
            let bob = accounts.bob;
            let charlie = accounts.charlie;

            test::push_execution_context(alice);
            let mut contract = Erc20::new(100);
            test::pop_execution_context();

            test::push_execution_context(bob);
            assert_eq!(contract.transfer_from(alice, charlie, 10), false);
            test::pop_execution_context();

            test::push_execution_context(alice);
            assert_eq!(contract.approve(bob, 10), true);
            test::pop_execution_context();

            let events = test::get_events();
            assert_eq!(events.len(), 1);
            let approval_event = events.last().unwrap();
            assert_eq!(approval_event.topics.len(), 3);
            assert_eq!(
                approval_event.topics[0],
                "0x444360fd9f99263247bc59eb6f6c9f5d7f1096ba7962aa22cb94c3f5b743eded"
                    .into()
            );
            assert_eq!(
                approval_event.topics[1],
                "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .into()
            );
            assert_eq!(
                approval_event.topics[2],
                "0x0000000000000000000000000101010101010101010101010101010101010101"
                    .into()
            );
            assert_eq!(approval_event.decode_data::<u128>(), 10);

            test::push_execution_context(bob);
            assert_eq!(contract.transfer_from(alice, charlie, 10), true);
            assert_eq!(contract.balance_of(alice), 90);
            assert_eq!(contract.balance_of(charlie), 10);
            assert_eq!(contract.balance_of(bob), 0);
        }
    }
}
