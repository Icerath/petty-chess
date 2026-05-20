use std::mem::MaybeUninit;

use arrayvec::ArrayVec;

use super::psqt;
use crate::{
    core::movegen::{KING_MOVES, PAWN_ATTACKS},
    prelude::*,
};

const MAX_CAPTURES: usize = 80;

fn sort_by_cached_key<F, T: Ord + Copy, U: Copy>(moves: &mut [U], mut f: F)
where
    F: FnMut(U) -> T,
{
    let mut indices = [MaybeUninit::uninit(); MAX_CAPTURES];
    for (i, mov) in moves.iter().copied().enumerate() {
        indices[i].write((f(mov), i as u8));
    }
    let indices = unsafe { &mut *((&raw mut indices[..moves.len()]) as *mut [(T, u8)]) };
    indices.sort_unstable_by_key(|(k, _)| *k);
    for i in 0..moves.len() {
        let mut index = indices[i].1;
        while (index as usize) < i {
            index = indices[index as usize].1;
        }
        indices[i].1 = index;
        moves.swap(i, index as usize);
    }
}

impl Engine {
    pub fn order_moves_capture(&mut self, moves: &[Move]) -> [ArrayVec<Move, 80>; 2] {
        let mut good_captures = ArrayVec::<_, MAX_CAPTURES>::new();
        let mut bad_captures = ArrayVec::<_, MAX_CAPTURES>::new();
        let phase = phase(&self.board);
        for &mov in moves {
            let score = self.order_capture(phase, mov);
            if score < 0 {
                bad_captures.push((mov, score));
            } else {
                good_captures.push((mov, score));
            }
        }
        sort_by_cached_key(&mut good_captures, |(_, score)| score);
        sort_by_cached_key(&mut bad_captures, |(_, score)| score);
        let good = good_captures.iter().map(|(mov, _)| *mov).collect();
        let bad = bad_captures.iter().map(|(mov, _)| *mov).collect();
        [good, bad]
    }

    pub fn order_moves_history(&mut self, moves: &mut [Move]) {
        sort_by_cached_key(moves, |mov| {
            if let Some(Promotion::Queen) = mov.flags().promotion() {
                return u32::MAX;
            }
            let piece = self.board.get_square(mov.from()).unwrap();
            self.history_table[piece][mov.to()]
        });
    }

    fn order_capture(&mut self, phase: Phase, mov: Move) -> i32 {
        let captured_piece = self.board.get_square_kind(mov.to()).unwrap_or(Pawn);
        let captured_piece_value = psqt::PIECE_MG[captured_piece] * phase.earlygame()
            + psqt::PIECE_EG[captured_piece] * phase.endgame();
        let board = self.board.clone();
        self.board.make_move_no_update(mov);
        let value = captured_piece_value - self.see(mov.to(), phase);
        self.board = board;
        value
    }

    fn see(&mut self, sq: Square, phase: Phase) -> i32 {
        let mut value = 0;
        if let Some(attacker) = get_smallest_attacker(&self.board, sq) {
            let captured_piece = self.board.get_square_kind(sq).unwrap();
            let captured_piece_value = psqt::PIECE_MG[captured_piece] * phase.earlygame()
                + psqt::PIECE_EG[captured_piece] * phase.endgame();

            let board = self.board.clone();
            self.board.make_move_no_update(Move::new(attacker, sq, MoveFlags::Capture));
            value = captured_piece_value - self.see(sq, phase).max(0);
            self.board = board;
        }
        value
    }
}

fn get_smallest_attacker(board: &Board, sq: Square) -> Option<Square> {
    let side = !board.active_side;
    let occupancy = board.all_pieces();
    if let Some(sq) = (PAWN_ATTACKS[side][sq] & board.get(side + Pawn)).bitscan() {
        return Some(sq);
    }
    if let Some(sq) = (KNIGHT_MOVES[sq] & board.get(side + Knight)).bitscan() {
        return Some(sq);
    }
    if let Some(sq) = (bishop_attacks(sq, occupancy) & board.get(side + Bishop)).bitscan() {
        return Some(sq);
    }
    if let Some(sq) = (rook_attacks(sq, occupancy) & board.get(side + Rook)).bitscan() {
        return Some(sq);
    }
    if let Some(sq) = (KING_MOVES[sq] & board.get(side + King)).bitscan() {
        return Some(sq);
    }
    None
}
