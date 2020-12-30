// https://github.com/digital-asset/ex-models/tree/master/shop
//
// This example models a simple shop management system. Vendors can offer items,
// which can be bought by users. During a purchase the item and payment are swapped atomically.
//
// # Workflow
// 1. The producer produces `Item`s and distributes them to vendors.
// 2. The issuer issues `Iou`s and distributes them to users.
// 3. The owner creates a `Shop` contract and onboards vendors and users via invite/accept creating mutually signed relationship contracts for each.
// 4. The vendor offers an item for a set price via the `offer_item` choice on its `VendorRelationship` contract.
// 5. The user buys the item via the `buy_item` choice on its `UserRelationship` contract.
// 6. The `Item` and the `Iou` are swapped atomically between vendor and user.

#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

#[liquid::collaboration]
mod shop {
    #[liquid(contract)]
    pub struct Iou {
        #[liquid(signers)]
        issuer: address,
        owner: address,
        amount: u64,
        currency: String,
    }

    #[liquid(rights)]
    impl Iou {
        #[liquid(belongs_to = "owner")]
        pub fn transfer_iou(self, new_owner: address) -> ContractId<Iou> {
            sign! { Iou =>
                owner: new_owner,
                ..self
            }
        }
    }

    #[liquid(contract)]
    pub struct Item {
        #[liquid(signers)]
        producer: address,
        owner: address,
        label: String,
        quantity: u64,
        unit: String,
        observers: Vec<address>,
    }

    #[liquid(rights_belong_to = "owner")]
    impl Item {
        pub fn transfer_item(self, new_owner: address) -> ContractId<Item> {
            sign! { Item =>
                owner: new_owner,
                ..self
            }
        }

        pub fn disclose(self, users: Vec<address>) -> ContractId<Item> {
            sign! { Item =>
                observers: users,
                ..self
            }
        }
    }

    #[liquid(contract)]
    pub struct Offer {
        #[liquid(signers)]
        owner: address,
        #[liquid(signers)]
        vendor: address,
        item_id: ContractId<Item>,
        price: u64,
        currency: String,
        users: Vec<address>,
    }

    #[liquid(rights)]
    impl Offer {
        #[liquid(belongs_to = "owner")]
        pub fn settle(self, buyer: address) -> ContractId<Item> {
            self.item_id.transfer_item(buyer)
        }
    }

    #[liquid(contract)]
    pub struct Shop {
        #[liquid(signers)]
        owner: address,
        vendors: Vec<address>,
        users: Vec<address>,
        offer_ids: Vec<ContractId<Offer>>,
    }

    #[liquid(rights_belong_to = "owner")]
    impl Shop {
        pub fn invite_vendor(
            mut self,
            vendor: address,
        ) -> (ContractId<Shop>, ContractId<VendorInvite>) {
            self.vendors.push(vendor);
            (
                sign! { Shop =>
                    ..self
                },
                sign! { VendorInvite =>
                    vendor,
                    owner: self.owner,
                },
            )
        }

        pub fn invite_user(
            mut self,
            user: address,
        ) -> (ContractId<Shop>, ContractId<UserInvite>) {
            self.users.push(user);
            (
                sign! { Shop =>
                    ..self
                },
                sign! { UserInvite =>
                    user,
                    owner: self.owner,
                },
            )
        }
    }

    #[liquid(contract)]
    pub struct VendorInvite {
        #[liquid(signers)]
        owner: address,
        vendor: address,
    }

    #[liquid(rights)]
    impl VendorInvite {
        #[liquid(belongs_to = "vendor")]
        pub fn accept_vendor_invite(self) -> ContractId<VendorRelationship> {
            sign! { VendorRelationship =>
                owner: self.owner,
                vendor: self.vendor,
            }
        }
    }

    #[liquid(contract)]
    pub struct VendorRelationship {
        #[liquid(signers)]
        owner: address,
        #[liquid(signers)]
        vendor: address,
    }

    #[liquid(rights)]
    impl VendorRelationship {
        #[liquid(belongs_to = "vendor")]
        pub fn offer_item(
            &self,
            shop_id: ContractId<Shop>,
            item_id: ContractId<Item>,
            price: u64,
            currency: String,
        ) -> (ContractId<Shop>, ContractId<Offer>) {
            let shop = shop_id.fetch();

            let mut users = shop.users.clone();
            users.push(self.owner);
            let disclosed_item = item_id.disclose(users);

            let offer_id = sign! { Offer =>
                item_id: disclosed_item,
                users: shop.users.clone(),
                price,
                currency,
                owner: self.owner,
                vendor: self.vendor,
            };

            let mut offer_ids = shop.offer_ids.clone();
            offer_ids.push(offer_id);
            let shop_id = sign! { Shop =>
                offer_ids,
                ..shop
            };

            (shop_id, offer_id)
        }
    }

    #[liquid(contract)]
    pub struct UserInvite {
        #[liquid(signers)]
        owner: address,
        user: address,
    }

    #[liquid(rights)]
    impl UserInvite {
        #[liquid(belongs_to = "user")]
        pub fn accept_user_invite(self) -> ContractId<UserRelationship> {
            sign! { UserRelationship =>
                owner: self.owner,
                user: self.user,
            }
        }
    }

    #[liquid(contract)]
    pub struct UserRelationship {
        #[liquid(signers)]
        owner: address,
        #[liquid(signers)]
        user: address,
    }

    #[liquid(rights)]
    impl UserRelationship {
        #[liquid(belongs_to = "user")]
        pub fn buy_item(
            &self,
            shop_id: ContractId<Shop>,
            offer_id: ContractId<Offer>,
            iou_id: ContractId<Iou>,
        ) -> (ContractId<Shop>, ContractId<Item>, ContractId<Iou>) {
            let shop = shop_id.fetch();
            let offer = offer_id.fetch();
            let iou = iou_id.fetch();

            assert_eq!(offer.price, iou.amount);
            assert_eq!(offer.currency, iou.currency);

            let new_offer_ids = shop
                .offer_ids
                .clone()
                .into_iter()
                .filter(|shop_offer_id| *shop_offer_id != offer_id)
                .collect::<Vec<_>>();
            assert_eq!(new_offer_ids.len(), shop.offer_ids.len() - 1);

            let new_shop = sign! { Shop =>
                offer_ids: new_offer_ids,
                ..shop
            };
            let vendor = offer.vendor;
            let new_item = offer_id.settle(self.user);
            let new_iou = iou_id.transfer_iou(vendor);
            (new_shop, new_item, new_iou)
        }
    }
}
