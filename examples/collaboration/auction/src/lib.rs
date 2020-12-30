// https://github.com/digital-asset/ex-models/tree/master/auction
//
// An auction model with bids.
//
// # Workflow
// 1. There is a seller party, and two or more bidders.
// 2. The `seller` generates an `Auction` contract, only visible to her, and then embeds it into `AuctionInvitations` for each individual participant.
// 3. Each bidder responds to the invitation with a `Bid`.
// 4. When the auction finishes at `Auction.end` time, an off-ledger process collects all the bids, calculates the resulting allocations as an `AuctionResult`.

#![cfg_attr(not(feature = "std"), no_std)]

use liquid::InOut;
use liquid_lang as liquid;

#[liquid::collaboration]
mod auction {
    use super::*;

    /// Used both to indicate a bid, and to show the final allocations.
    #[derive(Clone, InOut)]
    pub struct Allocation {
        party: address,
        price: u64,
        quantity: u64,
    }

    /// Used to place a bid on an active auction.
    #[liquid(contract)]
    pub struct Bid {
        auction: Auction,
        #[liquid(signers = "$.party")]
        allocation: Allocation,
    }

    /// Created on successful completion of an auction, to display allocs.
    #[liquid(contract)]
    pub struct AuctionResult {
        #[liquid(signers = "$.seller")]
        auction: Auction,
        allocations: Vec<Allocation>,
    }

    /// Contains a copy of the main `Auction` contract.
    #[liquid(contract)]
    pub struct AuctionInvitation {
        #[liquid(signers = "$.seller")]
        auction: Auction,
        buyer: address,
    }

    #[liquid(rights)]
    impl AuctionInvitation {
        #[liquid(belongs_to = "buyer")]
        pub fn submit_bid(self, price: u64, quantity: u64) -> ContractId<Bid> {
            let now = self.env().now();
            assert!(now < self.auction.end && now >= self.auction.start);

            sign! { Bid =>
                allocation: Allocation {
                    party: self.buyer,
                    price,
                    quantity,
                },
                auction: self.auction,
            }
        }
    }

    #[liquid(contract)]
    #[derive(Clone)]
    pub struct Auction {
        security: String,
        quantity: u64,
        #[liquid(signers)]
        seller: address,
        start: timestamp,
        end: timestamp,
    }

    #[liquid(rights_belong_to = "seller")]
    impl Auction {
        /// Sent individually to each participant (bidder) at start of auction.
        pub fn invite_bidder(&self, buyer: address) -> ContractId<AuctionInvitation> {
            sign! { AuctionInvitation =>
                buyer,
                auction: self.as_ref().clone(),
            }
        }

        /// This should be triggered externally at the auction end time.
        /// Collect the bids and publish allocations in `AuctionResult`.
        pub fn complete_auction(self, bids: Vec<Bid>) -> ContractId<AuctionResult> {
            let mut request_allocs = bids
                .iter()
                .map(|bid| bid.allocation.clone())
                .collect::<Vec<_>>();
            request_allocs.sort_by(|x, y| y.price.partial_cmp(&x.price).unwrap());

            let mut quantity = self.quantity;
            let mut final_allocs = Vec::new();
            for request_alloc in request_allocs.into_iter() {
                let (rem_qty, allocation) = alloc_bids(quantity, request_alloc);
                quantity = rem_qty;
                final_allocs.push(allocation);
            }

            sign! { AuctionResult =>
                auction: self.as_ref().clone(),
                allocations: final_allocs,
            }
        }
    }

    /// Given the remaining quantity and requested allocation amount,
    /// calculate the resulting quantity and allocation.
    fn alloc_bids(rem_qty: u64, mut allocation: Allocation) -> (u64, Allocation) {
        if rem_qty == 0 || rem_qty < allocation.quantity {
            allocation.quantity = rem_qty;
            return (0, allocation);
        }

        (rem_qty - allocation.quantity, allocation)
    }
}
