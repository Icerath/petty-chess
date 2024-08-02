use super::evaluation::{abs_piece_square_value, abs_piece_value};
use crate::prelude::*;

impl Engine {
    pub fn order_moves(&mut self, moves: &mut [Move], priority_moves: &[Move]) {
        moves.sort_by_cached_key(|&mov| {
            let endgame = self.endgame();
            let mut score = 0;

            score += priority_moves.contains(&mov) as i32 * i16::MAX as i32;
            let piece = self.board[mov.from()].unwrap();

            score += ((abs_piece_square_value(mov.to(), piece, endgame)
                - abs_piece_square_value(mov.from(), piece, endgame)) as f32
                * (1.0 - endgame)) as i32;

            if let Some(target_piece) = self.board[mov.to()] {
                if piece.kind() != PieceKind::King {
                    score += 4 * abs_piece_value(target_piece.kind(), endgame)
                        - abs_piece_value(piece.kind(), endgame);
                }
            };

            if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
                score += abs_piece_value(kind, endgame);
            };

            -score
        });
        // debug_assert_eq!(&moves[..priority_moves.len()], priority_moves);
    }
}
