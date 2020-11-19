use liquid_lang as liquid;

/// This example comes from https://github.com/digital-asset/ex-models/tree/master/auction
///
/// # Workflow
/// 1. There is a seller party, and two or more bidders.
/// 2. The seller generates an `Auction` contract, only visible to her, and then embeds it into `AuctionInvitations` for each individual participant.
/// 3. Each bidder responds to the invitation with a `Bid`, visible only to him and the seller.
/// 4. When the auction finishes at `Auction.end` time, an off-ledger process collects all the bids, calculates the resulting allocations as an `AuctionResult`.
#[liquid::collaboration]
mod auction {
    #[derive(Clone)]
    struct Allocation {
        party: address,
        price: u32,
        quantity: u32,
    }

    /// Used to place a bid on an active auction.
    #[liquid(definitions)]
    struct Bid {
        auction: Auction,
        #[liquid(signers = "$.party")]
        allocation: Allocation,
    }

    /// Created on successful completion of an auction, to display allocs.
    #[liquid(definitions)]
    struct AuctionResult {
        #[liquid(signers = "$.seller")]
        auction: Auction,
        allocations: Vec<Allocation>,
    }

    /// Private to each buyer; contains a copy of the main `Auction` contract.
    #[derive(Clone)]
    #[liquid(definitions)]
    struct AuctionInvitation {
        #[liquid(signers = "$.seller")]
        auction: Auction,
        buyer: address,
    }

    #[liquid(rights)]
    impl AuctionInvitation {
        #[liquid(belongs_to = "buyer")]
        pub fn submit_bid(self, price: u32, quantity: u32) -> Bid {
            let now = self.env().now();
            assert!(now < self.auction.end && now >= self.auction.start);

            create! { Bid =>
                allocation: Allocation {
                    party: self.buyer,
                    price,
                    quantity,
                },
                ..self
            }
        }
    }

    /// Given the remaining quantity and requested allocation amount,
    /// calculate the resulting quantity and allocation.
    fn alloc_bids(rem_qty: u32, mut allocation: Allocation) -> (u32, Allocation) {
        if rem_qty == 0 || rem_qty < allocation.quantity {
            allocation.quantity = rem_qty;
            return (0, allocation);
        }

        (rem_qty - allocation.quantity, allocation)
    }

    #[derive(Clone)]
    #[liquid(definitions)]
    struct Auction {
        security: String,
        quantity: u32,
        #[liquid(signers)]
        seller: address,
        start: timestamp,
        end: timestamp,
    }

    #[liquid(rights_belong_to = "seller")]
    impl Auction {
        /// Sent individually to each participant (bidder) at start of auction.
        pub fn invite_bidder(&self, buyer: address) -> AuctionInvitation {
            create! { AuctionInvitation =>
                buyer,
                auction: self.clone(),
            }
        }

        /// This should be triggered externally at the auction end time.
        /// Collect the bids and publish allocations in `AuctionResult`.
        pub fn complete_auction(self, bid_ids: Vec<Bid>) -> AuctionResult {
            let bids = bid_ids.iter().map(|bid_id| fetch!(bid_id));
            let mut request_allocs =
                bids.map(|bid| bid.allocation.clone()).collect::<Vec<_>>();
            request_allocs.sort_by(|x, y| y.price.partial_cmp(&x.price).unwrap());
            let final_allocs = request_allocs
                .iter()
                .map(|allocation| alloc_bids(self.quantity, allocation.clone()))
                .fold((0, Vec::new()), |mut acc, item| {
                    (acc.0 + item.0, {
                        acc.1.push(item.1);
                        acc.1
                    })
                });

            create! { AuctionResult =>
                auction: self.clone(),
                allocations: final_allocs.1,
            }
        }
    }
}

fn main() {}