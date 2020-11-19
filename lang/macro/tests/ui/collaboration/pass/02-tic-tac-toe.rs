use liquid_lang as liquid;

/// This example comes from https://github.com/digital-asset/ex-models/tree/master/tic-tac-toe
///
/// ## Workflow
/// 1. A player invites another player by creating a `GameInvite`.
/// 2. Upon acceptance of the invite a `Game` contract is created.
/// 3. Player 1 starts the game by exercising the `play` choice, which recreates the `Game` contract with the move added.
/// 4. The players now take turns until the `play` choice detects a final state (win or tie).
/// 5. When the game is finished a `Result` contract is created containing the state and outcome of the game.
#[liquid::collaboration]
mod tic_tac_toe {
    struct Move {
        player: address,
        x: u8,
        y: u8,
    }

    enum Outcome {
        Winner(address),
        Tie,
    }

    #[liquid(definitions)]
    struct Result {
        #[liquid(signers)]
        player1: address,
        #[liquid(signers)]
        player2: address,
        outcome: Outcome,
        moves: Vec<Move>,
        size: u8,
    }

    #[liquid(definitions)]
    struct GameInvite {
        #[liquid(signers)]
        player1: address,
        player2: address,
        size: u8,
    }

    #[liquid(rights)]
    impl GameInvite {
        #[liquid(belongs_to = "player1")]
        pub fn accept(self) -> Game {
            create! { Game =>
                moves: Vec::new(),
                ..self
            }
        }
    }

    #[liquid(definitions)]
    struct Game {
        #[liquid(signer)]
        player1: address,
        #[liquid(signer)]
        player2: address,
        current: address,
        moves: Vec<Move>,
        size: u8,
    }

    enum TurnResult {
        Result(Result),
        Game(Game),
    }

    #[liquid(rights)]
    impl Game {
        #[liquid(belongs_to = "current")]
        pub fn play(self, x: u8, y: u8) -> TurnResult {
            assert!(x < self.size);
            assert!(y < self.size);
            assert!(!self.moves.iter().any(|m| m.x == x && m.y == y));

            let m = Move {
                player: self.current,
                x,
                y,
            };
            self.moves.push(m);

            let opponent = if self.current == self.player1 {
                self.player2
            } else {
                self.player1
            };

            if self.has_won(self.current) {
                let result = create! { Result =>
                    outcome: Outcome::Winner(self.current),
                    ..self
                };
                TurnResult::Result(result)
            } else if self.moves.len() == (self.size * self.size) as usize {
                let result = create! { Result =>
                    outcome = Outcome::Tie,
                    ..self
                };
                TurnResult::Result(result)
            } else {
                self.current = opponent;
                TurnResult::Game(create! { Self =>
                    ..self
                })
            }
        }
    }

    impl Game {
        fn has_won(&self, player: address) -> bool {
            let get_moves = || self.moves.iter().filter(|m| m.player == player);

            let has_all_horizontal =
                |i| get_moves().filter(|m| m.x == i).count() == self.size as usize;
            let has_all_vertical =
                |i| get_moves().filter(|m| m.y == i).count() == self.size as usize;
            let has_all_diagonal = |i| {
                get_moves().filter(|m| m.y == i && m.x == i).count() == self.size as usize
            };
            let has_all_counter_diagonal = |i| {
                get_moves()
                    .filter(|m| m.y == i && m.x == self.size - 1 - i)
                    .count()
                    == self.size as usize
            };

            let has_won_horizontal = (0..self.size).all(|i| has_all_horizontal(i));
            let has_won_vertical = (0..self.size).all(|i| has_all_vertical(i));
            let has_won_diagonal = (0..self.size).all(|i| has_all_diagonal(i));
            let has_won_counter_diagonal =
                (0..self.size).all(|i| has_all_counter_diagonal(i));

            has_won_horizontal
                || has_won_vertical
                || has_won_diagonal
                || has_won_counter_diagonal
        }
    }
}

fn main() {}
