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
    pub struct Move {
        player: address,
        x: u8,
        y: u8,
    }

    #[derive(InOut)]
    pub enum Outcome {
        Winner(address),
        Tie,
    }

    #[liquid(contract)]
    pub struct Result {
        #[liquid(signers)]
        player1: address,
        #[liquid(signers)]
        player2: address,
        outcome: Outcome,
        moves: Vec<Move>,
        size: u8,
    }

    #[liquid(contract)]
    pub struct GameInvite {
        #[liquid(signers)]
        player1: address,
        player2: address,
        size: u8,
    }

    #[liquid(rights)]
    impl GameInvite {
        #[liquid(belongs_to = "player2")]
        pub fn accept(self) -> ContractId<Game> {
            sign! { Game =>
                current: self.player1,
                moves: Vec::new(),
                player1: self.player1,
                player2: self.player2,
                size: self.size,
            }
        }
    }

    #[liquid(contract)]
    pub struct Game {
        #[liquid(signers)]
        player1: address,
        #[liquid(signers)]
        player2: address,
        current: address,
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
                player: self.current,
                x,
                y,
            });

            let opponent = if self.current == self.player1 {
                self.player2
            } else {
                self.player1
            };

            if has_won(&self, self.current) {
                let result = sign! { Result =>
                    outcome: Outcome::Winner(self.current),
                    player1: self.player1,
                    player2: self.player1,
                    moves: self.moves,
                    size: self.size,
                };
                TurnResult::Result(result)
            } else if self.moves.len() == (self.size * self.size) as usize {
                let result = sign! { Result =>
                    outcome: Outcome::Tie,
                    player1: self.player1,
                    player2: self.player1,
                    moves: self.moves,
                    size: self.size,
                };
                TurnResult::Result(result)
            } else {
                self.current = opponent;
                TurnResult::Game(sign! { Self =>
                    ..self
                })
            }
        }
    }

    fn has_won(game: &Game, player: address) -> bool {
        let get_moves = || game.moves.iter().filter(|m| m.player == player);

        let has_all_horizontal =
            |i| get_moves().filter(|m| m.x == i).count() == game.size as usize;
        let has_all_vertical =
            |i| get_moves().filter(|m| m.y == i).count() == game.size as usize;
        let has_all_diagonal = |i| {
            get_moves().filter(|m| m.y == i && m.x == i).count() == game.size as usize
        };
        let has_all_counter_diagonal = |i| {
            get_moves()
                .filter(|m| m.y == i && m.x == game.size - 1 - i)
                .count()
                == game.size as usize
        };

        let has_won_horizontal = (0..game.size).all(|i| has_all_horizontal(i));
        let has_won_vertical = (0..game.size).all(|i| has_all_vertical(i));
        let has_won_diagonal = (0..game.size).all(|i| has_all_diagonal(i));
        let has_won_counter_diagonal =
            (0..game.size).all(|i| has_all_counter_diagonal(i));

        has_won_horizontal
            || has_won_vertical
            || has_won_diagonal
            || has_won_counter_diagonal
    }
}
