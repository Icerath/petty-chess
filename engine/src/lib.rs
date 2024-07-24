mod ai;
mod core;

pub mod prelude {
    pub use Colour::*;
    pub use PieceKind::*;

    pub use super::core::{
        board::Board,
        colour::Colour,
        move_flags::{Castle, MoveFlags, Promotion},
        piece::{Piece, PieceKind},
        position::Pos,
        r#move::Move,
    };
}
