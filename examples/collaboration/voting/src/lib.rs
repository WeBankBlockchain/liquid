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
        proposer: Address,
        content: String,
    }

    #[liquid(contract)]
    pub struct Decision {
        #[liquid(signers)]
        government: Address,
        proposal: Proposal,
        #[liquid(signers)]
        voters: Vec<Address>,
        accept: bool,
    }

    #[derive(InOut)]
    pub struct Voter {
        addr: Address,
        voted: bool,
        choice: bool,
    }

    #[liquid(contract)]
    pub struct Ballot {
        #[liquid(signers)]
        government: Address,
        #[liquid(signers = "$[..](?@.voted).addr")]
        voters: Vec<Voter>,
        proposal: Proposal,
    }

    #[liquid(rights_belong_to = "government")]
    impl Ballot {
        pub fn add(mut self, voter_addr: Address) -> ContractId<Ballot> {
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

            sign! { Ballot =>
                voters: self.voters,
                ..self
            }
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
            let voters = self.voters.iter().map(|voter| voter.addr.clone()).collect();
            sign! { Decision =>
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid::env::test;

        #[test]
        fn vote() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;
            let david = default_accounts.david;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.add(bob.clone());
            let ballot_id = ballot_id.add(charlie.clone());
            let ballot_id = ballot_id.add(david.clone());
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(charlie.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(david.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(government.clone());
            let decision_id = ballot_id.decide();
            let decision = decision_id.fetch();
            assert_eq!(decision.government, government);
            assert_eq!(decision.voters, vec![bob, charlie, david]);
            assert_eq!(decision.accept, true);
            let proposal = &decision.proposal;
            assert_eq!(proposal.proposer, government);
            assert_eq!(proposal.content, "take a holiday");
        }

        #[test]
        fn add_voter_after_voting() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;
            let david = default_accounts.david;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("let's 996"),
                },
            };
            let ballot_id = ballot_id.add(bob.clone());
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(government.clone());
            let ballot_id = ballot_id.add(charlie.clone());
            let ballot_id = ballot_id.add(david.clone());
            test::pop_execution_context();

            test::set_caller(charlie.clone());
            ballot_id.vote(false);
            test::pop_execution_context();

            test::set_caller(david.clone());
            ballot_id.vote(false);
            test::pop_execution_context();

            test::set_caller(government.clone());
            let decision_id = ballot_id.decide();
            let decision = decision_id.fetch();
            assert_eq!(decision.government, government);
            assert_eq!(decision.voters, vec![bob, charlie, david]);
            assert_eq!(decision.accept, false);
            let proposal = &decision.proposal;
            assert_eq!(proposal.proposer, government);
            assert_eq!(proposal.content, "let's 996");
        }

        #[test]
        #[should_panic(expected = "voter already voted")]
        fn duplicate_voting() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("let's 996"),
                },
            };
            let ballot_id = ballot_id.add(bob.clone());
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(false);
            test::pop_execution_context();
        }

        #[test]
        #[should_panic]
        fn unauthorized_signing() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(government.clone());
            sign! { Ballot =>
                government: bob.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("let's 996"),
                },
            };
        }

        #[test]
        #[should_panic(
            expected = "exercising right `decide` of contract `Ballot` is not permitted"
        )]
        fn unauthorized_exercising() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(government.clone());
            let ballot_id = sign! {Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("take a holiday"),
                },
            };
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.decide();
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "cannot decide on tie")]
        fn decide_on_tie() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.add(bob.clone());
            let ballot_id = ballot_id.add(charlie.clone());
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(charlie.clone());
            ballot_id.vote(false);
            test::pop_execution_context();

            test::set_caller(government.clone());
            ballot_id.decide();
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "all voters must vote")]
        fn not_all_voters_voted() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.add(bob.clone());
            let ballot_id = ballot_id.add(charlie.clone());
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();

            test::set_caller(government.clone());
            ballot_id.decide();
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "voter not added")]
        fn voter_not_added() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::set_caller(government.clone());
            let ballot_id = sign! { Ballot =>
                government: government.clone(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government.clone(),
                    content: String::from("take a holiday"),
                },
            };
            test::pop_execution_context();

            test::set_caller(bob.clone());
            ballot_id.vote(true);
            test::pop_execution_context();
        }
    }
}
