use liquid_lang as liquid;

#[liquid::collaboration]
mod iou {
    #[liquid(definitions)]
    struct Iou {
        #[liquid(signers)]
        issuer: address,
        #[liquid(signers)]
        owner: address,
        cash: u32,
    }

    #[liquid(rights)]
    impl Iou {
        #[liquid(belongs_to = "owner, !new_owner")]
        pub fn mutual_transfer(self, new_owner: address) -> Iou {
            create! { Self =>
                owner: new_owner,
                ..self
            }
        }
    }

    #[liquid(definitions)]
    struct IouSender {
        sender: address,
        #[liquid(signers)]
        receiver: address,
    }

    #[liquid(rights)]
    impl IouSender {
        #[liquid(belongs_to = "sender")]
        pub fn send_iou(&self, iou: Iou) -> Iou {
            let iou = fetch!(iou);
            assert!(iou.cash > 0);
            assert!(self.sender == iou.owner);
            iou.mutual_transfer(self.receiver.clone())
        }
    }
}
