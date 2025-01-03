#![feature(portable_simd)]

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
            move_flags::{Castle, MoveFlags, Promotion},
            movegen::{self, MoveGenerator, KNIGHT_MOVES},
            moves::Moves,
            piece::{Piece, PieceKind},
            r#move::Move,
            side::Side,
            square::{File, Rank, Square},
            zobrist::Zobrist,
        },
        engine::{phase, Engine, Eval, Phase},
    };
}
