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
    pub fn order_moves(&mut self, moves: &mut [Move]) {
        let endgame = self.endgame();
        moves.sort_by_cached_key(|&mov| {
            let mut score = 0;

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

            if self.depth_from_root <= 4 {
                let unmake = self.board.make_move(mov);
                let is_check =
                    MoveGenerator::new(&mut self.board).attack_map().contains(self.board.active_king_pos);
                self.board.unmake_move(unmake);
                if is_check {
                    score += 35;
                }
            }

            -score
        });
    }
}
