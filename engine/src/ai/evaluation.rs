use crate::prelude::*;

impl Engine {
    pub fn evaluate(&mut self) -> i32 {
        evaluate(&self.board) * self.board.active_colour.positive()
    }
}

pub fn evaluate(board: &Board) -> i32 {
    let mut sum = 0;
    for pos in (0..64).map(Pos) {
        let Some(piece) = board[pos] else { continue };
        sum += piece_value(piece);
    }
    sum
}

pub fn piece_value(piece: Piece) -> i32 {
    abs_piece_value(piece.kind()) * piece.colour().positive()
}

pub fn abs_piece_value(piece: PieceKind) -> i32 {
    match piece {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 20000,
    }
}
