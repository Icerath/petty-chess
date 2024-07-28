use crate::prelude::*;

impl Engine {
    pub fn evaluate(&mut self) -> i32 {
        self.raw_evaluation() * self.board.active_colour.positive()
    }
    pub fn raw_evaluation(&mut self) -> i32 {
        self.total_nodes += 1;
        let mut sum = 0;
        for pos in (0..64).map(Pos) {
            let Some(piece) = self.board[pos] else { continue };
            sum += piece_value(piece);
            sum += piece_square_value(pos, piece, self.endgame());
        }
        sum
    }
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

pub fn piece_square_value(pos: Pos, piece: Piece, endgame: f32) -> i32 {
    abs_piece_square_value(pos, piece.kind(), endgame) * piece.colour().positive()
}

pub fn abs_piece_square_value(pos: Pos, piece: PieceKind, endgame: f32) -> i32 {
    let index = Pos::new(Rank(7 - pos.rank().0), pos.file()).0 as usize;
    match piece {
        PieceKind::Pawn => square_tables::PAWN[index],
        PieceKind::Knight => square_tables::KNIGHT[index],
        PieceKind::Bishop => square_tables::BISHOP[index],
        PieceKind::Rook => square_tables::ROOK[index],
        PieceKind::Queen => square_tables::QUEEN[index],
        PieceKind::King => {
            (square_tables::KING_MIDDLE[index] as f32 * (1.0 - endgame)
                + square_tables::KING_LATE[index] as f32 * endgame) as i32
        }
    }
}

mod square_tables {
    #[rustfmt::skip]
    pub const PAWN: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
        5,  5, 10, 25, 25, 10,  5,  5,
        0,  0,  0, 20, 20,  0,  0,  0,
        5, -5,-10,  0,  0,-10, -5,  5,
        5, 10, 10,-20,-20, 10, 10,  5,
        0,  0,  0,  0,  0,  0,  0,  0
     ];

    #[rustfmt::skip]
    pub const KNIGHT: [i32; 64] = [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ];

    #[rustfmt::skip]
    pub const BISHOP: [i32; 64] = [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ];

    #[rustfmt::skip]
    pub const ROOK: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ];

    #[rustfmt::skip]
    pub const QUEEN: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ];

    #[rustfmt::skip]
    pub const KING_MIDDLE: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ];

    #[rustfmt::skip]
    pub const KING_LATE: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ];
}
