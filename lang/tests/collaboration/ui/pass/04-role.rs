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
            create! { Self =>
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
        pub fn send_iou(&self, iou: Iou) -> ContractId<Iou> {
            assert!(iou.cash > 0);
            assert!(self.sender == iou.owner);
            iou.mutual_transfer(self.receiver)
        }
    }
}

fn main() {}
