use core::fmt;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Move(u16);

impl Move {
    #[must_use]
    #[inline]
    pub fn new(from: Pos, to: Pos, flags: MoveFlags) -> Self {
        Self((from.0 as u16) | (to.0 as u16) << 6 | (flags as u16) << 12)
    }
    #[inline]
    #[must_use]
    pub fn from(self) -> Pos {
        Pos((self.0 & 0b11_1111) as i8)
    }
    #[must_use]
    #[inline]
    pub fn to(self) -> Pos {
        Pos(((self.0 >> 6) & 0b11_1111) as i8)
    }
    #[must_use]
    #[inline]
    pub fn flags(self) -> MoveFlags {
        MoveFlags::try_from((self.0 >> 12) as u8).unwrap_or(MoveFlags::Quiet)
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
            Some(Promotion::Knight) => "k",
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
    let flags = MoveFlags::Capture | MoveFlags::RookPromotion;
    let mov = Move::new(Pos::H8, Pos::A7, flags);
    assert_eq!(mov.from(), Pos::H8);
    assert_eq!(mov.to(), Pos::A7);
    assert_eq!(mov.flags(), flags);
}

#[test]
fn test_move_parsing() {
    assert_eq!("e7e5".parse(), Ok(Move::new(Pos::E7, Pos::E5, MoveFlags::Quiet)));
    assert_eq!("e2e4q".parse(), Ok(Move::new(Pos::E2, Pos::E4, MoveFlags::QueenPromotion)));
}
