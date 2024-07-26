use crate::prelude::*;

use super::evaluation::{abs_piece_square_value, abs_piece_value};

impl Engine {
    pub fn order_moves(&mut self, moves: &mut [Move]) {
        moves.sort_by_cached_key(|mov| {
            let piece = self.board[mov.from()].unwrap_or(Piece::DEFAULT);
            let mut score = 0;

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
    }
}
