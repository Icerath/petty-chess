use movegen::KNIGHT_MOVES;

use crate::prelude::*;

const MOBILITY_SCORE_MULTIPLIER: f32 = 2.0;

impl Engine {
    #[must_use]
    pub fn raw_mobility_eval(&self) -> i32 {
        let occupancy = self.board.all_pieces();
        let mut final_total = 0;
        for side in [White, Black] {
            let mut total = 0;
            // Pawns: TODO
            self.board.get(side + Knight).for_each(|sq| {
                total += knight_score((KNIGHT_MOVES[sq]).count());
            });
            self.board.get(side + Bishop).for_each(|sq| {
                total += bishop_score((self.magic.bishop_attacks(sq, occupancy)).count());
            });
            self.board.get(side + Rook).for_each(|sq| {
                total += rook_score((self.magic.rook_attacks(sq, occupancy)).count());
            });
            self.board.get(side + Rook).for_each(|sq| {
                total += rook_score((self.magic.rook_attacks(sq, occupancy)).count());
            });
            self.board.get(side + Queen).for_each(|sq| {
                total += queen_score((self.magic.queen_attacks(sq, occupancy)).count());
            });
            final_total += total * side.positive();
        }
        final_total
    }
}

const MAX_KNIGHT_MOVES: u8 = 8;
// const MAX_BISHOP_MOVES: u8 = 13;
const EXPECTED_BISHOP_MOVES: u8 = 10;
// const MAX_ROOK_MOVES: u8 = 14;
const EXPECTED_ROOK_MOVES: u8 = 10;
// const MAX_QUEEN_MOVES: u8 = 27;
const EXPECTED_QUEEN_MOVES: u8 = 20;

fn knight_score(num_moves: u8) -> i32 {
    ((num_moves as f32 / MAX_KNIGHT_MOVES as f32) * 15.0 * MOBILITY_SCORE_MULTIPLIER) as i32
}
fn bishop_score(num_moves: u8) -> i32 {
    ((num_moves as f32 / EXPECTED_BISHOP_MOVES as f32).min(1.0) * 15.0 * MOBILITY_SCORE_MULTIPLIER) as i32
}
fn rook_score(num_moves: u8) -> i32 {
    ((num_moves as f32 / EXPECTED_ROOK_MOVES as f32).min(1.0) * 25.0 * MOBILITY_SCORE_MULTIPLIER) as i32
}
fn queen_score(num_moves: u8) -> i32 {
    ((num_moves as f32 / EXPECTED_QUEEN_MOVES as f32).min(1.0) * 45.0 * MOBILITY_SCORE_MULTIPLIER) as i32
}
