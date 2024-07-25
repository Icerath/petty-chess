use std::{fmt, str::FromStr};

#[derive(Clone, Copy, PartialEq)]
pub struct Pos(pub u8);

impl Pos {
    #[must_use]
    #[inline]
    pub fn new(row: Row, col: Col) -> Self {
        Self(col.0 + row.0 * 8)
    }
    #[must_use]
    #[inline]
    pub fn col(self) -> Col {
        Col(self.0 % 8)
    }
    #[must_use]
    #[inline]
    pub fn row(self) -> Row {
        Row(self.0 / 8)
    }
    #[must_use]
    #[inline]
    pub fn add_row(self, row: i8) -> Option<Self> {
        let row = self.row().checked_add(row)?;
        Some(Self::new(row, self.col()))
    }
    #[must_use]
    #[inline]
    pub fn add_col(self, col: i8) -> Option<Self> {
        let col = self.col().checked_add(col)?;
        Some(Self::new(self.row(), col))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Row(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Col(pub u8);

impl Row {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = (self.0 as i8) + rhs;
        (0..8).contains(&out).then_some(Self(out as u8))
    }
}

impl Col {
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    pub fn checked_add(self, rhs: i8) -> Option<Self> {
        let out = (self.0 as i8) + rhs;
        (0..8).contains(&out).then_some(Self(out as u8))
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::SQUARES[self.0 as usize])
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl FromStr for Pos {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::SQUARES.iter().position(|&square| square == input).map(|index| Pos(index as u8)).ok_or(())
    }
}

impl Pos {
    pub fn algebraic(self) -> &'static str {
        Self::SQUARES[self.0 as usize]
    }
    #[rustfmt::skip]
    pub const SQUARES: [&'static str; 64] = [
        "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8",
        "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8",
        "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8",
        "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8",
        "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8",
        "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8",
        "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8",
        "h1", "h2", "h3", "h4", "h5", "h6", "h7", "h8",
    ];

    pub const A1: Self = Self(0);
    pub const A2: Self = Self(1);
    pub const A3: Self = Self(2);
    pub const A4: Self = Self(3);
    pub const A5: Self = Self(4);
    pub const A6: Self = Self(5);
    pub const A7: Self = Self(6);
    pub const A8: Self = Self(7);

    pub const B1: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const B3: Self = Self(10);
    pub const B4: Self = Self(11);
    pub const B5: Self = Self(12);
    pub const B6: Self = Self(13);
    pub const B7: Self = Self(14);
    pub const B8: Self = Self(15);

    pub const C1: Self = Self(16);
    pub const C2: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const C4: Self = Self(19);
    pub const C5: Self = Self(20);
    pub const C6: Self = Self(21);
    pub const C7: Self = Self(22);
    pub const C8: Self = Self(23);

    pub const D1: Self = Self(24);
    pub const D2: Self = Self(25);
    pub const D3: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const D5: Self = Self(28);
    pub const D6: Self = Self(29);
    pub const D7: Self = Self(30);
    pub const D8: Self = Self(31);

    pub const E1: Self = Self(32);
    pub const E2: Self = Self(33);
    pub const E3: Self = Self(34);
    pub const E4: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const E6: Self = Self(37);
    pub const E7: Self = Self(38);
    pub const E8: Self = Self(39);

    pub const F1: Self = Self(40);
    pub const F2: Self = Self(41);
    pub const F3: Self = Self(42);
    pub const F4: Self = Self(43);
    pub const F5: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const F7: Self = Self(46);
    pub const F8: Self = Self(47);

    pub const G1: Self = Self(48);
    pub const G2: Self = Self(49);
    pub const G3: Self = Self(50);
    pub const G4: Self = Self(51);
    pub const G5: Self = Self(52);
    pub const G6: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const G8: Self = Self(55);

    pub const H1: Self = Self(56);
    pub const H2: Self = Self(57);
    pub const H3: Self = Self(58);
    pub const H4: Self = Self(59);
    pub const H5: Self = Self(60);
    pub const H6: Self = Self(61);
    pub const H7: Self = Self(62);
    pub const H8: Self = Self(63);
}
