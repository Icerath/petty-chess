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
            move_flags::{Castle, MoveFlags, Promotion},
            movegen::{self, MoveGenerator},
            piece::{Piece, PieceKind},
            r#move::Move,
            side::Side,
            square::{File, Rank, Square},
            zobrist::Zobrist,
        },
        engine::{Engine, Phase},
    };

    pub type Moves = smallvec::SmallVec<[Move; 64]>;
}
