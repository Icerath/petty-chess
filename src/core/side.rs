use core::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Side {
    Black = 0,
    White = 1,
}

impl Side {
    #[must_use]
    #[inline]
    pub fn is_black(self) -> bool {
        self == Self::Black
    }
    #[must_use]
    #[inline]
    pub fn is_white(self) -> bool {
        self == Self::White
    }
    #[must_use]
    #[inline]
    pub const fn forward(self) -> i8 {
        match self {
            Self::Black => -1,
            Self::White => 1,
        }
    }
    /// What is considered a beneficial score for this side
    #[inline]
    #[must_use]
    pub const fn positive(self) -> i32 {
        match self {
            Self::Black => -1,
            Self::White => 1,
        }
    }
}

#[rustfmt::skip]
impl From<bool> for Side {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Side::White } else { Side::Black }
    }
}

impl From<Side> for bool {
    #[inline]
    fn from(val: Side) -> Self {
        val as u8 == 1
    }
}

impl Not for Side {
    type Output = Side;
    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}
