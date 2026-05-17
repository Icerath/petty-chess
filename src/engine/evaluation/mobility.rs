use super::Evaluation;
use crate::prelude::*;

impl<F> Evaluation<'_, F> {
    pub fn mobility(&mut self, side: Side) {
        let occupancy = self.board.all_pieces();
        let knight_total: i32 = (self.board.get(side + Knight).into_iter())
            .map(|sq| knight_score((KNIGHT_MOVES[sq] & !self.pawn_attacks[side]).count()))
            .sum();
        let bishop_total: i32 = (self.board.get(side + Bishop).into_iter())
            .map(|sq| bishop_score((bishop_attacks(sq, occupancy)).count()))
            .sum();
        let rook_total: i32 = (self.board.get(side + Rook).into_iter())
            .map(|sq| rook_score((rook_attacks(sq, occupancy)).count()))
            .sum();
        let queen_total: i32 = (self.board.get(side + Queen).into_iter())
            .map(|sq| queen_score((queen_attacks(sq, occupancy)).count()))
            .sum();

        let mut total = 0;
        total += knight_total * 32;
        total += bishop_total * 32;
        self.total[side] += total / 1024;

        self.earlygame[side] += rook_total * 50 / 1024;
        self.endgame[side] += rook_total * 50 / 1024;
        self.earlygame[side] += queen_total * 50 / 1024;
        self.endgame[side] += queen_total * 120 / 1024;
    }
}
const MAX_KNIGHT_MOVES: u8 = 8;
// const MAX_BISHOP_MOVES: u8 = 13;
const EXPECTED_BISHOP_MOVES: u8 = 10;
// const MAX_ROOK_MOVES: u8 = 14;
const EXPECTED_ROOK_MOVES: u8 = 10;
// const MAX_QUEEN_MOVES: u8 = 27;
const EXPECTED_QUEEN_MOVES: u8 = 20;

const fn knight_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    num_moves / MAX_KNIGHT_MOVES as i32
}

const fn bishop_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    num_moves / EXPECTED_BISHOP_MOVES as i32
}

const fn rook_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    num_moves / EXPECTED_ROOK_MOVES as i32
}

const fn queen_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    num_moves / EXPECTED_QUEEN_MOVES as i32
}
