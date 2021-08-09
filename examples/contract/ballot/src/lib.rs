#![cfg_attr(not(feature = "std"), no_std)]

use liquid::{storage, InOut};
use liquid_lang as liquid;

/// This example is inspired by [Solidity by Example](https://solidity.readthedocs.io/en/latest/solidity-by-example.html).
/// Voting with delegation.
#[liquid::contract]
mod ballot {
    use super::*;

    /// This declares a new complex type which will
    /// be used for variables later.
    /// It will represent a single voter.
    #[derive(InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub struct Voter {
        /// weight is accumulated by delegation
        weight: u32,
        /// if true, that person already voted
        voted: bool,
        /// person delegated to
        delegate: address,
        /// index of the voted proposal
        vote: u32,
    }

    /// This is a type for a single proposal.
    #[derive(InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub struct Proposal {
        /// name of the proposal
        name: String,
        /// number of accumulated votes
        vote_count: u32,
    }

    #[liquid(storage)]
    struct Ballot {
        pub chairperson: storage::Value<address>,
        /// This declares a state variable that
        /// stores a `Voter` struct for each possible address.
        pub voters: storage::Mapping<address, Voter>,
        /// A dynamically-sized array of `Proposal` structs.
        pub proposals: storage::Vec<Proposal>,
    }

    #[liquid(methods)]
    impl Ballot {
        /// Create a new ballot to choose one of `proposalNames`.
        pub fn new(&mut self, proposal_names: Vec<String>) {
            let chairperson = self.env().get_caller();
            self.chairperson.initialize(chairperson);

            self.voters.initialize();
            self.voters.insert(
                chairperson,
                Voter {
                    weight: 1,
                    voted: false,
                    delegate: address::empty(),
                    vote: 0,
                },
            );

            // For each of the provided proposal names,
            // create a new proposal object and add it
            // to the end of the array.
            self.proposals.initialize();
            for name in proposal_names {
                // `Proposal{...}` creates a temporary
                // Proposal object and `self.proposals.push(...)`
                // appends it to the end of `self.proposals`.
                self.proposals.push(Proposal {
                    name,
                    vote_count: 0,
                });
            }
        }

        /// Give `voter` the right to vote on this ballot.
        /// May only be called by `chairperson`.
        pub fn give_right_to_vote(&mut self, voter: address) {
            // If the first argument of `require` evaluates
            // to `false`, execution terminates.
            // It is often a good idea to use `require` to check if
            // functions are called correctly.
            // As a second argument, you can also provide an
            // explanation about what went wrong.
            require(
                self.env().get_caller() == *self.chairperson,
                "Only chairperson can give right to vote.",
            );

            if let Some(voter) = self.voters.get_mut(&voter) {
                require(!voter.voted, "The voter already voted.");
                require(voter.weight == 0, "The weight of voter is not zero.");
                voter.weight = 1;
            } else {
                self.voters.insert(
                    voter,
                    Voter {
                        weight: 1,
                        voted: false,
                        delegate: address::empty(),
                        vote: 0,
                    },
                );
            }
        }

        /// Delegate your vote to the voter `to`.
        pub fn delegate(&mut self, mut to: address) {
            require(
                to != self.env().get_caller(),
                "Self-delegation is disallowed.",
            );
            require(
                self.voters.contains_key(&to),
                "Can not delegate to an inexistent voter.",
            );

            // assigns reference
            let caller = &self.env().get_caller();
            let sender = &self.voters[caller];
            require(!sender.voted, "You already voted.");

            // Forward the delegation as long as
            // `to` also delegated.
            // In general, such loops are very dangerous,
            // because if they run too long, they might
            // need more gas than is available in a block.
            // In this case, the delegation will not be executed,
            // but in other situations, such loops might
            // cause a contract to get "stuck" completely.
            while self.voters[&to].delegate != address::empty() {
                to = self.voters[&to].delegate;

                // We found a loop in the delegation, not allowed.
                require(to != self.env().get_caller(), "Found loop in delegation.");
            }

            // Since `sender` is a reference, this
            // modifies `self.voters`
            let sender = &mut self.voters[caller];
            sender.voted = true;
            sender.delegate = to;

            let weight = sender.weight;
            let delegate_ = &mut self.voters[&to];
            if delegate_.voted {
                // If the delegate already voted,
                // directly add to the number of votes
                self.proposals[delegate_.vote].vote_count += weight;
            } else {
                // If the delegate did not vote yet,
                // add to her weight.
                delegate_.weight += weight;
            }
        }

        /// Give your vote (including votes delegated to you)
        /// to proposal `self.proposals[proposal].name`.
        pub fn vote(&mut self, proposal: u32) {
            let caller = self.env().get_caller();
            let sender = &mut self.voters[&caller];
            require(sender.weight != 0, "Has no right to vote");
            require(!sender.voted, "Already voted.");
            sender.voted = true;
            sender.vote = proposal;

            // If `proposal` is out of the range of the array,
            // this will throw automatically and revert all
            // changes.
            self.proposals[proposal].vote_count += sender.weight;
        }

        /// Computes the winning proposal taking all
        /// previous votes into account.
        pub fn winning_proposal(&self) -> u32 {
            let mut winning_vote_count = 0;
            let mut winning_proposal = 0;

            for p in 0..self.proposals.len() {
                if self.proposals[p].vote_count > winning_vote_count {
                    winning_vote_count = self.proposals[p].vote_count;
                    winning_proposal = p;
                }
            }

            winning_proposal
        }

        /// Calls winningProposal() function to get the index
        /// of the winner contained in the proposals array and then
        /// returns the name of the winner.
        pub fn winner_name(&self) -> String {
            self.proposals[self.winning_proposal()].name.clone()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid::env::test;

        fn deploy_contract() -> Ballot {
            let accounts = test::default_accounts();
            test::set_caller(accounts.alice);

            let proposal_names = vec![
                "play with cat".to_string(),
                "eat".to_string(),
                "sleep".to_string(),
            ];
            Ballot::new(proposal_names)
        }

        #[test]
        fn constructor_works() {
            let accounts = test::default_accounts();
            let ballot = deploy_contract();
            let alice = accounts.alice;
            assert_eq!(*ballot.chairperson, alice);
            assert_eq!(ballot.voters.len(), 1);

            let voter = &ballot.voters[&alice];
            assert_eq!(voter.weight, 1);
            assert_eq!(voter.voted, false);
            assert_eq!(voter.delegate, address::empty());
            assert_eq!(voter.vote, 0);

            assert_eq!(ballot.proposals.len(), 3);
            assert_eq!(ballot.proposals[0].name, "play with cat");
            assert_eq!(ballot.proposals[0].vote_count, 0);
            assert_eq!(ballot.proposals[1].name, "eat");
            assert_eq!(ballot.proposals[1].vote_count, 0);
            assert_eq!(ballot.proposals[2].name, "sleep");
            assert_eq!(ballot.proposals[2].vote_count, 0);
        }

        #[test]
        #[should_panic(expected = "Only chairperson can give right to vote.")]
        fn no_right_to_give_vote_right() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();

            // Another account who wants to distribute right to vote
            test::set_caller(accounts.bob);
            ballot.give_right_to_vote(accounts.charlie);
        }

        #[test]
        #[should_panic(expected = "The voter already voted.")]
        fn voted_voter() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let voter = accounts.bob;

            ballot.give_right_to_vote(voter);
            test::set_caller(voter);
            ballot.vote(0);
            test::pop_execution_context();
            ballot.give_right_to_vote(voter);
        }

        #[test]
        #[should_panic(expected = "The weight of voter is not zero.")]
        fn voter_has_weight() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let voter = accounts.bob;

            ballot.give_right_to_vote(voter);
            test::set_caller(voter);
            test::pop_execution_context();
            ballot.give_right_to_vote(voter);
        }

        #[test]
        fn give_right_works() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let voter = accounts.bob;

            ballot.give_right_to_vote(voter);
            assert_eq!(ballot.voters.len(), 2);
            assert_eq!(ballot.voters.contains_key(&voter), true);
            assert_eq!(
                ballot.voters[&voter],
                Voter {
                    weight: 1,
                    voted: false,
                    delegate: address::empty(),
                    vote: 0,
                }
            );
        }

        #[test]
        fn delegate_works_1() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            let charlie = accounts.charlie;
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);

            test::set_caller(bob);
            ballot.delegate(charlie);
            assert_eq!(ballot.voters[&bob].delegate, charlie);
            assert_eq!(ballot.voters[&bob].voted, true);
            assert_eq!(ballot.voters[&charlie].weight, 2);
        }

        #[test]
        fn delegate_works_2() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            let charlie = accounts.charlie;
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);
            test::set_caller(charlie);
            ballot.vote(0);
            test::pop_execution_context();
            assert_eq!(ballot.proposals[0].vote_count, 1);

            test::set_caller(bob);
            ballot.delegate(charlie);
            assert_eq!(ballot.voters[&bob].delegate, charlie);
            assert_eq!(ballot.voters[&bob].voted, true);
            assert_eq!(ballot.voters[&bob].weight, 1);
            assert_eq!(ballot.proposals[0].vote_count, 2);
        }

        #[test]
        #[should_panic(expected = "Self-delegation is disallowed.")]
        fn delegate_to_self() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            ballot.give_right_to_vote(bob);

            test::set_caller(bob);
            ballot.delegate(bob);
        }

        #[test]
        #[should_panic(expected = "Can not delegate to an inexistent voter.")]
        fn delegate_to_inexistent_account() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            ballot.give_right_to_vote(bob);

            test::set_caller(bob);
            let charlie = accounts.charlie;
            ballot.delegate(charlie);
        }

        #[test]
        #[should_panic(expected = "You already voted.")]
        fn delegate_after_voted() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            let charlie = accounts.charlie;
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);

            test::set_caller(bob);
            ballot.vote(0);
            ballot.delegate(charlie);
        }

        #[test]
        #[should_panic(expected = "Found loop in delegation.")]
        fn delegate_loop() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            let charlie = accounts.charlie;
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);

            test::set_caller(charlie);
            ballot.delegate(bob);
            test::pop_execution_context();

            test::set_caller(bob);
            ballot.delegate(charlie);
        }

        #[test]
        fn vote_works() {
            let accounts = test::default_accounts();
            let mut ballot = deploy_contract();
            let bob = accounts.bob;
            let charlie = accounts.charlie;
            let david = accounts.david;
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);
            ballot.give_right_to_vote(david);

            test::set_caller(bob);
            ballot.vote(0);
            test::pop_execution_context();

            test::set_caller(charlie);
            ballot.vote(0);
            test::pop_execution_context();

            test::set_caller(david);
            ballot.vote(1);
            test::pop_execution_context();

            assert_eq!(ballot.winning_proposal(), 0);
            assert_eq!(ballot.winner_name(), "play with cat");
        }
    }
}
