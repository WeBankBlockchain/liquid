// https://github.com/digital-asset/ex-models/tree/master/voting
//
// This example models a voting process controlled by a government entity.
// The government puts text-based proposals to a vote by creating a ballot.
// When the vote is decided a contract evidences the outcome on the ledger.
//
// # Workflow
//
// 1. The government creates a `Ballot` for a given `Proposal`.
// 2. It invites voters via `add`.
// 3. Voters can cast a `vote` for a given Ballot.
// 4. Once all voters have voted the government can `decide` the vote.
// 5. The outcome of the ballot is recorded in a `Decision` contract on the ledger.

#![cfg_attr(not(feature = "std"), no_std)]

use liquid::InOut;
use liquid_lang as liquid;

#[liquid::collaboration]
mod voting {
    use super::*;

    #[derive(InOut)]
    pub struct Proposal {
        proposer: address,
        content: String,
    }

    #[liquid(contract)]
    pub struct Decision {
        #[liquid(signers)]
        government: address,
        proposal: Proposal,
        #[liquid(signers)]
        voters: Vec<address>,
        accept: bool,
    }

    #[derive(InOut)]
    pub struct Voter {
        addr: address,
        voted: bool,
        choice: bool,
    }

    #[liquid(contract)]
    pub struct Ballot {
        #[liquid(signers)]
        government: address,
        #[liquid(signers = "$[..](?@.voted).addr")]
        voters: Vec<Voter>,
        proposal: Proposal,
    }

    #[liquid(rights_belong_to = "government")]
    impl Ballot {
        pub fn add(mut self, voter_addr: address) {
            assert!(self
                .voters
                .iter()
                .find(|voter| voter.addr == voter_addr)
                .is_none());

            let voter = Voter {
                addr: voter_addr,
                voted: false,
                choice: false,
            };
            self.voters.push(voter);

            create! { Self =>
                voters: self.voters,
                ..self
            };
        }

        pub fn decide(self) -> ContractId<Decision> {
            require(
                self.voters.iter().all(|voter| voter.voted),
                "all voters must vote",
            );

            let yays = self.voters.iter().filter(|v| v.choice).count();
            let nays = self.voters.iter().filter(|v| !v.choice).count();
            require(yays != nays, "cannot decide on tie");

            let accept = yays > nays;
            let voters = self.voters.iter().map(|voter| voter.addr).collect();
            create! { Decision =>
                accept,
                government: self.government,
                proposal: self.proposal,
                voters,
            }
        }
    }

    #[liquid(rights)]
    impl Ballot {
        #[liquid(belongs_to = "")]
        pub fn vote(&mut self, choice: bool) {
            let voter_addr = self.env().get_caller();

            let voter = self
                .voters
                .iter_mut()
                .find(|voter| voter.addr == voter_addr);
            require(voter.is_some(), "voter not added");

            let voter = voter.unwrap();
            require(!voter.voted, "voter already voted");

            voter.voted = true;
            voter.choice = choice;
        }
    }
}
