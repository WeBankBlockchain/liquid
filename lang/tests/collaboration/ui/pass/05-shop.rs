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

use liquid_lang as liquid;

#[liquid::collaboration]
mod shop {
    #[liquid(contract)]
    struct Iou {
        #[liquid(signers)]
        issuer: address,
        owner: address,
        amount: u64,
        currency: String,
    }

    #[liquid(rights)]
    impl Iou {
        #[liquid(belongs_to = "owner")]
        pub fn transfer_iou(self, new_owner: address) -> Iou {
            create! { Self =>
                owner: new_owner,
                ..self
            }
        }
    }

    #[liquid(contract)]
    struct Item {
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
        pub fn transfer_item(self, new_owner: address) -> Item {
            create! { Self =>
                owner: new_owner,
                ..self
            }
        }

        pub fn disclose(self, users: Vec<address>) -> Item {
            create! { Self =>
                observers: users,
                ..self
            }
        }
    }

    #[liquid(contract)]
    struct Offer {
        #[liquid(signers)]
        owner: address,
        #[liquid(signers)]
        vendor: address,
        item: Item,
        price: u64,
        currency: String,
        users: Vec<address>,
    }

    #[liquid(rights)]
    impl Offer {
        #[liquid(belongs_to = "owner")]
        pub fn settle(self, buyer: address) -> Item {
            self.item.transfer_item(buyer)
        }
    }

    #[liquid(contract)]
    struct Shop {
        #[liquid(signers)]
        owner: address,
        vendors: Vec<address>,
        users: Vec<address>,
        offers: Vec<address>,
    }

    #[liquid(rights_belong_to = "owner")]
    impl Shop {
        pub fn invite_vendor(mut self, vendor: address) -> (Shop, VendorInvite) {
            self.vendors.push(vendor);
            let shop = create! { Self =>
                ..self
            };
            let invite = create! { VendorInvite =>
                vendor,
                ..self
            };
            (shop, invite)
        }

        pub fn invite_user(mut self, user: address) -> (Shop, UserInvite) {
            self.users.push(user);
            let shop = create! { Self =>
                ..self
            };
            let invite = create! { UserInvite =>
                user,
                ..self
            };
            (shop, invite)
        }
    }

    #[liquid(contract)]
    struct VendorInvite {
        #[liquid(signers)]
        owner: address,
        vendor: address,
    }

    #[liquid(rights)]
    impl VendorInvite {
        #[liquid(belongs_to = "vendor")]
        pub fn accept_vendor_invite(self) -> VendorRelationship {
            create! { VendorRelationship =>
                ..self
            }
        }
    }

    #[liquid(contract)]
    struct VendorRelationship {
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
            shop: Shop,
            item: Item,
            price: u64,
            currency: String,
        ) -> (Shop, Offer) {
            let mut users = Vec::new();
            users.push(self.owner);
            users.extend(shop.users);
            let disclosed_item = item.disclose(users);

            let offer = create! { Offer =>
                item: disclosed_item,
                users: shop.users,
                price,
                currency,
                ..self
            };

            let mut offers = Vec::new();
            offers.push(offer);
            offers.extend(shop.offers);
            let shop = create! { Shop =>
                offers,
                ..shop
            };

            (shop, offer)
        }
    }

    #[liquid(contract)]
    struct UserInvite {
        #[liquid(signers)]
        owner: address,
        user: address,
    }

    #[liquid(rights)]
    impl UserInvite {
        #[liquid(belongs_to = "user")]
        pub fn accept_user_invite(self) -> UserRelationship {
            create! { UserRelationship =>
                ..self
            }
        }
    }

    #[liquid(contract)]
    struct UserRelationship {
        #[liquid(signers)]
        owner: address,
        #[liquid(signers)]
        user: address,
    }

    #[liquid(rights)]
    impl UserRelationship {
        #[liquid(belongs_to = "user")]
        pub fn buy_item(&self, shop: Shop, offer: Offer, iou: Iou) -> (Shop, Item, Iou) {
            assert_eq!(offer.price == iou.amount);
            assert_eq!(offer.currency == iou.currency);

            let new_offers = shop
                .offers
                .iter()
                .filter(|shop_offer| shop_offer != offer)
                .collect::<Vec<_>>();
            assert!(new_offers.len() == shop.offers.len() - 1);

            let new_shop = create! { Shop =>
                offers: new_offers,
                ..shop
            };
            let new_item = offer.settle(self.user);
            let new_iou = iou.transfer_iou(offer.vendor);
            (new_shop, new_item, new_iou)
        }
    }
}

fn main() {}
