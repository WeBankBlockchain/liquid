// https://docs.daml.com/daml/intro/6_Parties.html#use-role-contracts-for-ongoing-authorization
//
// Many actions, like the issuance of assets or their transfer,
// can be pre-agreed. You can represent this succinctly through relationship
// or role contracts.
//
// # Workflow
// 1. `receiver` creates an `IouSender` contract that indicates the `sender` on ledger.
// 2. `sender` excises `send_iou` choice on the `IouSender` contract.
// 3. An `Iou` transfer from `sender` to `receiver`.

#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::collaboration]
mod iou {
    #[liquid(contract)]
    pub struct Iou {
        #[liquid(signers)]
        issuer: address,
        #[liquid(signers)]
        owner: address,
        cash: u32,
    }

    #[liquid(rights)]
    impl Iou {
        #[liquid(belongs_to = "owner, ^new_owner")]
        pub fn mutual_transfer(self, new_owner: address) -> ContractId<Iou> {
            sign! { Iou =>
                owner: new_owner,
                ..self
            }
        }
    }

    #[liquid(contract)]
    pub struct IouSender {
        sender: address,
        #[liquid(signers)]
        receiver: address,
    }

    #[liquid(rights)]
    impl IouSender {
        #[liquid(belongs_to = "sender")]
        // The mutability of first parameter can *NOT* be immutable for now.
        // Due to https://github.com/vita-dounai/liquid/issues/8
        pub fn send_iou(&mut self, iou_id: ContractId<Iou>) -> ContractId<Iou> {
            let iou = iou_id.fetch();
            assert!(iou.cash > 0);
            assert!(self.sender == iou.owner);
            iou_id.mutual_transfer(self.receiver)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid_lang::env::test;

        fn create_iou(issuer: address, cash: u32) -> ContractId<Iou> {
            test::set_caller(issuer);
            let iou_id = sign! { Iou =>
                issuer,
                owner: issuer,
                cash,
            };
            test::pop_execution_context();
            iou_id
        }

        #[test]
        fn ongoing_transfer() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(bob);
            let iou_sender_id = sign! { IouSender =>
                sender: alice,
                receiver: bob,
            };
            test::pop_execution_context();

            let iou_id = create_iou(alice, 100);
            test::set_caller(alice);
            let iou_id = iou_sender_id.send_iou(iou_id);
            test::pop_execution_context();

            let iou = iou_id.fetch();
            assert_eq!(iou.issuer, alice);
            assert_eq!(iou.owner, bob);
            assert_eq!(iou.cash, 100);

            let iou_id = create_iou(alice, 200);
            test::set_caller(alice);
            let iou_id = iou_sender_id.send_iou(iou_id);
            test::pop_execution_context();

            let iou = iou_id.fetch();
            assert_eq!(iou.issuer, alice);
            assert_eq!(iou.owner, bob);
            assert_eq!(iou.cash, 200);
        }

        #[test]
        #[should_panic]
        fn unauthorized_send() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;

            test::set_caller(bob);
            let iou_sender_id = sign! { IouSender =>
                sender: charlie,
                receiver: bob,
            };
            test::pop_execution_context();

            let iou_id = create_iou(alice, 100);
            test::set_caller(alice);
            let _ = iou_sender_id.send_iou(iou_id);
            test::pop_execution_context();
        }

        #[test]
        #[should_panic]
        fn invalid_iou() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(bob);
            let iou_sender_id = sign! { IouSender =>
                sender: alice,
                receiver: bob,
            };
            test::pop_execution_context();

            let iou_id = create_iou(bob, 100);
            test::set_caller(alice);
            let _ = iou_sender_id.send_iou(iou_id);
            test::pop_execution_context();
        }
    }
}
