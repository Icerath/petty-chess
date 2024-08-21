use core::fmt;
use std::ops::Add;

use derive_try_from_primitive::TryFromPrimitive;

use crate::prelude::*;

#[derive(TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Piece {
    BlackPawn,
    WhitePawn,
    BlackKnight,
    WhiteKnight,
    BlackBishop,
    WhiteBishop,
    BlackRook,
    WhiteRook,
    BlackQueen,
    WhiteQueen,
    BlackKing,
    WhiteKing,
}

#[derive(TryFromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Default for Piece {
    #[inline]
    fn default() -> Self {
        Self::WhitePawn
    }
}

impl Piece {
    pub const ALL: [Self; 12] = [
        BlackPawn,
        WhitePawn,
        BlackKnight,
        WhiteKnight,
        BlackBishop,
        WhiteBishop,
        BlackRook,
        WhiteRook,
        BlackQueen,
        WhiteQueen,
        BlackKing,
        WhiteKing,
    ];
    #[must_use]
    #[inline]
    pub fn new(kind: PieceKind, side: Side) -> Self {
        Self::try_from(kind as u8 * 2 + side as u8).unwrap()
    }
    #[must_use]
    #[inline]
    pub fn kind(self) -> PieceKind {
        PieceKind::try_from(self as u8 / 2).unwrap()
    }
    #[must_use]
    #[inline]
    pub fn side(self) -> Side {
        Side::from(self as u8 % 2 == 1)
    }
}

impl Piece {
    #[must_use]
    #[inline]
    pub fn symbol(self) -> char {
        let mut symbol = match self.kind() {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        if self.is_white() {
            symbol.make_ascii_uppercase();
        };
        symbol
    }
    #[must_use]
    #[inline]
    pub fn is_white(self) -> bool {
        self.side().is_white()
    }
    #[must_use]
    #[inline]
    pub fn is_black(self) -> bool {
        self.side().is_black()
    }
}

impl Add<Side> for PieceKind {
    type Output = Piece;
    #[inline]
    fn add(self, side: Side) -> Self::Output {
        Piece::new(self, side)
    }
}

impl Add<PieceKind> for Side {
    type Output = Piece;
    #[inline]
    fn add(self, kind: PieceKind) -> Self::Output {
        Piece::new(kind, self)
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Piece").field("side", &self.side()).field("kind", &self.kind()).finish()
    }
}

impl PieceKind {
    pub const ALL: [Self; 6] = [Pawn, Knight, Bishop, Rook, Queen, King];
}

#[test]
fn test_piece_repr() {
    use PieceKind as P;
    assert_eq!(size_of::<Option<Piece>>(), 1);

    for kind in [P::Pawn, P::Knight, P::Bishop, P::Rook, P::Queen, P::King] {
        for side in [Side::White, Side::Black] {
            let piece = Piece::new(kind, side);
            assert_eq!(piece.kind(), kind);
            assert_eq!(piece.side(), side);
        }
    }
}
