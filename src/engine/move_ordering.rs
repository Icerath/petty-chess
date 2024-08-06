use super::evaluation::{abs_piece_square_value, abs_piece_value};
use crate::prelude::*;

impl Engine {
    pub fn order_moves(&mut self, moves: &mut [Move]) {
        moves.sort_by_cached_key(|&mov| {
            let endgame = self.endgame();
            let mut score = 0;

            let piece = self.board[mov.from()].unwrap();

            score += ((abs_piece_square_value(mov.to(), piece, endgame)
                - abs_piece_square_value(mov.from(), piece, endgame)) as f32
                * (1.0 - endgame)) as i32;

            if let Some(target_piece) = self.board[mov.to()] {
                score +=
                    4 * abs_piece_value(target_piece.kind(), endgame) - abs_piece_value(piece.kind(), endgame);
            };

            if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
                score += abs_piece_value(kind, endgame);
            };

            if mov.flags() == MoveFlags::KingCastle || mov.flags() == MoveFlags::QueenCastle {
                score += 20;
            }

            if self.depth_from_root <= 4 {
                let unmake = self.board.make_move(mov);
                let is_check =
                    MoveGenerator::new(&mut self.board).attack_map().contains(self.board.active_king_pos);
                self.board.unmake_move(unmake);
                if is_check {
                    score += 20;
                }
            }

            -score
        });
    }
}
