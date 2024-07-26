use std::{ops::BitOr, str::FromStr};

use derive_try_from_primitive::TryFromPrimitive;
use crate::prelude::*;

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
pub enum MoveFlags {
    #[default]
    Quiet = 0b0000,
    DoublePawnPush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EnPassant = 0b0101,
    _6 = 0b0110,
    _7 = 0b0111,
    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,
    KnightPromotionCapture = 0b1100,
    BishopPromotionCapture = 0b1101,
    RookPromotionCapture = 0b1110,
    QueenPromotionCapture = 0b1111,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Castle {
    KingSide,
    QueenSide,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Promotion {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl MoveFlags {
    #[must_use]
    #[inline]
    pub fn promotion(self) -> Option<Promotion> {
        Some(match self {
            Self::BishopPromotion | Self::BishopPromotionCapture => Promotion::Bishop,
            Self::KnightPromotion | Self::KnightPromotionCapture => Promotion::Knight,
            Self::RookPromotion | Self::RookPromotionCapture => Promotion::Rook,
            Self::QueenPromotion | Self::QueenPromotionCapture => Promotion::Queen,
            _ => return None,
        })
    }
    #[must_use]
    pub fn is_capture(self) -> bool {
        self as u8 & 0b0100 == 0b0100
    }
}

impl From<Castle> for MoveFlags {
    fn from(value: Castle) -> Self {
        match value {
            Castle::KingSide => Self::KingCastle,
            Castle::QueenSide => Self::QueenCastle,
        }
    }
}

impl From<Promotion> for MoveFlags {
    fn from(value: Promotion) -> Self {
        match value {
            Promotion::Knight => Self::KnightPromotion,
            Promotion::Bishop => Self::BishopPromotion,
            Promotion::Rook => Self::RookPromotion,
            Promotion::Queen => Self::QueenPromotion,
        }
    }
}

impl From<Promotion> for PieceKind {
    fn from(promotion: Promotion) -> Self {
        match promotion {
            Promotion::Knight => Knight,
            Promotion::Bishop => Bishop,
            Promotion::Rook => Rook,
            Promotion::Queen => Queen,
        }
    }
}

impl BitOr<Self> for MoveFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { Self::try_from(self as u8 | rhs as u8).unwrap_unchecked() }
    }
}

impl BitOr<Promotion> for MoveFlags {
    type Output = Self;
    fn bitor(self, rhs: Promotion) -> Self::Output {
        self | Self::from(rhs)
    }
}

impl FromStr for Promotion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "n" => Self::Knight,
            "b" => Self::Bishop,
            "r" => Self::Rook,
            "q" => Self::Queen,
            _ => return Err(()),
        })
    }
}

#[test]
fn test_moveflags() {
    assert_eq!(MoveFlags::Capture | MoveFlags::Quiet, MoveFlags::Capture);
    assert_eq!(MoveFlags::from(Promotion::Bishop), MoveFlags::BishopPromotion);
    assert_eq!(MoveFlags::from(Promotion::Queen) | MoveFlags::Capture, MoveFlags::QueenPromotionCapture);
}
