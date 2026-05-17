use std::{
    fmt,
    ops::{Mul, Neg},
};

use crate::core::side::Side;

// A Eval in centipawns
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Score(pub i32);

impl Score {
    const MATE: Self = Self(i32::MAX - 1);
    pub const MAX: Self = Self(i32::MAX);
    const MIN: Self = Self(-Self::MAX.0);
    const MIN_MATE: Self = Self(i32::MAX - u16::MAX as i32);
    const NEG_MATE: Self = Self(-Self::MATE.0);

    #[must_use]
    pub fn mate_in_ply(ply: u16) -> Self {
        Self(Self::MATE.0 - i32::from(ply))
    }

    #[must_use]
    pub fn mate_ply(self) -> Option<i16> {
        if !(Self::MIN_MATE.0..=Self::MATE.0).contains(&self.0.abs()) {
            return None;
        }
        Some(((Self::MATE.0 - self.0.abs()) * self.0.signum()) as i16)
    }

    #[must_use]
    pub fn mate_moves(self) -> Option<i16> {
        self.mate_ply().map(|ply| ply / 2 + 1)
    }
}

impl Mul<f32> for Score {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self((self.0 as f32 * rhs) as i32)
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Mul<Side> for Score {
    type Output = Self;

    fn mul(self, side: Side) -> Self::Output {
        Self(self.0 * side.positive())
    }
}

impl fmt::Debug for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("Eval");
        match *self {
            Self::MAX => tuple.field(&"max"),
            Self::MIN => tuple.field(&"-min"),
            Self::MATE => tuple.field(&"mate"),
            Self::NEG_MATE => tuple.field(&"-mate"),
            _ => tuple.field(&self.0),
        }
        .finish()
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mate_moves() {
            Some(moves) => write!(f, "mate {moves}"),
            None => write!(f, "cp {}", self.0),
        }
    }
}
