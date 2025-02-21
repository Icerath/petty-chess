use std::{hint::assert_unchecked, mem::MaybeUninit};

use super::evaluation::{abs_piece_square_value, abs_piece_value};
use crate::prelude::*;

const MVV_LVA: [[u8; 6]; 6] = [
    [15, 14, 13, 12, 11, 10], // victim P, attacker P, N, B, R, Q, K
    [25, 24, 23, 22, 21, 20], // victim N, attacker P, N, B, R, Q, K
    [35, 34, 33, 32, 31, 30], // victim B, attacker P, N, B, R, Q, K
    [45, 44, 43, 42, 41, 40], // victim R, attacker P, N, B, R, Q, K
    [55, 54, 53, 52, 51, 50], // victim Q, attacker P, N, B, R, Q, K
    [0, 0, 0, 0, 0, 0],       // victim K, attacker P, N, B, R, Q, K
];

pub fn sort_by_cached_key<F>(moves: &mut [Move], mut f: F)
where
    F: FnMut(Move) -> i16,
{
    let mut indices = [MaybeUninit::uninit(); 256];
    for (i, mov) in moves.iter().copied().enumerate() {
        indices[i].write((f(mov), i as u8));
    }
    let indices = unsafe { &mut *((&raw mut indices[..moves.len()]) as *mut [(i16, u8)]) };
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
    pub fn order_moves(&mut self, moves: &mut [Move], killer: Option<Move>) {
        let pawn_attacks = self.board.pawn_attacks(!self.board.active_side);
        let phase = phase(&self.board);
        sort_by_cached_key(moves, |mov| -self.move_order(mov, killer, phase, pawn_attacks));
    }

    fn move_order(
        &mut self,
        mov: Move,
        killer: Option<Move>,
        phase: Phase,
        pawn_attacks: Bitboard,
    ) -> i16 {
        let mut score = 0;
        if self.only_pv_nodes {
            if let Some(&pv) = self.pv.get(self.depth_from_root as usize) {
                if pv == mov {
                    return i16::MAX;
                }
            }
        } else if let Some(killer) = killer {
            if killer == mov {
                return i16::MAX;
            }
        }
        let piece = unsafe { self.board.get_square_kind(mov.from()).unwrap_unchecked() };

        let piece_sq_diff = abs_piece_square_value(mov.to(), piece + self.board.active_side, phase)
            - abs_piece_square_value(mov.from(), piece + self.board.active_side, phase);
        score += piece_sq_diff * (200 * phase.earlygame()) / 1024;

        if let Some(target_piece) = self.board.get_square_kind(mov.to()) {
            unsafe { assert_unchecked(target_piece != PieceKind::King) };
            score += MVV_LVA[target_piece as usize][piece as usize] as i32 * 4;
        } else if mov.flags() == MoveFlags::EnPassant {
            score += MVV_LVA[Pawn as usize][Pawn as usize] as i32 * 4;
        }

        if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
            score += abs_piece_value(kind, phase);
        }

        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            score += 10;
        }

        if !mov.flags().is_capture() && piece != Pawn && !pawn_attacks.contains(mov.to()) {
            score += 5;
        }

        score as i16
    }
}
