use core::fmt;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Move(u16);

#[cfg(feature = "serde")]
impl serde::Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Move {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = serde::Deserialize::deserialize(deserializer)?;

        Self::from_str(str)
            .map_err(|()| serde::de::Error::custom(format_args!("Unexpected: {str:?}")))
    }
}

impl Move {
    pub const NULL: Self = Self(0);

    #[must_use]
    pub const fn rank_diff(self) -> u8 {
        self.from().rank().diff(self.to().rank())
    }

    #[must_use]
    pub const fn file_diff(self) -> u8 {
        self.from().file().diff(self.to().file())
    }

    #[must_use]
    pub const fn with_flags(self, flags: MoveFlags) -> Self {
        Self::new(self.from(), self.to(), flags)
    }

    #[must_use]
    pub const fn new(from: Square, to: Square, flags: MoveFlags) -> Self {
        Self(from as u16 | ((to as u16) << 6) | ((flags as u16) << 12))
    }

    #[must_use]
    pub const fn from(self) -> Square {
        unsafe { Square::from_int_unchecked((self.0 & 0b11_1111) as u8) }
    }

    #[must_use]
    pub const fn to(self) -> Square {
        unsafe { Square::from_int_unchecked(((self.0 >> 6) & 0b11_1111) as u8) }
    }

    #[must_use]
    pub fn flags(self) -> MoveFlags {
        unsafe { MoveFlags::try_from((self.0 >> 12) as u8).unwrap_unchecked() }
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
        Ok(Self::new(from, to, flags))
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
