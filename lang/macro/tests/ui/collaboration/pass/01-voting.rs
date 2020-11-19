use liquid_lang as liquid;

/// This example comes from https://github.com/digital-asset/ex-models/tree/master/voting
/// 
/// # Workflow
///
/// 1. The government creates a `Ballot` for a given `Proposal`.
/// 2. It invites voters via `add`.
/// 3. Voters can cast a `vote` for a given Ballot.
/// 4. Once all voters have voted the government can `decide` the vote.
/// 5. The outcome of the ballot is recorded in a `Decision` contract on the ledger.
#[liquid::collaboration]
mod voting {
    struct Proposal {
        proposer: address,
        content: String,
    }

    #[liquid(definitions)]
    struct Decision {
        #[liquid(signers)]
        government: address,
        #[liquid(signers)]
        voters: Vec<address>,
        proposal: Proposal,
        accept: bool,
    }

    struct Voter {
        addr: address,
        voted: bool,
        choice: bool,
    }

    #[liquid(definitions)]
    struct Ballot {
        #[liquid(signers)]
        government: address,
        #[liquid(signers = "$[*(?@.voted)].addr")]
        voters: Vec<Voter>,
        proposal: Proposal,
    }

    #[liquid(rights_belong_to = "government")]
    impl Ballot {
        pub fn add(mut self, voter_addr: address) {
            let voter = Voter {
                addr: voter_addr,
                voted: false,
                choice: false,
            };
            self.voters.push(voter);

            create! { Self =>
                voters: self.voters,
                ..self
            }
        }

        pub fn decide(self) -> Decision {
            require(
                self.voters.iter().all(|voter| voter.voted),
                "all voters must vote",
            );

            let yays = self.voters.iter().filter(|v| v.choice).count();
            let nays = self.voters.iter().filter(|v| !v.choice).count();
            require(yays != nays, "cannot decide on tie");

            let accept = yays > nays;
            create! { Decision =>
                accept,
                ..self
            }
        }
    }

    #[liquid(rights)]
    impl Ballot {
        #[liquid(belongs_to = "")]
        pub fn vote(&mut self, choice: bool) {
            let voter_addr = env::get_caller();

            let voter = self.voters.iter_mut().find(|voter| voter.addr == voter_addr);
            require(voter.is_some(), "voter not added");

            let voter = voter.unwrap();
            require(!voter.voted, "voter already voted");

            voter.voted = true;
            voter.choice = choice;
        }
    }
}

fn main() {}
