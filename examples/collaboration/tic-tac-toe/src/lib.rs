// https://github.com/digital-asset/ex-models/tree/master/tic-tac-toe
//
// This example models the Tic-Tac-Toe game for two players playing against each other.
//
// # Workflow
// 1. A player invites another player by creating a `GameInvite`.
// 2. Upon acceptance of the invite a `Game` contract is created.
// 3. Player 1 starts the game by exercising the `play` choice, which recreates the `Game` contract with the move added.
// 4. The players now take turns until the `play` choice detects a final state (win or tie).
// 5. When the game is finished a `Result` contract is created containing the state and outcome of the game.

#![cfg_attr(not(feature = "std"), no_std)]

use liquid::InOut;
use liquid_lang as liquid;

#[liquid::collaboration]
mod tic_tac_toe {
    use super::*;

    #[derive(InOut)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Move {
        player: Address,
        x: u8,
        y: u8,
    }

    #[derive(InOut, PartialEq)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub enum Outcome {
        Winner(Address),
        Tie,
    }

    #[liquid(contract)]
    pub struct Result {
        #[liquid(signers)]
        player1: Address,
        #[liquid(signers)]
        player2: Address,
        outcome: Outcome,
        moves: Vec<Move>,
        size: u8,
    }

    #[liquid(contract)]
    pub struct GameInvite {
        #[liquid(signers)]
        player1: Address,
        player2: Address,
        size: u8,
    }

    #[liquid(rights)]
    impl GameInvite {
        #[liquid(belongs_to = "player2")]
        pub fn accept(self) -> ContractId<Game> {
            sign! { Game =>
                current: self.player1.clone(),
                moves: Vec::new(),
                player1: self.player1.clone(),
                player2: self.player2.clone(),
                size: self.size,
            }
        }
    }

    #[liquid(contract)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Game {
        #[liquid(signers)]
        player1: Address,
        #[liquid(signers)]
        player2: Address,
        current: Address,
        moves: Vec<Move>,
        size: u8,
    }

    #[derive(InOut)]
    pub enum TurnResult {
        Result(ContractId<Result>),
        Game(ContractId<Game>),
    }

    #[liquid(rights)]
    impl Game {
        #[liquid(belongs_to = "current")]
        pub fn play(mut self, x: u8, y: u8) -> TurnResult {
            assert!(x < self.size);
            assert!(y < self.size);
            assert!(!self.moves.iter().any(|m| m.x == x && m.y == y));

            self.moves.push(Move {
                player: self.current.clone(),
                x,
                y,
            });

            let opponent = if self.current == self.player1 {
                self.player2.clone()
            } else {
                self.player1.clone()
            };

            if has_won(self.as_ref(), &self.current) {
                let result = sign! { Result =>
                    outcome: Outcome::Winner(self.current.clone()),
                    player1: self.player1.clone(),
                    player2: self.player2.clone(),
                    moves: self.moves,
                    size: self.size,
                };
                TurnResult::Result(result)
            } else if self.moves.len() == (self.size * self.size) as usize {
                let result = sign! { Result =>
                    outcome: Outcome::Tie,
                    player1: self.player1,
                    player2: self.player2,
                    moves: self.moves,
                    size: self.size,
                };
                TurnResult::Result(result)
            } else {
                self.current = opponent;
                TurnResult::Game(sign! { Game =>
                    ..self
                })
            }
        }
    }

    fn has_won(game: &Game, player: &Address) -> bool {
        let get_moves = || game.moves.iter().filter(|m| &m.player == player);

        let has_all_horizontal =
            |i| get_moves().filter(|m| m.x == i).count() == game.size as usize;
        let has_all_vertical =
            |i| get_moves().filter(|m| m.y == i).count() == game.size as usize;
        let has_all_diagonal =
            || get_moves().filter(|m| m.y == m.x).count() == game.size as usize;
        let has_all_counter_diagonal = || {
            get_moves().filter(|m| m.y == game.size - 1 - m.x).count()
                == game.size as usize
        };

        let has_won_horizontal = (0..game.size).any(|i| has_all_horizontal(i));
        let has_won_vertical = (0..game.size).any(|i| has_all_vertical(i));
        let has_won_diagonal = has_all_diagonal();
        let has_won_counter_diagonal = has_all_counter_diagonal();

        has_won_horizontal
            || has_won_vertical
            || has_won_diagonal
            || has_won_counter_diagonal
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use liquid::env::test;

        fn setup(player1: Address, player2: Address) -> ContractId<Game> {
            test::set_caller(player1.clone());
            let invite_id = sign! { GameInvite =>
                player1: player1.clone(),
                player2: player2.clone(),
                size: 3,
            };
            test::pop_execution_context();

            test::set_caller(player2);
            let game_id = invite_id.accept();
            test::pop_execution_context();
            game_id
        }

        fn play(
            mut game_id: ContractId<Game>,
            moves: Vec<(u8, u8)>,
        ) -> ContractId<Result> {
            let game = game_id.fetch();
            let players = [game.player1, game.player2];

            for (i, m) in moves.iter().enumerate() {
                let x = m.0;
                let y = m.1;
                let player = &players[i % 2];
                test::set_caller(player.clone());
                let turn_result = game_id.play(x, y);
                test::pop_execution_context();

                match turn_result {
                    TurnResult::Game(new_game_id) => game_id = new_game_id,
                    TurnResult::Result(res_id) => {
                        if i != moves.len() - 1 {
                            panic!("the game ends unexpectedly at step {}", i + 1);
                        }
                        return res_id;
                    }
                }
            }
            panic!("then game is not over yet...");
        }

        #[test]
        fn won_horizontal() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let res_id = play(game_id, vec![(0, 0), (1, 1), (0, 1), (2, 2), (0, 2)]);

            let result = res_id.fetch();
            assert_eq!(result.player1, alice);
            assert_eq!(result.player2, bob);
            assert_eq!(result.outcome, Outcome::Winner(alice));
            assert_eq!(result.size, 3);
        }
        #[test]
        fn won_vertical() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let res_id = play(game_id, vec![(0, 0), (1, 1), (1, 0), (2, 2), (2, 0)]);

            let result = res_id.fetch();
            assert_eq!(result.player1, alice);
            assert_eq!(result.player2, bob);
            assert_eq!(result.outcome, Outcome::Winner(alice));
            assert_eq!(result.size, 3);
        }

        #[test]
        fn won_diagonal() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let res_id = play(game_id, vec![(0, 0), (0, 1), (1, 1), (0, 2), (2, 2)]);

            let result = res_id.fetch();
            assert_eq!(result.player1, alice);
            assert_eq!(result.player2, bob);
            assert_eq!(result.outcome, Outcome::Winner(alice));
            assert_eq!(result.size, 3);
        }

        #[test]
        fn won_counter_diagonal() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let res_id = play(game_id, vec![(0, 2), (0, 1), (1, 1), (0, 0), (2, 0)]);

            let result = res_id.fetch();
            assert_eq!(result.player1, alice);
            assert_eq!(result.player2, bob);
            assert_eq!(result.outcome, Outcome::Winner(alice));
            assert_eq!(result.size, 3);
        }

        #[test]
        fn tie() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let res_id = play(
                game_id,
                vec![
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (1, 0),
                    (1, 1),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                    (1, 2),
                ],
            );

            let result = res_id.fetch();
            assert_eq!(result.player1, alice);
            assert_eq!(result.player2, bob);
            assert_eq!(result.outcome, Outcome::Tie);
            assert_eq!(result.size, 3);
        }

        #[test]
        #[should_panic]
        fn repeat_move() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let _ = play(game_id, vec![(0, 2), (0, 2)]);
        }

        #[test]
        #[should_panic]
        fn out_of_bounds() {
            let default_accounts = test::default_accounts();
            let alice = default_accounts.alice;
            let bob = default_accounts.bob;
            let game_id = setup(alice.clone(), bob.clone());

            let _ = play(game_id, vec![(0, 3)]);
        }
    }
}
