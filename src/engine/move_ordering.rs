use std::{hint::assert_unchecked, mem::MaybeUninit};

use super::{evaluation::evaluation, psqt};
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

pub fn order_moves(board: &mut Board, depth: u8, moves: &mut [Move]) {
    let pawn_attacks = board.pawn_attacks(!board.active_side);
    let phase = phase(board);
    sort_by_cached_key(moves, |mov| move_order(board, mov, phase, depth, pawn_attacks));
}

fn move_order(
    board: &mut Board,
    mov: Move,
    phase: Phase,
    depth: u8,
    pawn_attacks: Bitboard,
) -> i16 {
    let mut score = 0;

    let piece = unsafe { board.get_square_kind(mov.from()).unwrap_unchecked() };

    let piece_sq_diff = psqt::MG[piece + White][mov.to()] - psqt::MG[piece + White][mov.from()];
    score += piece_sq_diff * (200 * phase.earlygame()) / 1024;

    if let Some(target_piece) = board.get_square_kind(mov.to()) {
        unsafe { assert_unchecked(target_piece != PieceKind::King) };
        score += MVV_LVA[target_piece][piece] as i32 * 4;
    } else if mov.flags() == MoveFlags::EnPassant {
        score += MVV_LVA[Pawn][Pawn] as i32 * 4;
    }

    if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
        score += psqt::PIECE_MG[kind] * phase.earlygame();
        score += psqt::PIECE_EG[kind] * phase.endgame();
    }

    if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
        score += 10;
    }

    if !mov.flags().is_capture() && piece != Pawn && !pawn_attacks.contains(mov.to()) {
        score += 5;
    }

    if depth >= 3 {
        let old_eval = if mov.flags().is_capture() { None } else { Some(evaluation(board)) };
        let unmake = board.make_move(mov);
        if let Some(old_eval) = old_eval {
            score += (Score(evaluation(board).0 - old_eval.0) * !board.active_side).0 / 2;
        }
        if board.in_check() {
            score += 10;
        }
        board.unmake_move(unmake);
    }

    score as i16
}
