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
        pub fn add(mut self, voter_addr: address) -> ContractId<Ballot> {
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
            let voters = self.voters.iter().map(|voter| voter.addr).collect();
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

            test::push_execution_context(government);
            let ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.take().add(bob);
            let ballot_id = ballot_id.take().add(charlie);
            let mut ballot_id = ballot_id.take().add(david);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(charlie);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(david);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(government);
            let decision_id = ballot_id.exec(|ballot| ballot.decide());
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

            test::push_execution_context(government);
            let ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("let's 996"),
                },
            };
            let mut ballot_id = ballot_id.take().add(bob);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(government);
            let ballot_id = ballot_id.exec(|ballot| ballot.add(charlie));
            let mut ballot_id = ballot_id.exec(|ballot| ballot.add(david));
            test::pop_execution_context();

            test::push_execution_context(charlie);
            ballot_id.as_mut().vote(false);
            test::pop_execution_context();

            test::push_execution_context(david);
            ballot_id.as_mut().vote(false);
            test::pop_execution_context();

            test::push_execution_context(government);
            let decision_id = ballot_id.exec(|ballot| ballot.decide());
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

            test::push_execution_context(government);
            let ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("let's 996"),
                },
            };
            let mut ballot_id = ballot_id.take().add(bob);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(false);
            test::pop_execution_context();
        }

        #[test]
        #[should_panic]
        fn unauthorized_signing() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::push_execution_context(government);
            sign! { Ballot =>
                government: bob,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("let's 996"),
                },
            };
        }

        #[test]
        #[should_panic(expected = "DO NOT excise right on an inexistent `Ballot`contract")]
        fn exercise_on_a_temporary_contract() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;

            test::push_execution_context(government);
            let ballot = Ballot {
                government: address::empty(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            ballot.add(government);
        }

        #[test]
        #[should_panic(expected = "DO NOT excise right on an inexistent `Ballot`contract")]
        fn exercise_on_fetch() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;

            test::push_execution_context(government);
            let ballot_id = sign! {Ballot =>
                government: address::empty(),
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            ballot_id.fetch().add(government);
        }

        #[test]
        #[should_panic(
            expected = "exercising right `decide` of contract `Ballot` is not permitted"
        )]
        fn unauthorized_exercising() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::push_execution_context(government);
            let ballot_id = sign! {Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.exec(|ballot| ballot.decide());
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "cannot decide on tie")]
        fn decide_on_tie() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;

            test::push_execution_context(government);
            let ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.take().add(bob);
            let mut ballot_id = ballot_id.take().add(charlie);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(charlie);
            ballot_id.as_mut().vote(false);
            test::pop_execution_context();

            test::push_execution_context(government);
            ballot_id.exec(|ballot| ballot.decide());
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "all voters must vote")]
        fn not_all_voters_voted() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;
            let charlie = default_accounts.charlie;

            test::push_execution_context(government);
            let ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            let ballot_id = ballot_id.take().add(bob);
            let mut ballot_id = ballot_id.take().add(charlie);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();

            test::push_execution_context(government);
            ballot_id.exec(|ballot| ballot.decide());
            test::pop_execution_context();
        }

        #[test]
        #[should_panic(expected = "voter not added")]
        fn voter_not_added() {
            let default_accounts = test::default_accounts();
            let government = default_accounts.alice;
            let bob = default_accounts.bob;

            test::push_execution_context(government);
            let mut ballot_id = sign! { Ballot =>
                government,
                voters: Vec::new(),
                proposal: Proposal {
                    proposer: government,
                    content: String::from("take a holiday"),
                },
            };
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot_id.as_mut().vote(true);
            test::pop_execution_context();
        }
    }
}
