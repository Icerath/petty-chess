use crate::prelude::*;

const MOBILITY_SCORE_MULTIPLIER: i32 = 2;

pub fn raw_mobility_eval(board: &Board) -> i32 {
    let occupancy = board.all_pieces();
    let mut final_total = 0;
    for side in [White, Black] {
        let mut total = 0;
        // Pawns: TODO
        board.get(side + Knight).for_each(|sq| {
            total += knight_score((KNIGHT_MOVES[sq.usize()]).count());
        });
        board.get(side + Bishop).for_each(|sq| {
            total += bishop_score((bishop_attacks(sq, occupancy)).count());
        });
        board.get(side + Rook).for_each(|sq| {
            total += rook_score((rook_attacks(sq, occupancy)).count());
        });
        board.get(side + Queen).for_each(|sq| {
            total += queen_score((queen_attacks(sq, occupancy)).count());
        });
        final_total += total * side.positive();
    }
    final_total
}

const MAX_KNIGHT_MOVES: u8 = 8;
// const MAX_BISHOP_MOVES: u8 = 13;
const EXPECTED_BISHOP_MOVES: u8 = 10;
// const MAX_ROOK_MOVES: u8 = 14;
const EXPECTED_ROOK_MOVES: u8 = 10;
// const MAX_QUEEN_MOVES: u8 = 27;
const EXPECTED_QUEEN_MOVES: u8 = 20;

fn knight_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    ((num_moves / MAX_KNIGHT_MOVES as i32) * 15 * MOBILITY_SCORE_MULTIPLIER) / 1024
}

fn bishop_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    ((num_moves / EXPECTED_BISHOP_MOVES as i32) * 15 * MOBILITY_SCORE_MULTIPLIER) / 1024
}

fn rook_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    ((num_moves / EXPECTED_ROOK_MOVES as i32) * 25 * MOBILITY_SCORE_MULTIPLIER) / 1024
}

fn queen_score(num_moves: u8) -> i32 {
    let num_moves = num_moves as i32 * 1024;
    ((num_moves / EXPECTED_QUEEN_MOVES as i32) * 45 * MOBILITY_SCORE_MULTIPLIER) / 1024
}
