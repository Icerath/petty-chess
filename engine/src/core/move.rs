use core::fmt;

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
    pub fn from(self) -> Pos {
        Pos((self.0 & 0b111111) as i8)
    }
    #[must_use]
    #[inline]
    pub fn to(self) -> Pos {
        Pos(((self.0 >> 6) & 0b111111) as i8)
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

#[test]
fn test_move_repr() {
    let flags = MoveFlags::Capture | MoveFlags::RookPromotion;
    let mov = Move::new(Pos::H8, Pos::A7, flags);
    assert_eq!(mov.from(), Pos::H8);
    assert_eq!(mov.to(), Pos::A7);
    assert_eq!(mov.flags(), flags);
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
