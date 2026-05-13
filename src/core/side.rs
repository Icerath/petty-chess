use core::ops::Not;

pod_enum! {
    #[derive(Debug)]
    pub enum Side {
        Black,
        White,
    }
}

impl Side {
    #[must_use]
    pub const fn is_black(self) -> bool {
        matches!(self, Self::Black)
    }

    #[must_use]
    pub const fn is_white(self) -> bool {
        matches!(self, Self::White)
    }

    #[must_use]
    pub const fn forward(self) -> i8 {
        match self {
            Self::Black => -1,
            Self::White => 1,
        }
    }

    /// What is considered a beneficial score for this side
    #[must_use]
    pub const fn positive(self) -> i32 {
        match self {
            Self::Black => -1,
            Self::White => 1,
        }
    }

    #[must_use]
    pub const fn symbol(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }
}

impl From<bool> for Side {
    fn from(value: bool) -> Self {
        if value { Self::White } else { Self::Black }
    }
}

impl From<Side> for bool {
    fn from(val: Side) -> Self {
        val as u8 == 1
    }
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}
