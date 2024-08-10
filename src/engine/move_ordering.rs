use movegen::FullGen;

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

impl Engine {
    pub fn order_moves(&mut self, moves: &mut [Move], killer: Option<Move>) {
        let pawn_attacks = MoveGenerator::<FullGen>::new(&mut self.board).pawn_attack_map();
        let endgame = self.endgame();
        moves.sort_by_cached_key(|&mov| -self.move_order(mov, killer, endgame, pawn_attacks));
    }
    pub fn move_order(&mut self, mov: Move, killer: Option<Move>, endgame: f32, pawn_attacks: Bitboard) -> i32 {
        let mut score = 0;
        if self.only_pv_nodes {
            if let Some(&pv) = self.pv.get(self.depth_from_root as usize) {
                if pv == mov {
                    return i16::MAX as i32;
                }
            }
        } else if let Some(killer) = killer {
            if killer == mov {
                return i16::MAX as i32;
            }
        }
        let piece = self.board[mov.from()].unwrap();

        score += ((abs_piece_square_value(mov.to(), piece, endgame)
            - abs_piece_square_value(mov.from(), piece, endgame)) as f32
            * (0.2 * (1.0 - endgame))) as i32;

        if let Some(target_piece) = self.board[mov.to()] {
            score += MVV_LVA[target_piece.kind()][piece.kind()] as i32 * 4;
        } else if mov.flags() == MoveFlags::EnPassant {
            score += MVV_LVA[Pawn][Pawn] as i32 * 4;
        }

        if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
            score += abs_piece_value(kind, endgame);
        };

        if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
            score += 10;
        }

        if !mov.flags().is_capture() && piece.kind() != Pawn && !pawn_attacks.contains(mov.to()) {
            score += 5;
        }

        score
    }
}
