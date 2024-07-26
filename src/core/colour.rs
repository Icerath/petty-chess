use core::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Colour {
    Black,
    White,
}

impl Colour {
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
    pub fn forward(self) -> i8 {
        match self {
            Self::Black => -1,
            Self::White => 1,
        }
    }
    /// What is considered a beneficial score for this colour
    #[inline]
    #[must_use]
    pub fn positive(self) -> i32 {
        self.forward() as i32
    }
}

#[rustfmt::skip]
impl From<bool> for Colour {
    #[inline]fn from(value: bool) -> Self {
        if value { Colour::White } else { Colour::Black }
    }
}

impl From<Colour> for bool {
    #[inline]
    fn from(val: Colour) -> Self {
        val as u8 == 1
    }
}

impl Not for Colour {
    type Output = Colour;
    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}
