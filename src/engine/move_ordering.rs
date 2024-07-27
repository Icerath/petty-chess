use super::evaluation::{abs_piece_square_value, abs_piece_value};
use crate::prelude::*;

impl Engine {
    pub fn order_moves(&mut self, moves: &mut [Move], priority_moves: &[Move]) {
        moves.sort_by_cached_key(|&mov| {
            let mut score = 0;
            score += priority_moves.contains(&mov) as i32 * i16::MAX as i32;

            let piece = self.board[mov.from()].unwrap_or(Piece::DEFAULT);

            score += abs_piece_square_value(mov.to(), piece.kind(), self.endgame())
                - abs_piece_square_value(mov.from(), piece.kind(), self.endgame());

            if let Some(target_piece) = self.board[mov.to()] {
                score += abs_piece_value(target_piece.kind()) - abs_piece_value(piece.kind());
            };

            if let Some(kind) = mov.flags().promotion().map(PieceKind::from) {
                score += abs_piece_value(kind);
            };

            -score
        });
        debug_assert_eq!(&moves[..priority_moves.len()], priority_moves);
    }
}
