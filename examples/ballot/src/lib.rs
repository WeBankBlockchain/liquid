#![cfg_attr(not(feature = "std"), no_std)]

use liquid_lang as liquid;

/// This example is inspired by [Solidity by Example](https://solidity.readthedocs.io/en/v0.5.3/solidity-by-example.html).
/// Voting with delegation.
#[liquid::contract(version = "0.1.0")]
mod ballot {
    use liquid_lang::{InOut, State};

    /// This declares a new complex type which will
    /// be used for variables later.
    /// It will represent a single voter.
    #[derive(State, InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub struct Voter {
        /// weight is accumulated by delegation
        weight: u32,
        /// if true, that person already voted
        voted: bool,
        /// person delegated to
        delegate: Address,
        /// index of the voted proposal
        vote: u32,
    }

    /// This is a type for a single proposal.
    #[derive(State, InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub struct Proposal {
        /// name of the proposal
        name: String,
        /// number of accumulated votes
        vote_count: u32,
    }

    use liquid_core::storage;

    #[liquid(storage)]
    struct Ballot {
        pub chair_person: storage::Value<Address>,
        /// This declares a state variable that
        /// stores a `Voter` struct for each possible address.
        pub voters: storage::Mapping<Address, Voter>,
        /// A dynamically-sized array of `Proposal` structs.
        pub proposals: storage::Vec<Proposal>,
    }

    #[liquid(methods)]
    impl Ballot {
        /// Create a new ballot to choose one of `proposalNames`.
        pub fn new(&mut self, proposal_names: liquid_core::env::types::Vec<String>) {
            let chair_person = self.env().get_caller();
            self.chair_person.initialize(chair_person);

            self.voters.initialize();
            self.voters.insert(
                &chair_person,
                Voter {
                    weight: 1,
                    voted: false,
                    delegate: Address::empty(),
                    vote: 0,
                },
            );

            // For each of the provided proposal names,
            // create a new proposal object and add it
            // to the end of the array.
            self.proposals.initialize();
            for name in proposal_names {
                // `Proposal({...})` creates a temporary
                // Proposal object and `proposals.push(...)`
                // appends it to the end of `proposals`.
                self.proposals.push(Proposal {
                    name,
                    vote_count: 0,
                });
            }
        }

        /// Give `voter` the right to vote on this ballot.
        /// May only be called by `chairperson`.
        pub fn give_right_to_vote(&mut self, voter: Address) {
            // If the first argument of `require` evaluates
            // to `false`, execution terminates and all
            // changes to the state and to Ether balances
            // are reverted.
            // This used to consume all gas in old EVM versions, but
            // not anymore.
            // It is often a good idea to use `require` to check if
            // functions are called correctly.
            // As a second argument, you can also provide an
            // explanation about what went wrong.
            require(
                self.env().get_caller() == *self.chair_person,
                "Only chairperson can give right to vote.",
            );

            if let Some(voter) = self.voters.get_mut(&voter) {
                require(!voter.voted, "The voter already voted.");
                require(voter.weight == 0, "The weight of voter is not zero.");
                voter.weight = 1;
            } else {
                self.voters.insert(
                    &voter,
                    Voter {
                        weight: 1,
                        voted: false,
                        delegate: Address::empty(),
                        vote: 0,
                    },
                );
            }
        }

        /// Delegate your vote to the voter `to`.
        pub fn delegate(&mut self, mut to: Address) {
            require(
                to != self.env().get_caller(),
                "Self-delegation is disallowed.",
            );
            require(
                self.voters.contains_key(&to),
                "Can not delegate to an inexistent voter",
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
            while self.voters[&to].delegate != Address::empty() {
                to = self.voters[&to].delegate;

                // We found a loop in the delegation, not allowed.
                require(to != self.env().get_caller(), "Found loop in delegation.");
            }

            // Since `sender` is a reference, this
            // modifies `self.voters[msg.sender].voted`
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
        /// to proposal `proposals[proposal].name`.
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
        use liquid_core::env::test;

        fn deploy_contract() -> Ballot {
            // The address of chairman is "0x000...000".
            test::push_execution_context(Address::from_bytes(&[0u8; 20]));

            let proposal_names = vec![
                "play with cat".to_string(),
                "eat".to_string(),
                "sleep".to_string(),
            ];
            Ballot::new(proposal_names)
        }

        #[test]
        fn constructor_works() {
            let ballot = deploy_contract();
            let chair_person = Address::from_bytes(&[0u8; 20]);
            assert_eq!(*ballot.chair_person, chair_person);
            assert_eq!(ballot.voters.len(), 1);

            let voter = &ballot.voters[&chair_person];
            assert_eq!(voter.weight, 1);
            assert_eq!(voter.voted, false);
            assert_eq!(voter.delegate, Address::empty());
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
        #[should_panic]
        fn no_right_to_give_vote_right() {
            let mut ballot = deploy_contract();

            // Another account who wants to distribute right to vote
            test::push_execution_context(Address::from_bytes(&[1u8; 20]));
            ballot.give_right_to_vote(Address::from_bytes(&[2u8; 20]));
        }

        #[test]
        #[should_panic]
        fn voted_voter() {
            let mut ballot = deploy_contract();
            let voter = Address::from_bytes(&[1u8; 20]);

            ballot.give_right_to_vote(voter);
            test::push_execution_context(voter);
            ballot.vote(0);
            test::pop_execution_context();
            ballot.give_right_to_vote(voter);
        }

        #[test]
        #[should_panic]
        fn voter_has_weight() {
            let mut ballot = deploy_contract();
            let voter = Address::from_bytes(&[1u8; 20]);

            ballot.give_right_to_vote(voter);
            test::push_execution_context(voter);
            test::pop_execution_context();
            ballot.give_right_to_vote(voter);
        }

        #[test]
        fn give_right_works() {
            let mut ballot = deploy_contract();
            let voter = Address::from_bytes(&[1u8; 20]);

            ballot.give_right_to_vote(voter);
            assert_eq!(ballot.voters.len(), 2);
            assert_eq!(ballot.voters.contains_key(&voter), true);
            assert_eq!(
                ballot.voters[&voter],
                Voter {
                    weight: 1,
                    voted: false,
                    delegate: Address::empty(),
                    vote: 0,
                }
            );
        }

        #[test]
        fn delegate_works_1() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            let bob = Address::from_bytes(&[2u8; 20]);
            ballot.give_right_to_vote(alice);
            ballot.give_right_to_vote(bob);

            test::push_execution_context(alice);
            ballot.delegate(bob);
            assert_eq!(ballot.voters[&alice].delegate, bob);
            assert_eq!(ballot.voters[&alice].voted, true);
            assert_eq!(ballot.voters[&bob].weight, 2);
        }

        #[test]
        fn delegate_works_2() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            let bob = Address::from_bytes(&[2u8; 20]);
            ballot.give_right_to_vote(alice);
            ballot.give_right_to_vote(bob);
            test::push_execution_context(bob);
            ballot.vote(0);
            test::pop_execution_context();
            assert_eq!(ballot.proposals[0].vote_count, 1);

            test::push_execution_context(alice);
            ballot.delegate(bob);
            assert_eq!(ballot.voters[&alice].delegate, bob);
            assert_eq!(ballot.voters[&alice].voted, true);
            assert_eq!(ballot.voters[&bob].weight, 1);
            assert_eq!(ballot.proposals[0].vote_count, 2);
        }

        #[test]
        #[should_panic]
        fn delegate_to_self() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            ballot.give_right_to_vote(alice);

            test::push_execution_context(alice);
            ballot.delegate(alice);
        }

        #[test]
        #[should_panic]
        fn delegate_to_inexistent_account() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            ballot.give_right_to_vote(alice);

            test::push_execution_context(alice);
            let bob = Address::from_bytes(&[2u8; 20]);
            ballot.delegate(bob);
        }

        #[test]
        #[should_panic]
        fn delegate_after_voted() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            let bob = Address::from_bytes(&[2u8; 20]);
            ballot.give_right_to_vote(alice);
            ballot.give_right_to_vote(bob);

            test::push_execution_context(alice);
            ballot.vote(0);
            ballot.delegate(bob);
        }

        #[test]
        #[should_panic]
        fn delegate_loop() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            let bob = Address::from_bytes(&[2u8; 20]);
            ballot.give_right_to_vote(alice);
            ballot.give_right_to_vote(bob);

            test::push_execution_context(bob);
            ballot.delegate(alice);
            test::pop_execution_context();

            test::push_execution_context(alice);
            ballot.delegate(bob);
        }

        #[test]
        fn vote_works() {
            let mut ballot = deploy_contract();
            let alice = Address::from_bytes(&[1u8; 20]);
            let bob = Address::from_bytes(&[2u8; 20]);
            let charlie = Address::from_bytes(&[3u8; 20]);
            ballot.give_right_to_vote(alice);
            ballot.give_right_to_vote(bob);
            ballot.give_right_to_vote(charlie);

            test::push_execution_context(alice);
            ballot.vote(0);
            test::pop_execution_context();

            test::push_execution_context(bob);
            ballot.vote(0);
            test::pop_execution_context();

            test::push_execution_context(charlie);
            ballot.vote(1);
            test::pop_execution_context();

            assert_eq!(ballot.winning_proposal(), 0);
            assert_eq!(ballot.winner_name(), "play with cat");
        }
    }
}
