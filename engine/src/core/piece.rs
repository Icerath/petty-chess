use core::fmt;
use std::num::NonZero;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece(NonZero<u8>);

impl Default for Piece {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Piece {
    pub const DEFAULT: Self = Self(NonZero::<u8>::MIN);

    #[must_use]
    #[inline]
    pub fn new(kind: PieceKind, colour: Colour) -> Self {
        Self::default().or_kind(kind).or_colour(colour)
    }

    #[must_use]
    #[inline]
    pub fn or_kind(self, kind: PieceKind) -> Self {
        Self(self.0 | (kind as u8) << 1)
    }
    #[must_use]
    #[inline]
    pub fn or_colour(self, colour: Colour) -> Self {
        Self(self.0 | (colour as u8) << 4)
    }
    #[must_use]
    #[inline]
    pub fn with_kind(self, kind: PieceKind) -> Self {
        Self::new(kind, self.colour())
    }
    #[must_use]
    #[inline]
    pub fn with_colour(self, colour: Colour) -> Self {
        Self::new(self.kind(), colour)
    }
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn kind(self) -> PieceKind {
        PieceKind::try_from(self.u8() >> 1 & 0b111).unwrap()
    }
    #[must_use]
    #[inline]
    pub fn colour(self) -> Colour {
        (self.u8() >> 4 & 1 == 1).into()
    }
    #[inline]
    fn u8(self) -> u8 {
        self.0.into()
    }
}

impl Piece {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<u8> for PieceKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Pawn,
            1 => Knight,
            2 => Bishop,
            3 => Rook,
            4 => Queen,
            5 => King,
            _ => return Err(()),
        })
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

    for kind in [P::Pawn, P::Knight, P::Bishop, P::Rook, P::Queen, P::King] {
        for colour in [Colour::White, Colour::Black] {
            let piece = Piece::new(kind, colour);
            assert_eq!(piece.kind(), kind);
            assert_eq!(piece.colour(), colour);
        }
    }
}
