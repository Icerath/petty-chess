use core::fmt;
use std::ops::Add;

use derive_try_from_primitive::TryFromPrimitive;

use crate::prelude::*;

#[derive(TryFromPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 6,
    BlackKnight = 7,
    BlackBishop = 8,
    BlackRook = 9,
    BlackQueen = 10,
    BlackKing = 11,
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
    #[must_use]
    #[inline]
    pub fn new(kind: PieceKind, colour: Colour) -> Self {
        Self::try_from(kind as u8 + colour as u8 * 6).unwrap()
    }
    #[must_use]
    #[inline]
    pub fn kind(self) -> PieceKind {
        PieceKind::try_from(self as u8 % 6).unwrap()
    }
    #[must_use]
    #[inline]
    pub fn colour(self) -> Colour {
        Colour::from(self as u8 / 6 == 1)
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
        self.colour().is_white()
    }
    #[must_use]
    #[inline]
    pub fn is_black(self) -> bool {
        self.colour().is_black()
    }
}

impl Add<Colour> for PieceKind {
    type Output = Piece;
    #[inline]
    fn add(self, colour: Colour) -> Self::Output {
        Piece::new(self, colour)
    }
}

impl Add<PieceKind> for Colour {
    type Output = Piece;
    #[inline]
    fn add(self, kind: PieceKind) -> Self::Output {
        Piece::new(kind, self)
    }
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Piece").field("colour", &self.colour()).field("kind", &self.kind()).finish()
    }
}

#[test]
fn test_piece_repr() {
    use PieceKind as P;
    assert_eq!(size_of::<Option<Piece>>(), 1);

    for kind in [P::Pawn, P::Knight, P::Bishop, P::Rook, P::Queen, P::King] {
        for colour in [Colour::White, Colour::Black] {
            let piece = Piece::new(kind, colour);
            assert_eq!(piece.kind(), kind);
            assert_eq!(piece.colour(), colour);
        }
    }
}
