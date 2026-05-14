#![feature(macro_metavar_expr)]

pub mod core;
pub mod engine;
pub mod uci;

pub mod prelude {
    pub use Piece::*;
    pub use PieceKind::*;
    pub use Side::*;

    pub use super::{
        core::{
            bitboard::Bitboard,
            board::Board,
            can_castle::CanCastle,
            fen,
            magic::{bishop_attacks, queen_attacks, rook_attacks},
            r#move::Move,
            move_flags::{Castle, MoveFlags, Promotion},
            movegen::{self, KNIGHT_MOVES},
            moves::Moves,
            piece::{Piece, PieceKind},
            side::Side,
            square::{File, Rank, Square},
            zobrist::Zobrist,
        },
        engine::{Engine, Phase, Score, phase},
    };
}
