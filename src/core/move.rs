use core::fmt;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Move(u16);

impl Move {
    pub const NULL: Move = Self(0);
    #[must_use]
    #[inline]
    pub fn new(from: Square, to: Square, flags: MoveFlags) -> Self {
        Self((u16::from(from)) | (u16::from(to)) << 6 | (flags as u16) << 12)
    }
    #[inline]
    #[must_use]
    pub fn from(self) -> Square {
        unsafe { Square::new_int_unchecked((self.0 & 0b11_1111) as u8) }
    }
    #[must_use]
    #[inline]
    pub fn to(self) -> Square {
        unsafe { Square::new_int_unchecked(((self.0 >> 6) & 0b11_1111) as u8) }
    }
    #[must_use]
    #[inline]
    pub fn flags(self) -> MoveFlags {
        MoveFlags::try_from((self.0 >> 12) as u8).unwrap()
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field("flags", &self.flags())
            .finish()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promote = match self.flags().promotion() {
            Some(Promotion::Knight) => "n",
            Some(Promotion::Bishop) => "b",
            Some(Promotion::Rook) => "r",
            Some(Promotion::Queen) => "q",
            None => "",
        };
        write!(f, "{}{}{}", self.from(), self.to(), promote)
    }
}

impl FromStr for Move {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let 4..=5 = s.len() else { return Err(()) };

        let from = s[..2].parse().map_err(|_| ())?;
        let to = s[2..4].parse().map_err(|_| ())?;

        let flags = match s.as_bytes().get(4) {
            Some(b'n') => MoveFlags::KnightPromotion,
            Some(b'b') => MoveFlags::BishopPromotion,
            Some(b'r') => MoveFlags::RookPromotion,
            Some(b'q') => MoveFlags::QueenPromotion,
            Some(_) => return Err(()),
            None => MoveFlags::default(),
        };
        Ok(Move::new(from, to, flags))
    }
}

#[test]
fn test_move_repr() {
    let flags = MoveFlags::RookPromotionCapture;
    let mov = Move::new(Square::H8, Square::A7, flags);
    assert_eq!(mov.from(), Square::H8);
    assert_eq!(mov.to(), Square::A7);
    assert_eq!(mov.flags(), flags);
}

#[test]
fn test_move_parsing() {
    assert_eq!("e7e5".parse(), Ok(Move::new(Square::E7, Square::E5, MoveFlags::Quiet)));
    assert_eq!("e2e4q".parse(), Ok(Move::new(Square::E2, Square::E4, MoveFlags::QueenPromotion)));
}
