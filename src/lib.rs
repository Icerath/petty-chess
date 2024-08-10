pub mod core;
pub mod engine;
pub mod uci;

pub mod prelude {
    pub use Colour::*;
    pub use Piece::*;
    pub use PieceKind::*;

    pub use super::{
        core::{
            bitboard::Bitboard,
            board::Board,
            can_castle::CanCastle,
            colour::Colour,
            fen,
            move_flags::{Castle, MoveFlags, Promotion},
            movegen::{self, MoveGenerator},
            piece::{Piece, PieceKind},
            r#move::Move,
            square::{File, Rank, Square},
            zobrist::Zobrist,
        },
        engine::Engine,
    };

    pub type Moves = smallvec::SmallVec<[Move; 64]>;
}
