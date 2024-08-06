pub mod core;
pub mod engine;
pub mod uci;

pub mod prelude {
    use smallvec::SmallVec;
    pub use Colour::*;
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
            position::{File, Pos, Rank},
            r#move::Move,
            zobrist::Zobrist,
        },
        engine::Engine,
    };

    pub type Moves = SmallVec<[Move; 64]>;
}
